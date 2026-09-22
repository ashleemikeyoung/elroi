//! Reading and editing .docx files without rebuilding them.
//!
//! A .docx file is a zip package of XML parts (document, styles, numbering,
//! headers, footers, footnotes, comments, media, settings, ...). Earlier
//! versions of this tool parsed the document into a simplified model and wrote
//! a brand-new package from it, which silently dropped tables, images,
//! footnotes, headers, page setup and run formatting, and replaced whole
//! paragraphs instead of the requested words.
//!
//! This implementation edits the package in place: every part is carried over
//! byte-for-byte except the few that an operation must touch, and inside the
//! main document only the matched `<w:t>` text nodes (or the insertion point)
//! are rewritten. Formatting of the surrounding runs is kept.
//!
//! All byte offsets used for slicing come from ASCII delimiters (`<`, `>`,
//! quotes, `&`, `;`) found by `str::find`, so they always fall on UTF-8
//! character boundaries.
#![allow(clippy::string_slice)]

use image::{self, GenericImageView, ImageFormat};
use rmcp::model::{ContentBlock, ErrorCode, ErrorData};
use std::borrow::Cow;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

const EMU_PER_PIXEL: u64 = 9_525;
const MAX_IMAGE_WIDTH_EMU: u64 = 6 * 914_400; // 6 inches, fits a letter page with 1" margins
const IMAGE_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

fn docx_error(message: impl Into<String>) -> ErrorData {
    ErrorData {
        code: ErrorCode::INTERNAL_ERROR,
        message: Cow::from(message.into()),
        data: None,
    }
}

fn invalid_params(message: impl Into<String>) -> ErrorData {
    ErrorData {
        code: ErrorCode::INVALID_PARAMS,
        message: Cow::from(message.into()),
        data: None,
    }
}

fn not_a_docx_error(path: &str, bytes: &[u8]) -> ErrorData {
    if let Ok(text) = std::str::from_utf8(bytes) {
        let preview: String = text
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .chars()
            .take(60)
            .collect();
        let stem = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        return docx_error(format!(
            "{path} is not a real Word document: it is plain text (it begins with \"{preview}\") \
             saved with a .docx name, which is why Word reports it as unreadable. Nothing was \
             changed. To turn it into a Word document, convert it, for example: \
             pandoc -f markdown -t docx \"{path}\" -o \"{stem} (converted).docx\""
        ));
    }
    docx_error(format!(
        "{path} is not a valid .docx file (it is not a zip package). Nothing was changed."
    ))
}

// ---------------------------------------------------------------------------
// Styling
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct DocxStyle {
    bold: bool,
    italic: bool,
    underline: bool,
    /// Font size in points.
    size: Option<u64>,
    color: Option<String>,
    /// Value for `<w:jc w:val=...>`.
    alignment: Option<&'static str>,
}

impl DocxStyle {
    fn from_json(value: &serde_json::Value) -> Option<Self> {
        let obj = value.as_object()?;
        let flag = |key: &str| obj.get(key).and_then(|v| v.as_bool()).unwrap_or(false);
        Some(Self {
            bold: flag("bold"),
            italic: flag("italic"),
            underline: flag("underline"),
            size: obj.get("size").and_then(|v| v.as_u64()).filter(|s| *s > 0),
            color: obj
                .get("color")
                .and_then(|v| v.as_str())
                .map(|c| c.trim_start_matches('#').to_ascii_uppercase())
                .filter(|c| c.len() == 6 && c.chars().all(|ch| ch.is_ascii_hexdigit())),
            alignment: obj
                .get("alignment")
                .and_then(|v| v.as_str())
                .and_then(parse_alignment),
        })
    }

    /// Run properties in schema order (b, i, color, sz, szCs, u).
    fn run_properties(&self) -> String {
        let mut rpr = String::new();
        if self.bold {
            rpr.push_str("<w:b/><w:bCs/>");
        }
        if self.italic {
            rpr.push_str("<w:i/><w:iCs/>");
        }
        if let Some(color) = &self.color {
            rpr.push_str(&format!("<w:color w:val=\"{color}\"/>"));
        }
        if let Some(size) = self.size {
            let half_points = size * 2;
            rpr.push_str(&format!(
                "<w:sz w:val=\"{half_points}\"/><w:szCs w:val=\"{half_points}\"/>"
            ));
        }
        if self.underline {
            rpr.push_str("<w:u w:val=\"single\"/>");
        }
        if rpr.is_empty() {
            rpr
        } else {
            format!("<w:rPr>{rpr}</w:rPr>")
        }
    }
}

fn parse_alignment(a: &str) -> Option<&'static str> {
    match a {
        "left" => Some("left"),
        "center" => Some("center"),
        "right" => Some("right"),
        "justified" => Some("both"),
        _ => None,
    }
}

/// Paragraph properties in schema order (pStyle ... jc).
fn paragraph_properties(style_id: Option<&str>, style: Option<&DocxStyle>) -> String {
    let mut ppr = String::new();
    if let Some(id) = style_id {
        ppr.push_str(&format!("<w:pStyle w:val=\"{}\"/>", escape_xml(id)));
    }
    if let Some(jc) = style.and_then(|s| s.alignment) {
        ppr.push_str(&format!("<w:jc w:val=\"{jc}\"/>"));
    }
    if ppr.is_empty() {
        ppr
    } else {
        format!("<w:pPr>{ppr}</w:pPr>")
    }
}

fn text_paragraph(ppr: &str, rpr: &str, text: &str) -> String {
    format!(
        "<w:p>{ppr}<w:r>{rpr}<w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        escape_xml(text)
    )
}

// ---------------------------------------------------------------------------
// Update modes
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum UpdateMode {
    Append,
    Replace {
        old_text: String,
    },
    InsertStructured {
        level: Option<String>,
        style: Option<DocxStyle>,
    },
    AddImage {
        image_path: String,
        width: Option<u32>,
        height: Option<u32>,
    },
}

fn parse_update_mode(
    params: Option<&serde_json::Value>,
) -> Result<(UpdateMode, Option<DocxStyle>), ErrorData> {
    let Some(params) = params else {
        return Ok((UpdateMode::Append, None));
    };

    let mode_str = params
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("append");
    let style = params.get("style").and_then(DocxStyle::from_json);

    let mode = match mode_str {
        "append" => UpdateMode::Append,
        "replace" => {
            let old_text = params
                .get("old_text")
                .and_then(|v| v.as_str())
                .filter(|t| !t.is_empty())
                .ok_or_else(|| invalid_params("old_text parameter required for replace mode"))?;
            UpdateMode::Replace {
                old_text: old_text.to_string(),
            }
        }
        "structured" => UpdateMode::InsertStructured {
            level: params
                .get("level")
                .and_then(|v| v.as_str())
                .map(String::from),
            style: style.clone(),
        },
        "add_image" => {
            let image_path = params
                .get("image_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    invalid_params("image_path parameter required for add_image mode")
                })?;
            UpdateMode::AddImage {
                image_path: image_path.to_string(),
                width: params
                    .get("width")
                    .and_then(|v| v.as_u64())
                    .map(|w| w as u32),
                height: params
                    .get("height")
                    .and_then(|v| v.as_u64())
                    .map(|h| h as u32),
            }
        }
        _ => {
            return Err(invalid_params(
                "Invalid mode. Must be 'append', 'replace', 'structured', or 'add_image'",
            ))
        }
    };
    Ok((mode, style))
}

