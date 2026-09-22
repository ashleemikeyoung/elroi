/**
 * Theme tokens — the single source of truth for all MCP semantic token values.
 *
 * Every key in McpUiStyleVariableKey must be present in both lightTokens and
 * darkTokens. The TypeScript compiler enforces this: if the SDK adds a new key,
 * the build breaks until both maps are updated.
 *
 * Values are applied to :root via style.setProperty() before first paint
 * (see renderer.tsx). main.css only registers the variable names for Tailwind
 * class generation — it does NOT define values.
 *
 * These tokens serve two purposes:
 *  1. Goose desktop — applied to :root per resolved theme.
 *  2. MCP apps — encoded as light-dark() in hostContext.styles.variables.
 */
import type {
  McpUiHostStyles,
  McpUiStyleVariableKey,
  McpUiStyles,
} from '@modelcontextprotocol/ext-apps/app-bridge';

type ThemeTokens = Record<McpUiStyleVariableKey, string>;

// Subset of keys that are the same across both themes.
type BaseTokenKey = Extract<
  McpUiStyleVariableKey,
  `--font-${string}` | `--border-radius-${string}` | `--border-width-${string}`
>;

type ColorTokenKey = Exclude<McpUiStyleVariableKey, BaseTokenKey>;

// ---------------------------------------------------------------------------
// Base tokens — shared across light and dark themes
// ---------------------------------------------------------------------------
const baseTokens: Pick<ThemeTokens, BaseTokenKey> = {
  // Typography — families
  '--font-sans': "'Cash Sans', sans-serif",
  '--font-mono': 'monospace',

  // Typography — weights
  '--font-weight-normal': '400',
  '--font-weight-medium': '500',
  '--font-weight-semibold': '600',
  '--font-weight-bold': '700',

  // Typography — text sizes
  '--font-text-xs-size': '0.75rem',
  '--font-text-sm-size': '0.875rem',
  '--font-text-md-size': '1rem',
  '--font-text-lg-size': '1.125rem',

  // Typography — heading sizes
  '--font-heading-xs-size': '1rem',
  '--font-heading-sm-size': '1.125rem',
  '--font-heading-md-size': '1.25rem',
  '--font-heading-lg-size': '1.5rem',
  '--font-heading-xl-size': '1.875rem',
  '--font-heading-2xl-size': '2.25rem',
  '--font-heading-3xl-size': '3rem',

  // Typography — text line heights
  '--font-text-xs-line-height': '1rem',
  '--font-text-sm-line-height': '1.25rem',
  '--font-text-md-line-height': '1.5rem',
  '--font-text-lg-line-height': '1.75rem',

  // Typography — heading line heights
  '--font-heading-xs-line-height': '1.5rem',
  '--font-heading-sm-line-height': '1.75rem',
  '--font-heading-md-line-height': '1.75rem',
  '--font-heading-lg-line-height': '2rem',
  '--font-heading-xl-line-height': '2.25rem',
  '--font-heading-2xl-line-height': '2.5rem',
  '--font-heading-3xl-line-height': '3.5rem',

  // Border radius
  '--border-radius-xs': '2px',
  '--border-radius-sm': '4px',
  '--border-radius-md': '8px',
  '--border-radius-lg': '12px',
  '--border-radius-xl': '16px',
  '--border-radius-full': '9999px',

  // Border width
  '--border-width-regular': '1px',
};

// Theme-specific color/shadow tokens only.
type ColorTokens = Pick<ThemeTokens, ColorTokenKey>;

