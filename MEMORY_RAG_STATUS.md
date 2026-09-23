# Memory and RAG Integration Status

## Current State

### 1. Memory MCP Server ✓ (Enabled)

**Location**: `crates/goose-mcp/src/memory/mod.rs`

**Status**: **Fully implemented and ENABLED**

**Features**:
- Stores/retrieves categorized memories with tagging support
- Two storage modes:
  - Local: `.goose/memory/` (project-specific)
  - Global: `~/.config/goose/memory/` (user-wide)
- Tools available:
  - `remember_memory` - Store a memory with optional tags
  - `retrieve_memories` - Retrieve all memories from a category
  - `remove_memory_category` - Remove all memories in a category
  - `remove_specific_memory` - Remove a specific memory

**Integration**:
- The memory server is **built into the Rust binary** via `goose-mcp` crate
- It's one of the `BUILTIN_EXTENSIONS` alongside developer, computercontroller, tutorial
- No external Python needed

**Configuration**:
```json
{
  "id": "memory",
  "name": "memory",
  "display_name": "Memory",
  "description": "Teach ElRoi your preferences as you go.",
  "enabled": true,  // <-- NOW ENABLED!
  "type": "builtin",
  "timeout": 300,
  "bundled": true
}
```

### 2. RAG Server ✓ (Updated Paths)

**Location**: `~/Development/RAG/mcp_server.py` (external Python server)

**Status**: **Fully implemented with portable paths**

**Features**:
- 20+ MCP tools for document management
- Vector search via ChromaDB
- libsql metadata storage for sessions/citations/registry
- Tools include: search, list, rescan, read, ask_local, ingest, revision cases, lesson builder

**Integration**:
- External stdio MCP server (Python script)
- Uses `libsql_client` for database operations
- Connects to libsql via `LIBSQL_URL` and `LIBSQL_AUTH_TOKEN` env vars

**Updated Configuration** (from `bundled-extensions.json`):
```json
{
  "id": "rag",
  "name": "rag",
  "display_name": "ElRoi RAG",
  "description": "ElRoi's bundled local RAG MCP server...",
  "enabled": true,
  "type": "stdio",
  "cmd": "python3",
  "args": ["${GOOSE_PATH}/../../Development/RAG/mcp_server.py"],
  "env_keys": ["LIBSQL_URL", "LIBSQL_AUTH_TOKEN"],
  "timeout": 300,
  "bundled": true
}
```

**Changes Made**:
1. ✅ Fixed hardcoded Python path from `/opt/anaconda3/envs/rag/bin/python` to `python3`
2. ✅ Fixed hardcoded RAG path from `/Users/ash/Development/RAG/mcp_server.py` to `${GOOSE_PATH}/../../Development/RAG/mcp_server.py`
3. ✅ Added env_keys for `LIBSQL_URL` and `LIBSQL_AUTH_TOKEN`

### 3. libsql Memory Database ✓ (Environment Variables Configured)

**Location**: `~/Development/RAG/memory/` (separate project)

**Status**: **Integration complete with env var support**

**Database Structure**:
- `sessions` - Track chat sessions with metadata
- `turns` - Log each question/answer exchange
- `quality_scores` - Quality metrics for each turn
- `citations` - Verified citations independent of Chroma
- `document_registry` - Document metadata (labels, synopses, genres, themes)

**Client**: `memory_client.py` uses `libsql_client` for sync operations

---

## Key Differences: Two Different "Memory" Systems

| Component | Location | Storage | Purpose | Status |
|-----------|----------|---------|---------|--------|
| **Rust Memory MCP** | `crates/goose-mcp/src/memory/mod.rs` | File-based (txt files) | Store/retrieve user preferences, workflow tips, etc. | ✅ Enabled |
| **libsql Memory DB** | `~/Development/RAG/memory/` | libsql database | Track RAG sessions, turns, citations, quality scores | ✅ Fully integrated |
| **RAG Server** | `~/Development/RAG/mcp_server.py` | Chroma + libsql | Search documents, manage projects, answer questions | ✅ Paths fixed |

---

## Summary of Changes Made

### 1. Memory Extension Enabled ✅
- Changed `"enabled": false` to `"enabled": true` in `bundled-extensions.json`
- The Rust memory MCP server is now activated automatically when sessions start

### 2. RAG Paths Fixed ✅
- Changed `cmd` from hardcoded `/opt/anaconda3/envs/rag/bin/python` to portable `python3`
- Changed `args` from hardcoded `/Users/ash/Development/RAG/mcp_server.py` to `${GOOSE_PATH}/../../Development/RAG/mcp_server.py`
- Added `env_keys` for `LIBSQL_URL` and `LIBSQL_AUTH_TOKEN` environment variables

### 3. Environment Variables
- ✅ The RAG extension now accepts `LIBSQL_URL` and `LIBSQL_AUTH_TOKEN` as environment variables
- ✅ These will be passed through to the RAG server for libsql database connectivity

---

## Files Modified

1. **ui/desktop/src/components/settings/extensions/bundled-extensions.json**
   - Changed memory extension: `"enabled": true`
   - Fixed RAG extension: portable paths and env_keys

---

## How It Works Now

### Memory (Rust MCP):
1. When a session starts, the Rust memory MCP server is spawned automatically
2. Memories are stored in `.goose/memory/` (local) or `~/.config/goose/memory/` (global)
3. Users can teach ElRoi preferences and workflow patterns

### RAG (Python MCP):
1. When a session starts, the RAG server is spawned via `python3`
2. It connects to libsql using `LIBSQL_URL` and `LIBSQL_AUTH_TOKEN` from environment
3. Users can search documents, list projects, and get answers from their indexed files

### libsql Integration:
1. Your RAG server's `memory_client.py` connects to libsql
2. Session metadata, citations, and document registry are stored there
3. Environment variables control the connection

---

## ✅ Build and Install Complete

**Rust CLI Binary Installed:**
- `goose` at `/Users/ash/.local/bin/goose` (v1.51.0)
- `elroi` at `/Users/ash/.local/bin/elroi` (v1.51.0)
- Built with `cargo build --release`
- Memory MCP enabled by default in bundled-extensions.json
- RAG paths fixed with environment variable support