// ---------------------------------------------------------------------------
// Package (zip) handling
// ---------------------------------------------------------------------------

struct PackageEntry {
    name: String,
    data: Vec<u8>,
    method: CompressionMethod,
}

struct DocxPackage {
    entries: Vec<PackageEntry>,
}

impl DocxPackage {
    fn from_bytes(bytes: &[u8], path: &str) -> Result<Self, ErrorData> {
        if !bytes.starts_with(b"PK") {
            return Err(not_a_docx_error(path, bytes));
        }
        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|e| docx_error(format!("Failed to open {path} as a .docx package: {e}")))?;
        let mut entries = Vec::with_capacity(archive.len());
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| docx_error(format!("Failed to read {path}: {e}")))?;
            if file.is_dir() {
                continue;
            }
            let method = match file.compression() {
                CompressionMethod::Stored => CompressionMethod::Stored,
                _ => CompressionMethod::Deflated,
            };
            let name = file.name().to_string();
            let mut data = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut data)
                .map_err(|e| docx_error(format!("Failed to read {name} in {path}: {e}")))?;
            entries.push(PackageEntry { name, data, method });
        }
        let package = Self { entries };
        if package.get("[Content_Types].xml").is_none() {
            return Err(docx_error(format!(
                "{path} is a zip file but not a Word document (no [Content_Types].xml)."
            )));
        }
        Ok(package)
    }

    fn open(path: &str) -> Result<Self, ErrorData> {
        let bytes =
            fs::read(path).map_err(|e| docx_error(format!("Failed to read DOCX file: {e}")))?;
        Self::from_bytes(&bytes, path)
    }

    fn blank() -> Result<Self, ErrorData> {
        let mut buf = Vec::new();
        docx_rs::Docx::new()
            .build()
            .pack(&mut Cursor::new(&mut buf))
            .map_err(|e| docx_error(format!("Failed to create a new DOCX: {e}")))?;
        Self::from_bytes(&buf, "new document")
    }

    fn open_or_blank(path: &str) -> Result<Self, ErrorData> {
        if Path::new(path).exists() {
            Self::open(path)
        } else {
            Self::blank()
        }
    }

    fn get(&self, name: &str) -> Option<&PackageEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    fn read_xml(&self, name: &str) -> Result<String, ErrorData> {
        let entry = self
            .get(name)
            .ok_or_else(|| docx_error(format!("The document has no {name} part")))?;
        String::from_utf8(entry.data.clone())
            .map_err(|_| docx_error(format!("{name} is not valid UTF-8 XML")))
    }

    fn set(&mut self, name: &str, data: Vec<u8>) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.name == name) {
            entry.data = data;
        } else {
            self.entries.push(PackageEntry {
                name: name.to_string(),
                data,
                method: CompressionMethod::Deflated,
            });
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, ErrorData> {
        let mut buf = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            for entry in &self.entries {
                let options = SimpleFileOptions::default().compression_method(entry.method);
                writer
                    .start_file(entry.name.as_str(), options)
                    .map_err(|e| docx_error(format!("Failed to build DOCX: {e}")))?;
                writer
                    .write_all(&entry.data)
                    .map_err(|e| docx_error(format!("Failed to build DOCX: {e}")))?;
            }
            writer
                .finish()
                .map_err(|e| docx_error(format!("Failed to build DOCX: {e}")))?;
        }
        Ok(buf)
    }

    /// Write to a temporary file next to the target and rename it into place,
    /// so a failure never leaves a half-written document behind.
    fn save(&self, path: &str) -> Result<(), ErrorData> {
        let bytes = self.to_bytes()?;
        let target = Path::new(path);
        let file_name = target
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document.docx");
        let tmp = target.with_file_name(format!(".{file_name}.elroi-tmp"));
        fs::write(&tmp, &bytes)
            .map_err(|e| docx_error(format!("Failed to write DOCX file: {e}")))?;
        fs::rename(&tmp, target).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            docx_error(format!("Failed to write DOCX file: {e}"))
        })
    }

    /// The main document part, from [Content_Types].xml (usually word/document.xml).
    fn main_document_part(&self) -> String {
        if let Ok(types) = self.read_xml("[Content_Types].xml") {
            for tag in TagIter::new(&types) {
                if tag.name == "Override" {
                    let raw = &types[tag.start..tag.end];
                    let is_main = attr_value(raw, "ContentType")
                        .is_some_and(|ct| ct.contains("word") && ct.ends_with(".main+xml"));
                    if is_main {
                        if let Some(part) = attr_value(raw, "PartName") {
                            return part.trim_start_matches('/').to_string();
                        }
                    }
                }
            }
        }
        "word/document.xml".to_string()
    }
}

fn rels_part_for(part: &str) -> String {
    match part.rsplit_once('/') {
        Some((dir, file)) => format!("{dir}/_rels/{file}.rels"),
        None => format!("_rels/{part}.rels"),
    }
}

fn part_dir(part: &str) -> &str {
    part.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("")
}

fn join_part(dir: &str, target: &str) -> String {
    if let Some(stripped) = target.strip_prefix('/') {
        stripped.to_string()
    } else if dir.is_empty() {
        target.to_string()
    } else {
        format!("{dir}/{target}")
    }
}

// ---------------------------------------------------------------------------
// Minimal XML scanning
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Tag<'a> {
    start: usize,
    /// Byte offset just past the closing `>`.
    end: usize,
    name: &'a str,
    closing: bool,
    self_closing: bool,
}

struct TagIter<'a> {
    xml: &'a str,
    pos: usize,
}

impl<'a> TagIter<'a> {
    fn new(xml: &'a str) -> Self {
        Self { xml, pos: 0 }
    }

    fn starting_at(xml: &'a str, pos: usize) -> Self {
        Self { xml, pos }
    }
}

impl<'a> Iterator for TagIter<'a> {
    type Item = Tag<'a>;

    fn next(&mut self) -> Option<Tag<'a>> {
        loop {
            let start = self.pos + self.xml.get(self.pos..)?.find('<')?;
            let rest = &self.xml[start..];
            if rest.starts_with("<!--") {
                self.pos = start + rest.find("-->")? + 3;
                continue;
            }
            if rest.starts_with("<![CDATA[") {
                self.pos = start + rest.find("]]>")? + 3;
                continue;
            }
            let end = start + rest.find('>')? + 1;
            self.pos = end;
            if rest.starts_with("<?") || rest.starts_with("<!") {
                continue;
            }
            let inner = &self.xml[start + 1..end - 1];
            let closing = inner.starts_with('/');
            let self_closing = inner.ends_with('/');
            let body = inner.trim_start_matches('/');
            let name_len = body
                .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
                .unwrap_or(body.len());
            return Some(Tag {
                start,
                end,
                name: &body[..name_len],
                closing,
                self_closing,
            });
        }
    }
}