// ---------------------------------------------------------------------------
// Light theme — colors & shadows
// ---------------------------------------------------------------------------
const lightColorTokens: ColorTokens = {
  // Backgrounds
  '--color-background-primary': '#ffffff',
  '--color-background-secondary': '#f4f6f7',
  '--color-background-tertiary': '#e3e6ea',
  '--color-background-inverse': '#000000',
  '--color-background-ghost': 'transparent',
  '--color-background-info': '#5c98f9',
  '--color-background-danger': '#f94b4b',
  '--color-background-success': '#91cb80',
  '--color-background-warning': '#fbcd44',
  '--color-background-disabled': '#e3e6ea',

  // Text
  '--color-text-primary': '#3f434b',
  '--color-text-secondary': '#878787',
  '--color-text-tertiary': '#a7b0b9',
  '--color-text-inverse': '#ffffff',
  '--color-text-ghost': '#878787',
  '--color-text-info': '#5c98f9',
  '--color-text-danger': '#f94b4b',
  '--color-text-success': '#91cb80',
  '--color-text-warning': '#fbcd44',
  '--color-text-disabled': '#cbd1d6',

  // Borders
  '--color-border-primary': '#e3e6ea',
  '--color-border-secondary': '#e3e6ea',
  '--color-border-tertiary': '#cbd1d6',
  '--color-border-inverse': '#000000',
  '--color-border-ghost': 'transparent',
  '--color-border-info': '#5c98f9',
  '--color-border-danger': '#f94b4b',
  '--color-border-success': '#91cb80',
  '--color-border-warning': '#fbcd44',
  '--color-border-disabled': '#e3e6ea',

  // Rings
  '--color-ring-primary': '#e3e6ea',
  '--color-ring-secondary': '#cbd1d6',
  '--color-ring-inverse': '#ffffff',
  '--color-ring-info': '#5c98f9',
  '--color-ring-danger': '#f94b4b',
  '--color-ring-success': '#91cb80',
  '--color-ring-warning': '#fbcd44',

  // Shadows
  '--shadow-hairline': '0 0 0 1px rgba(0, 0, 0, 0.05)',
  '--shadow-sm': '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
  '--shadow-md': '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
  '--shadow-lg': '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)',
};

// ---------------------------------------------------------------------------
// Dark theme — colors & shadows
// ---------------------------------------------------------------------------
const darkColorTokens: ColorTokens = {
  // Backgrounds
  '--color-background-primary': '#22252a',
  '--color-background-secondary': '#3f434b',
  '--color-background-tertiary': '#474e57',
  '--color-background-inverse': '#cbd1d6',
  '--color-background-ghost': 'transparent',
  '--color-background-info': '#7cacff',
  '--color-background-danger': '#ff6b6b',
  '--color-background-success': '#a3d795',
  '--color-background-warning': '#ffd966',
  '--color-background-disabled': '#474e57',

  // Text
  '--color-text-primary': '#ffffff',
  '--color-text-secondary': '#878787',
  '--color-text-tertiary': '#606c7a',
  '--color-text-inverse': '#000000',
  '--color-text-ghost': '#878787',
  '--color-text-info': '#7cacff',
  '--color-text-danger': '#ff6b6b',
  '--color-text-success': '#a3d795',
  '--color-text-warning': '#ffd966',
  '--color-text-disabled': '#525b68',

  // Borders
  '--color-border-primary': '#3f434b',
  '--color-border-secondary': '#525b68',
  '--color-border-tertiary': '#474e57',
  '--color-border-inverse': '#ffffff',
  '--color-border-ghost': 'transparent',
  '--color-border-info': '#7cacff',
  '--color-border-danger': '#ff6b6b',
  '--color-border-success': '#a3d795',
  '--color-border-warning': '#ffd966',
  '--color-border-disabled': '#3f434b',

  // Rings
  '--color-ring-primary': '#525b68',
  '--color-ring-secondary': '#474e57',
  '--color-ring-inverse': '#000000',
  '--color-ring-info': '#7cacff',
  '--color-ring-danger': '#ff6b6b',
  '--color-ring-success': '#a3d795',
  '--color-ring-warning': '#ffd966',

  // Shadows (darker for dark mode)
  '--shadow-hairline': '0 0 0 1px rgba(0, 0, 0, 0.2)',
  '--shadow-sm': '0 1px 2px 0 rgba(0, 0, 0, 0.2)',
  '--shadow-md': '0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -2px rgba(0, 0, 0, 0.2)',
  '--shadow-lg': '0 10px 15px -3px rgba(0, 0, 0, 0.3), 0 4px 6px -4px rgba(0, 0, 0, 0.2)',
};

