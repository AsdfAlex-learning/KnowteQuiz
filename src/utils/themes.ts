/**
 * VS Code theme presets.
 *
 * Each preset is a full palette of the app's CSS variables, not just a
 * background colour: switching presets must restyle sidebars, text, borders and
 * semantic colours together or the result looks half-themed.
 */

export type ThemeKind = 'dark' | 'light';

export interface ThemePalette {
  bgBase: string;
  bgSidebar: string;
  bgSurface: string;
  bgElevated: string;
  bgHover: string;
  bgActive: string;
  textPrimary: string;
  textSecondary: string;
  textMuted: string;
  textFaint: string;
  accent: string;
  accentBlue: string;
  accentGreen: string;
  accentYellow: string;
  accentPeach: string;
  accentRed: string;
  accentPink: string;
  accentCyan: string;
  accentOrange: string;
  accentMauve: string;
  accentLavender: string;
  success: string;
  warning: string;
  error: string;
  info: string;
  borderDefault: string;
  borderSubtle: string;
  borderFocus: string;
}

export interface ThemePreset {
  id: string;
  label: string;
  kind: ThemeKind;
  palette: ThemePalette;
}

const DARK_PLUS: ThemePalette = {
  bgBase: '#1e1e1e',
  bgSidebar: '#252526',
  bgSurface: '#1e1e1e',
  bgElevated: '#2d2d30',
  bgHover: '#2a2d2e',
  bgActive: '#37373d',
  textPrimary: '#cccccc',
  textSecondary: '#d4d4d4',
  textMuted: '#9d9d9d',
  textFaint: '#6e7681',
  accent: '#007acc',
  accentBlue: '#569cd6',
  accentGreen: '#89d185',
  accentYellow: '#dcdcaa',
  accentPeach: '#ce9178',
  accentRed: '#f14c4c',
  accentPink: '#c586c0',
  accentCyan: '#4ec9b0',
  accentOrange: '#ce9178',
  accentMauve: '#c586c0',
  accentLavender: '#9cdcfe',
  success: '#89d185',
  warning: '#cca700',
  error: '#f14c4c',
  info: '#3794ff',
  borderDefault: '#3c3c3c',
  borderSubtle: '#2b2b2b',
  borderFocus: '#007acc',
};

const DARK_MODERN: ThemePalette = {
  ...DARK_PLUS,
  bgBase: '#1f1f1f',
  bgSidebar: '#181818',
  bgSurface: '#1f1f1f',
  bgElevated: '#2b2b2b',
  bgActive: '#37373d',
  accent: '#0078d4',
  accentBlue: '#4daafc',
  info: '#4daafc',
  borderDefault: '#2b2b2b',
  borderSubtle: '#1f1f1f',
  borderFocus: '#0078d4',
};

const LIGHT_PLUS: ThemePalette = {
  bgBase: '#ffffff',
  bgSidebar: '#f3f3f3',
  bgSurface: '#ffffff',
  bgElevated: '#f3f3f3',
  bgHover: '#e8e8e8',
  bgActive: '#e4e6f1',
  textPrimary: '#333333',
  textSecondary: '#3b3b3b',
  textMuted: '#717171',
  textFaint: '#a0a0a0',
  accent: '#007acc',
  accentBlue: '#0066bf',
  accentGreen: '#388a34',
  accentYellow: '#bf8803',
  accentPeach: '#a31515',
  accentRed: '#a31515',
  accentPink: '#af00db',
  accentCyan: '#267f99',
  accentOrange: '#a31515',
  accentMauve: '#af00db',
  accentLavender: '#0000ff',
  success: '#388a34',
  warning: '#bf8803',
  error: '#a31515',
  info: '#0066bf',
  borderDefault: '#d4d4d4',
  borderSubtle: '#e5e5e5',
  borderFocus: '#007acc',
};

const MONOKAI: ThemePalette = {
  bgBase: '#272822',
  bgSidebar: '#1e1f1c',
  bgSurface: '#272822',
  bgElevated: '#3e3d32',
  bgHover: '#3e3d32',
  bgActive: '#49483e',
  textPrimary: '#f8f8f2',
  textSecondary: '#f8f8f2',
  textMuted: '#9e9e8f',
  textFaint: '#75715e',
  accent: '#a6e22e',
  accentBlue: '#66d9ef',
  accentGreen: '#a6e22e',
  accentYellow: '#e6db74',
  accentPeach: '#fd971f',
  accentRed: '#f92672',
  accentPink: '#f92672',
  accentCyan: '#66d9ef',
  accentOrange: '#fd971f',
  accentMauve: '#ae81ff',
  accentLavender: '#ae81ff',
  success: '#a6e22e',
  warning: '#e6db74',
  error: '#f92672',
  info: '#66d9ef',
  borderDefault: '#49483e',
  borderSubtle: '#3e3d32',
  borderFocus: '#a6e22e',
};