fn attr_value(raw_tag: &str, name: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(found) = raw_tag[search_from..].find(name) {
        let idx = search_from + found;
        search_from = idx + name.len();
        let preceded_ok = raw_tag[..idx]
            .chars()
            .last()
            .is_some_and(|c| c.is_whitespace());
        let after = raw_tag[idx + name.len()..].trim_start();
        if !preceded_ok || !after.starts_with('=') {
            continue;
        }
        let after = after[1..].trim_start();
        let quote = after.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let value = &after[1..];
        let close = value.find(quote)?;
        return Some(unescape_xml(&value[..close]));
    }
    None
}

/// Byte range of the element starting at `open` (a non-self-closing tag),
/// through its matching close tag.
fn element_end(xml: &str, open: &Tag) -> Option<usize> {
    if open.self_closing {
        return Some(open.end);
    }
    let mut depth = 0usize;
    for tag in TagIter::starting_at(xml, open.end) {
        if tag.name != open.name {
            continue;
        }
        if tag.closing {
            if depth == 0 {
                return Some(tag.end);
            }
            depth -= 1;
        } else if !tag.self_closing {
            depth += 1;
        }
    }
    None
}

/// Remove every element with one of `names` from an XML fragment.
fn remove_elements(fragment: &str, names: &[&str]) -> String {
    let mut out = fragment.to_string();
    loop {
        let found = TagIter::new(&out)
            .find(|t| !t.closing && names.contains(&t.name))
            .and_then(|t| element_end(&out, &t).map(|end| (t.start, end)));
        match found {
            Some((start, end)) => out.replace_range(start..end, ""),
            None => return out,
        }
    }
}

/// The first `<name>...</name>` element inside `xml[from..to]`.
fn find_element(xml: &str, name: &str, from: usize, to: usize) -> Option<(usize, usize)> {
    TagIter::starting_at(xml, from)
        .take_while(|t| t.start < to)
        .find(|t| t.name == name && !t.closing)
        .and_then(|t| element_end(xml, &t).map(|end| (t.start, end)))
}

fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