// ---------------------------------------------------------------------------
// Aura theme — colors & shadows (dark variant)
// Ported from OpenChamber's Aura preset: purple-black surfaces, purple accent,
// mint/peach/cyan/coral status colors, monospace typography.
// ---------------------------------------------------------------------------
const auraColorTokens: ColorTokens = {
  // Backgrounds
  '--color-background-primary': '#15141b',
  '--color-background-secondary': '#1a1921',
  '--color-background-tertiary': '#201e2b',
  '--color-background-inverse': '#a277ff',
  '--color-background-ghost': 'transparent',
  '--color-background-info': '#82e2ff',
  '--color-background-danger': '#ff6767',
  '--color-background-success': '#61ffca',
  '--color-background-warning': '#ffca85',
  '--color-background-disabled': '#25232f',

  // Text
  '--color-text-primary': '#edecee',
  '--color-text-secondary': '#8a8282',
  '--color-text-tertiary': '#6d6d6d',
  '--color-text-inverse': '#15141b',
  '--color-text-ghost': '#8a8282',
  '--color-text-info': '#82e2ff',
  '--color-text-danger': '#ff6767',
  '--color-text-success': '#61ffca',
  '--color-text-warning': '#ffca85',
  '--color-text-disabled': '#525b68',

  // Borders
  '--color-border-primary': '#2d2b38',
  '--color-border-secondary': '#47415a',
  '--color-border-tertiary': '#4e496c',
  '--color-border-inverse': '#edecee',
  '--color-border-ghost': 'transparent',
  '--color-border-info': '#82e2ff',
  '--color-border-danger': '#ff6767',
  '--color-border-success': '#61ffca',
  '--color-border-warning': '#ffca85',
  '--color-border-disabled': '#2d2b38',

  // Rings
  '--color-ring-primary': '#47415a',
  '--color-ring-secondary': '#2d2b38',
  '--color-ring-inverse': '#15141b',
  '--color-ring-info': '#82e2ff',
  '--color-ring-danger': '#ff6767',
  '--color-ring-success': '#61ffca',
  '--color-ring-warning': '#ffca85',

  // Shadows (dark)
  '--shadow-hairline': '0 0 0 1px rgba(0, 0, 0, 0.2)',
  '--shadow-sm': '0 1px 2px 0 rgba(0, 0, 0, 0.2)',
  '--shadow-md': '0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -2px rgba(0, 0, 0, 0.2)',
  '--shadow-lg': '0 10px 15px -3px rgba(0, 0, 0, 0.3), 0 4px 6px -4px rgba(0, 0, 0, 0.2)',
};