const SOLARIZED_DARK: ThemePalette = {
  bgBase: '#002b36',
  bgSidebar: '#073642',
  bgSurface: '#002b36',
  bgElevated: '#073642',
  bgHover: '#0a4a5a',
  bgActive: '#586e75',
  textPrimary: '#93a1a1',
  textSecondary: '#839496',
  textMuted: '#657b83',
  textFaint: '#586e75',
  accent: '#268bd2',
  accentBlue: '#268bd2',
  accentGreen: '#859900',
  accentYellow: '#b58900',
  accentPeach: '#cb4b16',
  accentRed: '#dc322f',
  accentPink: '#d33682',
  accentCyan: '#2aa198',
  accentOrange: '#cb4b16',
  accentMauve: '#6c71c4',
  accentLavender: '#6c71c4',
  success: '#859900',
  warning: '#b58900',
  error: '#dc322f',
  info: '#268bd2',
  borderDefault: '#586e75',
  borderSubtle: '#073642',
  borderFocus: '#268bd2',
};

const SOLARIZED_LIGHT: ThemePalette = {
  ...SOLARIZED_DARK,
  bgBase: '#fdf6e3',
  bgSidebar: '#eee8d5',
  bgSurface: '#fdf6e3',
  bgElevated: '#eee8d5',
  bgHover: '#e6dfc8',
  bgActive: '#d9d2be',
  textPrimary: '#586e75',
  textSecondary: '#657b83',
  textMuted: '#839496',
  textFaint: '#93a1a1',
  borderDefault: '#d9d2be',
  borderSubtle: '#eee8d5',
};

export const THEME_PRESETS: ThemePreset[] = [
  { id: 'vscode-dark-plus', label: 'VS Code Dark+', kind: 'dark', palette: DARK_PLUS },
  { id: 'vscode-dark-modern', label: 'VS Code Dark Modern', kind: 'dark', palette: DARK_MODERN },
  { id: 'vscode-light-plus', label: 'VS Code Light+', kind: 'light', palette: LIGHT_PLUS },
  { id: 'vscode-monokai', label: 'Monokai', kind: 'dark', palette: MONOKAI },
  { id: 'vscode-solarized-dark', label: 'Solarized Dark', kind: 'dark', palette: SOLARIZED_DARK },
  {
    id: 'vscode-solarized-light',
    label: 'Solarized Light',
    kind: 'light',
    palette: SOLARIZED_LIGHT,
  },
];

export const DEFAULT_THEME_PRESET_ID = 'vscode-dark-plus';

export function findThemePreset(id: string | undefined | null): ThemePreset | undefined {
  if (!id) return undefined;
  return THEME_PRESETS.find((preset) => preset.id === id);
}

/** Maps a palette to the CSS custom properties the app reads. */
export function paletteCssVars(palette: ThemePalette): Record<string, string> {
  return {
    '--bg-base': palette.bgBase,
    '--bg-sidebar': palette.bgSidebar,
    '--bg-surface': palette.bgSurface,
    '--bg-elevated': palette.bgElevated,
    '--bg-hover': palette.bgHover,
    '--bg-active': palette.bgActive,
    '--text-primary': palette.textPrimary,
    '--text-secondary': palette.textSecondary,
    '--text-muted': palette.textMuted,
    '--text-faint': palette.textFaint,
    '--accent-purple': palette.accent,
    '--accent-blue': palette.accentBlue,
    '--accent-green': palette.accentGreen,
    '--accent-yellow': palette.accentYellow,
    '--accent-peach': palette.accentPeach,
    '--accent-red': palette.accentRed,
    '--accent-pink': palette.accentPink,
    '--accent-cyan': palette.accentCyan,
    '--accent-orange': palette.accentOrange,
    '--accent-mauve': palette.accentMauve,
    '--accent-lavender': palette.accentLavender,
    '--color-success': palette.success,
    '--color-warning': palette.warning,
    '--color-error': palette.error,
    '--color-info': palette.info,
    '--border-default': palette.borderDefault,
    '--border-subtle': palette.borderSubtle,
    '--border-focus': palette.borderFocus,
  };
}