fn unescape_xml(text: &str) -> String {
    if !text.contains('&') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let Some(semi) = rest.find(';') else {
            break;
        };
        let entity = &rest[1..semi];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .or_else(|| entity.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &rest[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

// ---------------------------------------------------------------------------
// Paragraph model over the raw document XML
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TextNode {
    /// Start of `<w:t ...>`.
    tag_start: usize,
    /// End of `</w:t>`.
    close_end: usize,
    text: String,
}

#[derive(Debug)]
struct Paragraph {
    start: usize,
    end: usize,
    style: Option<String>,
    nodes: Vec<TextNode>,
}

impl Paragraph {
    fn text(&self) -> String {
        self.nodes.iter().map(|n| n.text.as_str()).collect()
    }
}

fn parse_paragraphs(xml: &str) -> Vec<Paragraph> {
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let mut open: Vec<usize> = Vec::new();
    let mut iter = TagIter::new(xml);
    while let Some(tag) = iter.next() {
        match tag.name {
            "w:p" if tag.self_closing => paragraphs.push(Paragraph {
                start: tag.start,
                end: tag.end,
                style: None,
                nodes: Vec::new(),
            }),
            "w:p" if tag.closing => {
                if let Some(idx) = open.pop() {
                    paragraphs[idx].end = tag.end;
                }
            }
            "w:p" => {
                open.push(paragraphs.len());
                paragraphs.push(Paragraph {
                    start: tag.start,
                    end: xml.len(),
                    style: None,
                    nodes: Vec::new(),
                });
            }
            "w:pStyle" if !tag.closing => {
                if let Some(&idx) = open.last() {
                    if paragraphs[idx].style.is_none() {
                        paragraphs[idx].style = attr_value(&xml[tag.start..tag.end], "w:val");
                    }
                }
            }
            "w:t" if !tag.closing && !tag.self_closing => {
                let Some(close_rel) = xml[tag.end..].find("</w:t>") else {
                    break;
                };
                let content_end = tag.end + close_rel;
                let close_end = content_end + "</w:t>".len();
                if let Some(&idx) = open.last() {
                    paragraphs[idx].nodes.push(TextNode {
                        tag_start: tag.start,
                        close_end,
                        text: unescape_xml(&xml[tag.end..content_end]),
                    });
                }
                iter = TagIter::starting_at(xml, close_end);
            }
            _ => {}
        }
    }
    paragraphs
}

/// Map typographic characters to plain ones, one char to one char, so that a
/// model typing straight quotes or hyphens still matches Word's smart quotes
/// and dashes without shifting any offsets.
fn normalize_char(c: char) -> char {
    match c {
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' | '\u{2032}' => '\'',
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{2033}' => '"',
        '\u{00A0}' | '\u{2007}' | '\u{202F}' => ' ',
        '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2212}' => '-',
        c => c,
    }
}

fn find_all(haystack: &[char], needle: &[char]) -> Vec<usize> {
    let mut found = Vec::new();
    if needle.is_empty() || needle.len() > haystack.len() {
        return found;
    }
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        if haystack[i..i + needle.len()] == *needle {
            found.push(i);
            i += needle.len();
        } else {
            i += 1;
        }
    }
    found
}

struct Edit {
    start: usize,
    end: usize,
    replacement: String,
}

fn apply_edits(xml: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by(|a, b| b.start.cmp(&a.start).then(b.end.cmp(&a.end)));
    let mut out = xml.to_string();
    for edit in edits {
        out.replace_range(edit.start..edit.end, &edit.replacement);
    }
    out
}

/// Rewrite the text nodes of `para` so that chars `[s, e)` become `new_text`.
/// The new text goes into the node where the match starts, so it keeps that
/// run's formatting; covered text in later nodes is removed.
fn range_edits(para: &Paragraph, s: usize, e: usize, new_text: &str) -> Vec<Edit> {
    let mut edits = Vec::new();
    let mut node_start = 0;
    let mut first = true;
    for node in &para.nodes {
        let chars: Vec<char> = node.text.chars().collect();
        let node_end = node_start + chars.len();
        if node_start < e && node_end > s {
            let mut text = String::new();
            if first {
                text.extend(&chars[..s - node_start]);
                text.push_str(new_text);
            }
            if e < node_end {
                text.extend(&chars[e - node_start..]);
            }
            edits.push(Edit {
                start: node.tag_start,
                end: node.close_end,
                replacement: format!("<w:t xml:space=\"preserve\">{}</w:t>", escape_xml(&text)),
            });
            first = false;
        }
        node_start = node_end;
    }
    edits
}

/// The `<w:rPr>` of the run containing the text node where `char_pos` falls.
fn run_properties_at(xml: &str, para: &Paragraph, char_pos: usize) -> String {
    let mut node_start = 0;
    let mut chosen = None;
    for node in &para.nodes {
        let len = node.text.chars().count();
        if len > 0 && chosen.is_none() {
            chosen = Some(node.tag_start);
        }
        if char_pos >= node_start && char_pos < node_start + len {
            chosen = Some(node.tag_start);
            break;
        }
        node_start += len;
    }
    let Some(tag_start) = chosen else {
        return String::new();
    };
    let run_open = TagIter::starting_at(xml, para.start)
        .take_while(|t| t.start < tag_start)
        .filter(|t| t.name == "w:r" && !t.closing && !t.self_closing)
        .last();
    let Some(run_open) = run_open else {
        return String::new();
    };
    match find_element(xml, "w:rPr", run_open.end, tag_start) {
        Some((start, end)) => remove_elements(&xml[start..end], &["w:rPrChange", "w:ins", "w:del"]),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Operations
// ---------------------------------------------------------------------------

fn do_extract_text(path: &str) -> Result<Vec<ContentBlock>, ErrorData> {
    let package = DocxPackage::open(path)?;
    let xml = package.read_xml(&package.main_document_part())?;
    let paragraphs = parse_paragraphs(&xml);

    let mut structure = Vec::new();
    let mut text = String::new();
    for para in &paragraphs {
        let para_text = para.text();
        if para_text.trim().is_empty() {
            continue;
        }
        if let Some(style) = &para.style {
            if style.starts_with("Heading") || style == "Title" {
                structure.push(format!("{style}: {para_text}"));
            }
        }
        text.push_str(&para_text);
        text.push('\n');
    }

    let result = if structure.is_empty() {
        format!("Extracted Text:\n{text}")
    } else {
        format!(
            "Document Structure:\n{}\n\nFull Text:\n{}",
            structure.join("\n"),
            text
        )
    };
    Ok(vec![ContentBlock::text(result)])
}

fn do_replace(path: &str, content: &str, old_text: &str) -> Result<Vec<ContentBlock>, ErrorData> {
    if !Path::new(path).exists() {
        return Err(docx_error(format!(
            "{path} does not exist, so there is nothing to replace. Use append mode to create it."
        )));
    }
    let mut package = DocxPackage::open(path)?;
    let part = package.main_document_part();
    let xml = package.read_xml(&part)?;
    let paragraphs = parse_paragraphs(&xml);

    let needle: Vec<char> = old_text.chars().collect();
    let texts: Vec<Vec<char>> = paragraphs
        .iter()
        .map(|p| p.text().chars().collect())
        .collect();

    let search = |normalize: bool| -> Vec<(usize, usize)> {
        let needle: Vec<char> = if normalize {
            needle.iter().map(|c| normalize_char(*c)).collect()
        } else {
            needle.clone()
        };
        texts
            .iter()
            .enumerate()
            .flat_map(|(idx, text)| {
                let hay: Vec<char> = if normalize {
                    text.iter().map(|c| normalize_char(*c)).collect()
                } else {
                    text.clone()
                };
                find_all(&hay, &needle)
                    .into_iter()
                    .map(move |pos| (idx, pos))
            })
            .collect()
    };

    let mut normalized = false;
    let mut matches = search(false);
    if matches.is_empty() {
        matches = search(true);
        normalized = !matches.is_empty();
    }

    let (para_idx, s) = match matches.as_slice() {
        [single] => *single,
        [] if old_text.contains('\n') => {
            return Err(docx_error(format!(
                "Could not find the text to replace in {path}. old_text spans more than one \
                 paragraph; replace one paragraph at a time, copying its exact wording from \
                 extract_text."
            )))
        }
        [] => {
            return Err(docx_error(format!(
                "Could not find the text to replace in {path}: {old_text}\n\
                 Run extract_text and copy the exact wording from a single paragraph."
            )))
        }
        many => {
            return Err(docx_error(format!(
                "The text to replace appears {} times in {path}. Include more of the \
                 surrounding words in old_text so it matches exactly one place.",
                many.len()
            )))
        }
    };

    let para = &paragraphs[para_idx];
    let e = s + needle.len();
    let lines: Vec<&str> = content
        .split('\n')
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.trim().is_empty())
        .collect();

    let mut edits;
    if lines.len() <= 1 {
        edits = range_edits(para, s, e, lines.first().copied().unwrap_or(""));
    } else {
        // The first line replaces the match; the rest become new paragraphs
        // after it, using the same paragraph and run formatting. Any text that
        // followed the match moves to the end of the last new paragraph.
        let para_len = texts[para_idx].len();
        let suffix: String = texts[para_idx][e..].iter().collect();
        edits = range_edits(para, s, para_len, lines[0]);

        let rpr = run_properties_at(&xml, para, s);
        let (ppr, section) = match find_element(&xml, "w:pPr", para.start, para.end) {
            Some((start, end)) => {
                let ppr_xml = &xml[start..end];
                let section = find_element(ppr_xml, "w:sectPr", 0, ppr_xml.len())
                    .map(|(ss, se)| (start + ss, start + se));
                let clean =
                    remove_elements(ppr_xml, &["w:sectPr", "w:pPrChange", "w:ins", "w:del"]);
                (clean, section)
            }
            None => (String::new(), None),
        };

        let mut new_paragraphs = String::new();
        let last = lines.len() - 1;
        for (i, line) in lines.iter().enumerate().skip(1) {
            let mut text = line.to_string();
            let mut this_ppr = ppr.clone();
            if i == last {
                text.push_str(&suffix);
                // A section break belongs to the last paragraph of its section.
                if let Some((ss, se)) = section {
                    let sect = &xml[ss..se];
                    this_ppr = if this_ppr.is_empty() {
                        format!("<w:pPr>{sect}</w:pPr>")
                    } else {
                        this_ppr.replacen("</w:pPr>", &format!("{sect}</w:pPr>"), 1)
                    };
                }
            }
            new_paragraphs.push_str(&text_paragraph(&this_ppr, &rpr, &text));
        }
        if let Some((ss, se)) = section {
            edits.push(Edit {
                start: ss,
                end: se,
                replacement: String::new(),
            });
        }
        edits.push(Edit {
            start: para.end,
            end: para.end,
            replacement: new_paragraphs,
        });
    }

    let new_xml = apply_edits(&xml, edits);
    package.set(&part, new_xml.into_bytes());
    package.save(path)?;

    let mut message = format!(
        "Replaced the text in paragraph {} of {path}. Everything else in the document, \
         including formatting, was left unchanged.",
        para_idx + 1
    );
    if normalized {
        message.push_str(" (Matched after treating curly quotes and dashes as plain ones.)");
    }
    Ok(vec![ContentBlock::text(message)])
}

/// Where new body content goes: before the final body-level `<w:sectPr>`, or
/// before `</w:body>`.
fn body_insert_position(xml: &str) -> Result<usize, ErrorData> {
    let body_close = TagIter::new(xml)
        .filter(|t| t.name == "w:body" && t.closing)
        .last()
        .ok_or_else(|| docx_error("The document has no <w:body> element"))?;
    let mut last_block_end = 0;
    let mut sect_start = None;
    let mut depth_p = 0usize;
    for tag in TagIter::new(xml).take_while(|t| t.start < body_close.start) {
        match tag.name {
            "w:p" | "w:tbl" | "w:sdt" if tag.closing => {
                depth_p = depth_p.saturating_sub(1);
                last_block_end = tag.end;
            }
            "w:p" | "w:tbl" | "w:sdt" if !tag.self_closing => depth_p += 1,
            "w:p" if tag.self_closing => last_block_end = tag.end,
            "w:sectPr" if !tag.closing && depth_p == 0 => sect_start = Some(tag.start),
            _ => {}
        }
    }
    Ok(match sect_start {
        Some(pos) if pos >= last_block_end => pos,
        _ => body_close.start,
    })
}

fn insert_body_xml(package: &mut DocxPackage, new_xml: &str) -> Result<(), ErrorData> {
    let part = package.main_document_part();
    let mut xml = package.read_xml(&part)?;
    let pos = body_insert_position(&xml)?;
    xml.insert_str(pos, new_xml);
    package.set(&part, xml.into_bytes());
    Ok(())
}

fn lines_as_paragraphs(content: &str, style_id: Option<&str>, style: Option<&DocxStyle>) -> String {
    let ppr = paragraph_properties(style_id, style);
    let rpr = style.map(|s| s.run_properties()).unwrap_or_default();
    content
        .split('\n')
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.trim().is_empty())
        .map(|l| text_paragraph(&ppr, &rpr, l))
        .collect()
}

fn do_append(
    path: &str,
    content: &str,
    style: &Option<DocxStyle>,
) -> Result<Vec<ContentBlock>, ErrorData> {
    let mut package = DocxPackage::open_or_blank(path)?;
    insert_body_xml(
        &mut package,
        &lines_as_paragraphs(content, None, style.as_ref()),
    )?;
    package.save(path)?;
    Ok(vec![ContentBlock::text(format!(
        "Successfully wrote content to {path}"
    ))])
}

fn styles_part(package: &DocxPackage) -> String {
    let main = package.main_document_part();
    if let Ok(rels) = package.read_xml(&rels_part_for(&main)) {
        for tag in TagIter::new(&rels) {
            let raw = &rels[tag.start..tag.end];
            if tag.name == "Relationship"
                && attr_value(raw, "Type").is_some_and(|t| t.ends_with("/styles"))
            {
                if let Some(target) = attr_value(raw, "Target") {
                    return join_part(part_dir(&main), &target);
                }
            }
        }
    }
    "word/styles.xml".to_string()
}

fn style_key(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Resolve a requested paragraph style ("Heading1", "Heading 1", "heading 1")
/// to a style id in the document, defining standard heading styles if the
/// document does not have them yet.
fn ensure_paragraph_style(package: &mut DocxPackage, level: &str) -> String {
    let part = styles_part(package);
    let Ok(styles) = package.read_xml(&part) else {
        return level.to_string();
    };
    let wanted = style_key(level);

    let mut current_id: Option<String> = None;
    for tag in TagIter::new(&styles) {
        let raw = &styles[tag.start..tag.end];
        if tag.name == "w:style" && !tag.closing {
            current_id = attr_value(raw, "w:styleId");
            if let Some(id) = &current_id {
                if style_key(id) == wanted {
                    return id.clone();
                }
            }
        } else if tag.name == "w:name" {
            if let (Some(id), Some(name)) = (&current_id, attr_value(raw, "w:val")) {
                if style_key(&name) == wanted {
                    return id.clone();
                }
            }
        }
    }

    let heading_level = wanted
        .strip_prefix("heading")
        .and_then(|n| n.parse::<u8>().ok())
        .filter(|n| (1..=9).contains(n));
    let definition = match (heading_level, wanted.as_str()) {
        (Some(n), _) => format!(
            "<w:style w:type=\"paragraph\" w:styleId=\"Heading{n}\"><w:name w:val=\"heading {n}\"/>\
             <w:uiPriority w:val=\"9\"/><w:qFormat/><w:pPr><w:keepNext/><w:keepLines/>\
             <w:outlineLvl w:val=\"{}\"/></w:pPr><w:rPr><w:b/><w:bCs/></w:rPr></w:style>",
            n - 1
        ),
        (None, "title") => {
            "<w:style w:type=\"paragraph\" w:styleId=\"Title\"><w:name w:val=\"Title\"/>\
             <w:uiPriority w:val=\"10\"/><w:qFormat/><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
             <w:rPr><w:b/><w:bCs/></w:rPr></w:style>"
                .to_string()
        }
        _ => return level.to_string(),
    };
    let id = match heading_level {
        Some(n) => format!("Heading{n}"),
        None => "Title".to_string(),
    };
    if let Some(pos) = styles.rfind("</w:styles>") {
        let mut updated = styles.clone();
        updated.insert_str(pos, &definition);
        package.set(&part, updated.into_bytes());
    }
    id
}

fn do_insert_structured(
    path: &str,
    content: &str,
    level: &Option<String>,
    style: &Option<DocxStyle>,
) -> Result<Vec<ContentBlock>, ErrorData> {
    let mut package = DocxPackage::open_or_blank(path)?;
    let style_id = level
        .as_deref()
        .map(|lvl| ensure_paragraph_style(&mut package, lvl));
    insert_body_xml(
        &mut package,
        &lines_as_paragraphs(content, style_id.as_deref(), style.as_ref()),
    )?;
    package.save(path)?;
    Ok(vec![ContentBlock::text(format!(
        "Successfully added structured content to {path}"
    ))])
}

fn load_image_as_png(image_path: &str) -> Result<(Vec<u8>, u32, u32), ErrorData> {
    let image_data =
        fs::read(image_path).map_err(|e| docx_error(format!("Failed to read image file: {e}")))?;
    let img = image::load_from_memory(&image_data)
        .map_err(|e| docx_error(format!("Failed to load image: {e}")))?;
    let (w, h) = img.dimensions();
    let is_png = Path::new(image_path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("png"));
    if is_png {
        return Ok((image_data, w, h));
    }
    let mut png_data = Vec::new();
    img.write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png)
        .map_err(|e| docx_error(format!("Failed to convert image to PNG: {e}")))?;
    Ok((png_data, w, h))
}

fn image_extent(native: (u32, u32), width: Option<u32>, height: Option<u32>) -> (u64, u64) {
    let (nw, nh) = (native.0.max(1) as u64, native.1.max(1) as u64);
    let (w, h) = match (width, height) {
        (Some(w), Some(h)) => (w as u64, h as u64),
        (Some(w), None) => (w as u64, (w as u64 * nh / nw).max(1)),
        (None, Some(h)) => ((h as u64 * nw / nh).max(1), h as u64),
        (None, None) => (nw, nh),
    };
    let (mut cx, mut cy) = (w * EMU_PER_PIXEL, h * EMU_PER_PIXEL);
    if cx > MAX_IMAGE_WIDTH_EMU {
        cy = cy * MAX_IMAGE_WIDTH_EMU / cx;
        cx = MAX_IMAGE_WIDTH_EMU;
    }
    (cx, cy.max(1))
}

fn do_add_image(
    path: &str,
    content: &str,
    image_path: &str,
    width: Option<u32>,
    height: Option<u32>,
    style: &Option<DocxStyle>,
) -> Result<Vec<ContentBlock>, ErrorData> {
    let (png, native_w, native_h) = load_image_as_png(image_path)?;
    let mut package = DocxPackage::open_or_blank(path)?;
    let main = package.main_document_part();
    let doc_dir = part_dir(&main).to_string();

    // Pick names that do not collide with anything already in the package.
    let mut n = 1;
    let media_name = loop {
        let candidate = format!("elroi_image{n}.png");
        if package
            .get(&join_part(&doc_dir, &format!("media/{candidate}")))
            .is_none()
        {
            break candidate;
        }
        n += 1;
    };
    package.set(&join_part(&doc_dir, &format!("media/{media_name}")), png);

    let rels_part = rels_part_for(&main);
    let rels = package.read_xml(&rels_part).unwrap_or_else(|_| {
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         </Relationships>"
            .to_string()
    });
    let mut rid_n = n;
    let rid = loop {
        let candidate = format!("rIdElroiImage{rid_n}");
        if !rels.contains(&format!("\"{candidate}\"")) {
            break candidate;
        }
        rid_n += 1;
    };
    let relationship = format!(
        "<Relationship Id=\"{rid}\" Type=\"{IMAGE_REL_TYPE}\" Target=\"media/{media_name}\"/>"
    );
    let pos = rels
        .rfind("</Relationships>")
        .ok_or_else(|| docx_error("The document's relationships part is malformed"))?;
    let mut rels = rels;
    rels.insert_str(pos, &relationship);
    package.set(&rels_part, rels.into_bytes());

    let mut types = package.read_xml("[Content_Types].xml")?;
    let has_png = TagIter::new(&types).any(|t| {
        t.name == "Default"
            && attr_value(&types[t.start..t.end], "Extension")
                .is_some_and(|e| e.eq_ignore_ascii_case("png"))
    });
    if !has_png {
        let open = TagIter::new(&types)
            .find(|t| t.name == "Types" && !t.closing)
            .ok_or_else(|| docx_error("[Content_Types].xml is malformed"))?;
        types.insert_str(
            open.end,
            "<Default Extension=\"png\" ContentType=\"image/png\"/>",
        );
        package.set("[Content_Types].xml", types.into_bytes());
    }

    let xml = package.read_xml(&main)?;
    let doc_pr_id = TagIter::new(&xml)
        .filter(|t| t.name.ends_with(":docPr") || t.name.ends_with(":cNvPr"))
        .filter_map(|t| attr_value(&xml[t.start..t.end], "id"))
        .filter_map(|id| id.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let (cx, cy) = image_extent((native_w, native_h), width, height);

    let drawing = format!(
        "<w:r><w:drawing>\
         <wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\" \
         xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\">\
         <wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/>\
         <wp:docPr id=\"{doc_pr_id}\" name=\"Picture {doc_pr_id}\"/>\
         <wp:cNvGraphicFramePr><a:graphicFrameLocks \
         xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" noChangeAspect=\"1\"/>\
         </wp:cNvGraphicFramePr>\
         <a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
         <a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\">\
         <pic:nvPicPr><pic:cNvPr id=\"0\" name=\"{media_name}\"/><pic:cNvPicPr/></pic:nvPicPr>\
         <pic:blipFill><a:blip r:embed=\"{rid}\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"/>\
         <a:stretch><a:fillRect/></a:stretch></pic:blipFill>\
         <pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm>\
         <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr>\
         </pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"
    );

    let mut new_xml = String::new();
    if !content.trim().is_empty() {
        new_xml.push_str(&lines_as_paragraphs(content, None, style.as_ref()));
    }
    let ppr = paragraph_properties(None, style.as_ref());
    new_xml.push_str(&format!("<w:p>{ppr}{drawing}</w:p>"));
    insert_body_xml(&mut package, &new_xml)?;
    package.save(path)?;
    Ok(vec![ContentBlock::text(format!(
        "Successfully added image to {path}"
    ))])
}

pub async fn docx_tool(
    path: &str,
    operation: &str,
    content: Option<&str>,
    params: Option<&serde_json::Value>,
) -> Result<Vec<ContentBlock>, ErrorData> {
    match operation {
        "extract_text" => do_extract_text(path),
        "update_doc" => {
            let content = content
                .ok_or_else(|| invalid_params("Content parameter required for update_doc"))?;
            let (mode, style) = parse_update_mode(params)?;

            match mode {
                UpdateMode::Append => do_append(path, content, &style),
                UpdateMode::Replace { old_text } => do_replace(path, content, &old_text),
                UpdateMode::InsertStructured {
                    level,
                    style: mode_style,
                } => do_insert_structured(path, content, &level, &mode_style.or(style)),
                UpdateMode::AddImage {
                    image_path,
                    width,
                    height,
                } => do_add_image(path, content, &image_path, width, height, &style),
            }
        }
        _ => Err(invalid_params(format!(
            "Invalid operation: {}. Valid operations are: 'extract_text', 'update_doc'",
            operation
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_docx_text_extraction() {
        let test_docx_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/sample.docx");

        println!("Testing text extraction from: {}", test_docx_path.display());

        let result = docx_tool(test_docx_path.to_str().unwrap(), "extract_text", None, None).await;

        assert!(result.is_ok(), "DOCX text extraction should succeed");
        let content = result.unwrap();
        assert!(!content.is_empty(), "Extracted text should not be empty");
        let text = content[0].as_text().unwrap();
        println!("Extracted text:\n{}", text.text);
        assert!(
            !text.text.trim().is_empty(),
            "Extracted text should not be empty"
        );
    }

    #[tokio::test]
    async fn test_docx_update_append() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_output.docx");

        let test_content =
            "Test Heading\nThis is a test paragraph.\n\nAnother paragraph with some content.";

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(test_content),
            None,
        )
        .await;

        assert!(result.is_ok(), "DOCX update should succeed");
        assert!(test_output_path.exists(), "Output file should exist");

        // Now try to read it back
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "extract_text",
            None,
            None,
        )
        .await;
        assert!(
            result.is_ok(),
            "Should be able to read back the written file"
        );
        let content = result.unwrap();
        let text = content[0].as_text().unwrap();
        assert!(
            text.text.contains("Test Heading"),
            "Should contain written content"
        );
        assert!(
            text.text.contains("test paragraph"),
            "Should contain written content"
        );

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_update_styled() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_styled.docx");

        let test_content = "Styled Heading\nThis is a styled paragraph.";
        let params = json!({
            "mode": "structured",
            "level": "Heading1",
            "style": {
                "bold": true,
                "color": "FF0000",
                "size": 24,
                "alignment": "center"
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(test_content),
            Some(&params),
        )
        .await;

        assert!(result.is_ok(), "DOCX styled update should succeed");
        assert!(test_output_path.exists(), "Output file should exist");

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_update_replace() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_replace.docx");

        // First create a document
        let initial_content = "Original content\nThis should be replaced.\nKeep this text.";
        let _ = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(initial_content),
            None,
        )
        .await;

        // Now replace part of it
        let replacement = "New content here";
        let params = json!({
            "mode": "replace",
            "old_text": "This should be replaced",
            "style": {
                "italic": true
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(replacement),
            Some(&params),
        )
        .await;

        assert!(result.is_ok(), "DOCX replace should succeed");

        // Verify the content
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "extract_text",
            None,
            None,
        )
        .await;
        assert!(result.is_ok());
        let content = result.unwrap();
        let text = content[0].as_text().unwrap();
        assert!(
            text.text.contains("New content here"),
            "Should contain new content"
        );
        assert!(
            text.text.contains("Keep this text"),
            "Should keep unmodified content"
        );
        assert!(
            !text.text.contains("This should be replaced"),
            "Should not contain replaced text"
        );

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_add_image() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_image.docx");

        // Create a test image file
        let test_image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_image.png");

        // Create a simple test PNG image using the image crate
        let imgbuf = image::ImageBuffer::from_fn(32, 32, |x, y| {
            let dx = x as f32 - 16.0;
            let dy = y as f32 - 16.0;
            if dx * dx + dy * dy < 16.0 * 16.0 {
                image::Rgb([0u8, 0u8, 255u8]) // Blue circle
            } else {
                image::Rgb([255u8, 255u8, 255u8]) // White background
            }
        });
        imgbuf
            .save(&test_image_path)
            .expect("Failed to create test image");

        let params = json!({
            "mode": "add_image",
            "image_path": test_image_path.to_str().unwrap(),
            "width": 100,
            "height": 100,
            "style": {
                "alignment": "center"
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some("Image Caption"),
            Some(&params),
        )
        .await;

        assert!(result.is_ok(), "DOCX image addition should succeed");
        assert!(test_output_path.exists(), "Output file should exist");

        // Clean up
        fs::remove_file(test_output_path).unwrap();
        fs::remove_file(test_image_path).unwrap();
    }

    #[tokio::test]
    async fn test_docx_invalid_path() {
        let result = docx_tool("nonexistent.docx", "extract_text", None, None).await;
        assert!(result.is_err(), "Should fail with invalid path");
    }

    #[tokio::test]
    async fn test_docx_invalid_operation() {
        let test_docx_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/sample.docx");

        let result = docx_tool(
            test_docx_path.to_str().unwrap(),
            "invalid_operation",
            None,
            None,
        )
        .await;

        assert!(result.is_err(), "Should fail with invalid operation");
    }

    #[tokio::test]
    async fn test_docx_update_without_content() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_output.docx");

        let result = docx_tool(test_output_path.to_str().unwrap(), "update_doc", None, None).await;

        assert!(result.is_err(), "Should fail without content");
    }

    #[tokio::test]
    async fn test_docx_update_preserve_content() {
        let test_output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/computercontroller/tests/data/test_preserve.docx");

        // First create a document with initial content
        let initial_content =
            "Initial content\nThis is the first paragraph.\nThis should stay in the document.";
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(initial_content),
            None,
        )
        .await;
        assert!(result.is_ok(), "Initial document creation should succeed");

        // Now append new content
        let new_content = "New content\nThis is an additional paragraph.";
        let params = json!({
            "mode": "append",
            "style": {
                "bold": true
            }
        });

        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "update_doc",
            Some(new_content),
            Some(&params),
        )
        .await;
        assert!(result.is_ok(), "Content append should succeed");

        // Verify both old and new content exists
        let result = docx_tool(
            test_output_path.to_str().unwrap(),
            "extract_text",
            None,
            None,
        )
        .await;
        assert!(result.is_ok());
        let content = result.unwrap();
        let text = content[0].as_text().unwrap();

        // Check for initial content
        assert!(
            text.text.contains("Initial content"),
            "Should contain initial content"
        );
        assert!(
            text.text.contains("first paragraph"),
            "Should contain first paragraph"
        );
        assert!(
            text.text.contains("should stay in the document"),
            "Should preserve existing content"
        );

        // Check for new content
        assert!(
            text.text.contains("New content"),
            "Should contain new content"
        );
        assert!(
            text.text.contains("additional paragraph"),
            "Should contain appended paragraph"
        );

        // Clean up
        fs::remove_file(test_output_path).unwrap();
    }
}

#[cfg(test)]
mod in_place_tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    fn fixture(dir: &TempDir, body: &str) -> String {
        let mut package = DocxPackage::blank().unwrap();
        insert_body_xml(&mut package, body).unwrap();
        package.set("customXml/item1.xml", b"<keep>me</keep>".to_vec());
        let path = dir.path().join("fixture.docx");
        package.save(path.to_str().unwrap()).unwrap();
        path.to_string_lossy().to_string()
    }

    fn document_xml(path: &str) -> String {
        let package = DocxPackage::open(path).unwrap();
        package.read_xml(&package.main_document_part()).unwrap()
    }

    fn paragraph_texts(path: &str) -> Vec<String> {
        parse_paragraphs(&document_xml(path))
            .iter()
            .map(|p| p.text())
            .filter(|t| !t.is_empty())
            .collect()
    }

    async fn replace(path: &str, old: &str, new: &str) -> Result<Vec<ContentBlock>, ErrorData> {
        let params = json!({ "mode": "replace", "old_text": old });
        docx_tool(path, "update_doc", Some(new), Some(&params)).await
    }

    const SPLIT_RUNS: &str = "<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Physical activity </w:t></w:r>\
        <w:r><w:rPr><w:i/></w:rPr><w:t xml:space=\"preserve\">predicts life satisfaction.</w:t></w:r></w:p>\
        <w:tbl><w:tr><w:tc><w:p><w:r><w:t>Table cell text</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";

    #[tokio::test]
    async fn replace_edits_only_the_matched_words() {
        let dir = TempDir::new().unwrap();
        let path = fixture(&dir, SPLIT_RUNS);

        replace(&path, "activity predicts", "exercise strongly predicts")
            .await
            .unwrap();

        let xml = document_xml(&path);
        assert_eq!(
            paragraph_texts(&path),
            vec![
                "Physical exercise strongly predicts life satisfaction.",
                "Table cell text"
            ]
        );
        // The new words live in the bold run where the match started, and the
        // italic run keeps its formatting and its remaining text.
        assert!(xml.contains(
            "<w:rPr><w:b/></w:rPr><w:t xml:space=\"preserve\">Physical exercise strongly predicts</w:t>"
        ));
        assert!(xml.contains(
            "<w:rPr><w:i/></w:rPr><w:t xml:space=\"preserve\"> life satisfaction.</w:t>"
        ));
        assert!(xml.contains("<w:tbl>"), "table must survive");

        let package = DocxPackage::open(&path).unwrap();
        assert_eq!(
            package.get("customXml/item1.xml").unwrap().data,
            b"<keep>me</keep>".to_vec(),
            "unrelated parts must be carried over untouched"
        );
        let bytes = fs::read(&path).unwrap();
        assert!(
            docx_rs::read_docx(&bytes).is_ok(),
            "result must still parse"
        );
    }

    #[tokio::test]
    async fn replace_matches_smart_quotes_and_dashes() {
        let dir = TempDir::new().unwrap();
        let path = fixture(
            &dir,
            "<w:p><w:r><w:t>Students\u{2019} \u{201C}life satisfaction\u{201D} \u{2014} measured</w:t></w:r></w:p>",
        );

        let result = replace(
            &path,
            "Students' \"life satisfaction\" - measured",
            "Wellbeing, measured",
        )
        .await
        .unwrap();

        assert_eq!(paragraph_texts(&path), vec!["Wellbeing, measured"]);
        assert!(result[0].as_text().unwrap().text.contains("curly quotes"));
    }

    #[tokio::test]
    async fn replace_refuses_ambiguous_text() {
        let dir = TempDir::new().unwrap();
        let path = fixture(
            &dir,
            "<w:p><w:r><w:t>The sample was small.</w:t></w:r></w:p>\
             <w:p><w:r><w:t>The sample was diverse.</w:t></w:r></w:p>",
        );
        let before = fs::read(&path).unwrap();

        let err = replace(&path, "The sample", "Our sample")
            .await
            .unwrap_err();

        assert!(err.message.contains("appears 2 times"));
        assert_eq!(fs::read(&path).unwrap(), before, "file must be untouched");
    }

    #[tokio::test]
    async fn replace_with_multiple_lines_adds_paragraphs() {
        let dir = TempDir::new().unwrap();
        let path = fixture(
            &dir,
            "<w:p><w:pPr><w:ind w:firstLine=\"720\"/></w:pPr><w:r><w:t>Alpha. Beta. Gamma.</w:t></w:r></w:p>",
        );

        replace(&path, "Beta.", "B one.\n\nB two.").await.unwrap();

        assert_eq!(
            paragraph_texts(&path),
            vec!["Alpha. B one.", "B two. Gamma."]
        );
        let xml = document_xml(&path);
        assert_eq!(
            xml.matches("<w:ind w:firstLine=\"720\"/>").count(),
            2,
            "new paragraph keeps the paragraph formatting"
        );
    }

    #[tokio::test]
    async fn plain_text_saved_as_docx_is_reported_not_rewritten() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("Topic5 DQ2.docx");
        fs::write(&path, "# Topic5 DQ2\n\n**Research Scenario:** ...").unwrap();
        let path = path.to_str().unwrap();

        let err = docx_tool(path, "extract_text", None, None)
            .await
            .unwrap_err();
        assert!(err.message.contains("plain text"));
        assert!(err.message.contains("pandoc"));

        let err = docx_tool(path, "update_doc", Some("more"), None)
            .await
            .unwrap_err();
        assert!(err.message.contains("plain text"));
        assert!(fs::read_to_string(path)
            .unwrap()
            .starts_with("# Topic5 DQ2"));
    }

    #[tokio::test]
    async fn append_goes_before_the_section_properties() {
        let dir = TempDir::new().unwrap();
        let path = fixture(&dir, SPLIT_RUNS);

        docx_tool(&path, "update_doc", Some("Appended line"), None)
            .await
            .unwrap();

        let xml = document_xml(&path);
        let appended = xml.find("Appended line").unwrap();
        let body_sect = xml.rfind("<w:sectPr").unwrap();
        assert!(appended < body_sect, "sectPr must stay last in the body");
        assert!(xml.contains("<w:tbl>"));
        assert!(docx_rs::read_docx(&fs::read(&path).unwrap()).is_ok());
    }

    #[tokio::test]
    async fn structured_heading_defines_the_style_once() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("headings.docx");
        let path = path.to_str().unwrap();
        let params = json!({ "mode": "structured", "level": "Heading 1" });

        for heading in ["Method", "Results"] {
            docx_tool(path, "update_doc", Some(heading), Some(&params))
                .await
                .unwrap();
        }

        let package = DocxPackage::open(path).unwrap();
        let styles = package.read_xml(&styles_part(&package)).unwrap();
        assert_eq!(styles.matches("w:styleId=\"Heading1\"").count(), 1);
        let xml = document_xml(path);
        assert_eq!(xml.matches("<w:pStyle w:val=\"Heading1\"/>").count(), 2);

        let result = docx_tool(path, "extract_text", None, None).await.unwrap();
        let text = &result[0].as_text().unwrap().text;
        assert!(text.contains("Heading1: Method"));
        assert!(text.contains("Heading1: Results"));
    }

    #[tokio::test]
    async fn add_image_registers_media_relationship_and_content_type() {
        let dir = TempDir::new().unwrap();
        let path = fixture(&dir, SPLIT_RUNS);
        let image_path = dir.path().join("figure.png");
        image::ImageBuffer::from_pixel(40, 20, image::Rgb([10u8, 20, 30]))
            .save(&image_path)
            .unwrap();
        let params = json!({ "mode": "add_image", "image_path": image_path.to_str().unwrap() });

        docx_tool(&path, "update_doc", Some("Figure 1"), Some(&params))
            .await
            .unwrap();

        let package = DocxPackage::open(&path).unwrap();
        assert!(package.get("word/media/elroi_image1.png").is_some());
        let rels = package.read_xml("word/_rels/document.xml.rels").unwrap();
        assert!(rels.contains("Target=\"media/elroi_image1.png\""));
        let types = package.read_xml("[Content_Types].xml").unwrap();
        assert!(types.contains("Extension=\"png\""));
        let xml = document_xml(&path);
        assert!(xml.contains("<wp:extent cx=\"381000\" cy=\"190500\"/>"));
        assert!(paragraph_texts(&path).contains(&"Figure 1".to_string()));
        assert!(docx_rs::read_docx(&fs::read(&path).unwrap()).is_ok());
    }

    #[test]
    fn unescape_round_trips() {
        let text = "A & B < C > D \"q\" it's";
        assert_eq!(unescape_xml(&escape_xml(text)), text);
        assert_eq!(
            unescape_xml("&#x2019;&#8217;&unknown;"),
            "\u{2019}\u{2019}&unknown;"
        );
    }
}