// ---------------------------------------------------------------------------
// ElRoi orchestrator themes — colors & shadows
// Ported from RAG/templates/index.html so desktop and orchestrator share the
// same proofreader palette family.
// ---------------------------------------------------------------------------
const pinkColorTokens: ColorTokens = {
  '--color-background-primary': '#faeef0',
  '--color-background-secondary': '#f4e2e6',
  '--color-background-tertiary': '#ecd4da',
  '--color-background-inverse': '#2e1e24',
  '--color-background-ghost': 'transparent',
  '--color-background-info': '#b3187f',
  '--color-background-danger': '#a8324f',
  '--color-background-success': '#4a7256',
  '--color-background-warning': '#b27f4a',
  '--color-background-disabled': '#ebd3d9',

  '--color-text-primary': '#2e1e24',
  '--color-text-secondary': '#6b4f58',
  '--color-text-tertiary': '#7c5e67',
  '--color-text-inverse': '#faeef0',
  '--color-text-ghost': '#7c5e67',
  '--color-text-info': '#b3187f',
  '--color-text-danger': '#a8324f',
  '--color-text-success': '#4a7256',
  '--color-text-warning': '#8b5f2d',
  '--color-text-disabled': '#9f8790',

  '--color-border-primary': '#e0c3cb',
  '--color-border-secondary': '#ebd3d9',
  '--color-border-tertiary': '#d3adb8',
  '--color-border-inverse': '#2e1e24',
  '--color-border-ghost': 'transparent',
  '--color-border-info': '#b3187f',
  '--color-border-danger': '#a8324f',
  '--color-border-success': '#4a7256',
  '--color-border-warning': '#b27f4a',
  '--color-border-disabled': '#ebd3d9',

  '--color-ring-primary': '#e0c3cb',
  '--color-ring-secondary': '#d3adb8',
  '--color-ring-inverse': '#faeef0',
  '--color-ring-info': '#b3187f',
  '--color-ring-danger': '#a8324f',
  '--color-ring-success': '#4a7256',
  '--color-ring-warning': '#b27f4a',

  '--shadow-hairline': '0 0 0 1px rgba(46, 30, 36, 0.08)',
  '--shadow-sm': '0 1px 2px 0 rgba(46, 30, 36, 0.08)',
  '--shadow-md': '0 4px 6px -1px rgba(46, 30, 36, 0.12), 0 2px 4px -2px rgba(46, 30, 36, 0.08)',
  '--shadow-lg': '0 10px 15px -3px rgba(46, 30, 36, 0.14), 0 4px 6px -4px rgba(46, 30, 36, 0.1)',
};

const grassColorTokens: ColorTokens = {
  '--color-background-primary': '#1e4620',
  '--color-background-secondary': '#275a29',
  '--color-background-tertiary': '#316d33',
  '--color-background-inverse': '#fff0a5',
  '--color-background-ghost': 'transparent',
  '--color-background-info': '#b64926',
  '--color-background-danger': '#8e2800',
  '--color-background-success': '#fff0a5',
  '--color-background-warning': '#d8cc86',
  '--color-background-disabled': '#3b673d',

  '--color-text-primary': '#fff0a5',
  '--color-text-secondary': '#eadf99',
  '--color-text-tertiary': '#d8cc86',
  '--color-text-inverse': '#1e4620',
  '--color-text-ghost': '#d8cc86',
  '--color-text-info': '#f0a17c',
  '--color-text-danger': '#ffb090',
  '--color-text-success': '#fff0a5',
  '--color-text-warning': '#eadf99',
  '--color-text-disabled': '#9f9a6a',

  '--color-border-primary': '#47794a',
  '--color-border-secondary': '#3b673d',
  '--color-border-tertiary': '#568a59',
  '--color-border-inverse': '#fff0a5',
  '--color-border-ghost': 'transparent',
  '--color-border-info': '#b64926',
  '--color-border-danger': '#8e2800',
  '--color-border-success': '#fff0a5',
  '--color-border-warning': '#d8cc86',
  '--color-border-disabled': '#3b673d',

  '--color-ring-primary': '#47794a',
  '--color-ring-secondary': '#568a59',
  '--color-ring-inverse': '#1e4620',
  '--color-ring-info': '#b64926',
  '--color-ring-danger': '#8e2800',
  '--color-ring-success': '#fff0a5',
  '--color-ring-warning': '#d8cc86',

  '--shadow-hairline': '0 0 0 1px rgba(0, 0, 0, 0.35)',
  '--shadow-sm': '0 1px 2px 0 rgba(0, 0, 0, 0.35)',
  '--shadow-md': '0 4px 6px -1px rgba(0, 0, 0, 0.4), 0 2px 4px -2px rgba(0, 0, 0, 0.3)',
  '--shadow-lg': '0 10px 15px -3px rgba(0, 0, 0, 0.42), 0 4px 6px -4px rgba(0, 0, 0, 0.32)',
};

