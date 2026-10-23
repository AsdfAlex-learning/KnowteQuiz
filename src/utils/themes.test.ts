import { describe, expect, it } from 'vitest';
import { DEFAULT_THEME_PRESET_ID, THEME_PRESETS, findThemePreset, paletteCssVars } from './themes';

describe('theme presets', () => {
  it('has unique ids and a valid default', () => {
    const ids = THEME_PRESETS.map((preset) => preset.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(findThemePreset(DEFAULT_THEME_PRESET_ID)).toBeDefined();
  });

  it('ships the VS Code Dark+ palette as the default', () => {
    const preset = findThemePreset(DEFAULT_THEME_PRESET_ID);
    expect(preset?.kind).toBe('dark');
    expect(preset?.palette.bgBase).toBe('#1e1e1e');
    expect(preset?.palette.accent).toBe('#007acc');
    expect(preset?.palette.textPrimary).toBe('#cccccc');
  });

  it('maps every palette to the full CSS variable set', () => {
    const expected = [
      '--bg-base',
      '--bg-sidebar',
      '--bg-surface',
      '--bg-elevated',
      '--bg-hover',
      '--bg-active',
      '--text-primary',
      '--text-secondary',
      '--text-muted',
      '--text-faint',
      '--accent-purple',
      '--accent-blue',
      '--accent-green',
      '--accent-yellow',
      '--accent-peach',
      '--accent-red',
      '--accent-pink',
      '--accent-cyan',
      '--accent-orange',
      '--accent-mauve',
      '--accent-lavender',
      '--color-success',
      '--color-warning',
      '--color-error',
      '--color-info',
      '--border-default',
      '--border-subtle',
      '--border-focus',
    ];

    for (const preset of THEME_PRESETS) {
      const vars = paletteCssVars(preset.palette);
      expect(Object.keys(vars).sort()).toEqual([...expected].sort());
      for (const value of Object.values(vars)) {
        expect(value).toMatch(/^#[0-9a-f]{6}$/i);
      }
    }
  });

  it('includes both dark and light presets', () => {
    const kinds = new Set(THEME_PRESETS.map((preset) => preset.kind));
    expect(kinds.has('dark')).toBe(true);
    expect(kinds.has('light')).toBe(true);
  });
});