const oceanColorTokens: ColorTokens = {
  '--color-background-primary': '#224fbc',
  '--color-background-secondary': '#1d45a6',
  '--color-background-tertiary': '#173b90',
  '--color-background-inverse': '#ffffff',
  '--color-background-ghost': 'transparent',
  '--color-background-info': '#ffffff',
  '--color-background-danger': '#ffd0c2',
  '--color-background-success': '#ffffff',
  '--color-background-warning': '#d5e1ff',
  '--color-background-disabled': '#4f72cf',

  '--color-text-primary': '#ffffff',
  '--color-text-secondary': '#eaf0ff',
  '--color-text-tertiary': '#d5e1ff',
  '--color-text-inverse': '#224fbc',
  '--color-text-ghost': '#d5e1ff',
  '--color-text-info': '#ffffff',
  '--color-text-danger': '#ffd0c2',
  '--color-text-success': '#ffffff',
  '--color-text-warning': '#d5e1ff',
  '--color-text-disabled': '#9fb7f0',

  '--color-border-primary': '#6f8ee0',
  '--color-border-secondary': '#4f72cf',
  '--color-border-tertiary': '#87a2eb',
  '--color-border-inverse': '#ffffff',
  '--color-border-ghost': 'transparent',
  '--color-border-info': '#ffffff',
  '--color-border-danger': '#ffd0c2',
  '--color-border-success': '#ffffff',
  '--color-border-warning': '#d5e1ff',
  '--color-border-disabled': '#4f72cf',

  '--color-ring-primary': '#6f8ee0',
  '--color-ring-secondary': '#87a2eb',
  '--color-ring-inverse': '#224fbc',
  '--color-ring-info': '#ffffff',
  '--color-ring-danger': '#ffd0c2',
  '--color-ring-success': '#ffffff',
  '--color-ring-warning': '#d5e1ff',

  '--shadow-hairline': '0 0 0 1px rgba(0, 0, 0, 0.32)',
  '--shadow-sm': '0 1px 2px 0 rgba(0, 0, 0, 0.32)',
  '--shadow-md': '0 4px 6px -1px rgba(0, 0, 0, 0.38), 0 2px 4px -2px rgba(0, 0, 0, 0.28)',
  '--shadow-lg': '0 10px 15px -3px rgba(0, 0, 0, 0.4), 0 4px 6px -4px rgba(0, 0, 0, 0.3)',
};

// Aura is monospace-first — override the shared sans family.
const auraFontTokens: Partial<Pick<ThemeTokens, BaseTokenKey>> = {
  '--font-sans': 'ui-monospace, "SFMono-Regular", "Menlo", "Cascadia Mono", "Segoe UI Mono", monospace',
  '--font-mono': 'ui-monospace, "SFMono-Regular", "Menlo", "Cascadia Mono", "Segoe UI Mono", monospace',
};

// ---------------------------------------------------------------------------
// Merged token maps — used by applyThemeTokens() and buildMcpHostStyles()
// ---------------------------------------------------------------------------
export const lightTokens: ThemeTokens = { ...baseTokens, ...lightColorTokens };
export const darkTokens: ThemeTokens = { ...baseTokens, ...darkColorTokens };
export const auraTokens: ThemeTokens = { ...baseTokens, ...auraFontTokens, ...auraColorTokens };
export const pinkTokens: ThemeTokens = { ...baseTokens, ...pinkColorTokens };
export const grassTokens: ThemeTokens = { ...baseTokens, ...grassColorTokens };
export const oceanTokens: ThemeTokens = { ...baseTokens, ...oceanColorTokens };

// ---------------------------------------------------------------------------
// Theme registry — the set of selectable named themes.
// `variant` drives the .dark/.light class and colorScheme for anything outside
// the token system; `tokens` is the map applied to :root. Adding a future theme
// is a single entry here plus its token map above.
// ---------------------------------------------------------------------------
export type ThemeId = 'light' | 'dark' | 'aura' | 'pink' | 'grass' | 'ocean';
export type ThemeVariant = 'light' | 'dark';

interface ThemeDefinition {
  variant: ThemeVariant;
  tokens: ThemeTokens;
}

export const themes: Record<ThemeId, ThemeDefinition> = {
  light: { variant: 'light', tokens: lightTokens },
  dark: { variant: 'dark', tokens: darkTokens },
  aura: { variant: 'dark', tokens: auraTokens },
  pink: { variant: 'light', tokens: pinkTokens },
  grass: { variant: 'dark', tokens: grassTokens },
  ocean: { variant: 'dark', tokens: oceanTokens },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// @font-face rules passed to MCP apps so sandboxed iframes can load host fonts.
const HOST_FONT_CSS = `
@font-face {
  font-family: 'Cash Sans';
  src: url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff2/CashSans-Light.woff2) format('woff2'),
       url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff/CashSans-Light.woff) format('woff');
  font-weight: 300;
  font-style: normal;
}
@font-face {
  font-family: 'Cash Sans';
  src: url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff2/CashSans-Regular.woff2) format('woff2'),
       url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff/CashSans-Regular.woff) format('woff');
  font-weight: 400;
  font-style: normal;
}
@font-face {
  font-family: 'Cash Sans';
  src: url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff2/CashSans-Medium.woff2) format('woff2'),
       url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff/CashSans-Medium.woff) format('woff');
  font-weight: 500;
  font-style: normal;
}
@font-face {
  font-family: 'Cash Sans';
  src: url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff2/CashSans-Bold.woff2) format('woff2'),
       url(https://cash-f.squarecdn.com/static/fonts/cashsans/woff/CashSans-Bold.woff) format('woff');
  font-weight: 700;
  font-style: normal;
}
`.trim();

/**
 * Build the McpUiHostStyles object for MCP apps.
 *
 * For the built-in light/dark pair, color keys use light-dark() so a single
 * payload resolves correctly against the guest's color-scheme. Custom themes
 * (e.g. aura) share a variant with dark but carry their own palette and fonts,
 * so light-dark() can't express them — emit that theme's concrete token values
 * instead. Non-color keys always use the theme's own values so overrides like
 * Aura's monospace font family reach the guest.
 * css.fonts provides @font-face rules so sandboxed apps can load host fonts.
 */
export function buildMcpHostStyles(themeId: ThemeId = 'light'): McpUiHostStyles {
  const tokens = (themes[themeId] ?? themes.light).tokens;
  const isBuiltinVariant = themeId === 'light' || themeId === 'dark';
  const variables: McpUiStyles = {} as McpUiStyles;
  for (const key of Object.keys(lightTokens) as McpUiStyleVariableKey[]) {
    if (key.startsWith('--color-') && isBuiltinVariant) {
      variables[key] = `light-dark(${lightTokens[key]}, ${darkTokens[key]})`;
    } else {
      variables[key] = tokens[key];
    }
  }
  return { variables, css: { fonts: HOST_FONT_CSS } };
}

/**
 * Resolve the current theme id from localStorage / system preference.
 * Best-effort pre-paint resolution; the authoritative preference lives in the
 * Electron settings store and is applied by ThemeContext once loaded.
 */
export function getResolvedTheme(): ThemeId {
  const useSystem = localStorage.getItem('use_system_theme') !== 'false';
  if (useSystem) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  const stored = localStorage.getItem('theme');
  if (stored === 'aura' || stored === 'pink' || stored === 'grass' || stored === 'ocean') {
    return stored;
  }
  return stored === 'dark' ? 'dark' : 'light';
}

/**
 * Apply a theme's tokens to the document root as CSS custom properties.
 * When called without an argument, resolves the theme from localStorage.
 */
export function applyThemeTokens(theme?: ThemeId): void {
  const resolved = theme ?? getResolvedTheme();
  const { tokens } = themes[resolved] ?? themes.light;
  const root = document.documentElement;
  for (const [key, value] of Object.entries(tokens)) {
    root.style.setProperty(key, value);
  }
}
