import { watch, onMounted, onUnmounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import type { ThemeConfig } from '@/types/settings';
import { isTauri, convertFileSrc } from '@/services/tauri';
import { clampBlur } from '@/utils/glass';
import { findThemePreset, paletteCssVars } from '@/utils/themes';

const CSS_VAR_BG_BASE = '--bg-base';
const CSS_VAR_ACCENT_PURPLE = '--accent-purple';
const CSS_VAR_BORDER_FOCUS = '--border-focus';

const DEFAULT_BG_BASE = '#1e1e1e';
const DEFAULT_ACCENT_PURPLE = '#007acc';

function applyThemeToRoot(theme: ThemeConfig) {
  const root = document.documentElement;
  const preset = findThemePreset(theme.preset);

  if (preset) {
    for (const [name, value] of Object.entries(paletteCssVars(preset.palette))) {
      root.style.setProperty(name, value);
    }
    root.style.colorScheme = preset.kind;
    // Glass is colourless: it only borrows a neutral tint from the scene.
    root.style.setProperty('--glass-rgb', preset.kind === 'light' ? '15, 23, 42' : '255, 255, 255');
    root.style.setProperty(
      '--glass-border',
      preset.kind === 'light' ? 'rgba(15, 23, 42, 0.10)' : 'rgba(255, 255, 255, 0.15)'
    );
  } else {
    // Custom: preserve the look of settings written before presets existed.
    root.style.colorScheme = 'dark';
    root.style.setProperty('--glass-rgb', '255, 255, 255');
    root.style.setProperty('--glass-border', 'rgba(255, 255, 255, 0.15)');
  }

  // The two user-picked colours win over the preset, so a manual tweak on top
  // of a preset behaves as expected.
  root.style.setProperty(CSS_VAR_BG_BASE, theme.background_color || preset?.palette.bgBase || DEFAULT_BG_BASE);

  const accent = theme.accent_color || preset?.palette.accent || DEFAULT_ACCENT_PURPLE;
  root.style.setProperty(CSS_VAR_ACCENT_PURPLE, accent);
  root.style.setProperty(CSS_VAR_BORDER_FOCUS, accent);

  // Handle glassmorphism data attribute
  if (theme.glassmorphism?.enabled) {
    document.body.setAttribute('data-glassmorphism', theme.glassmorphism.scope === 'content' ? 'content' : 'global');
  } else {
    document.body.removeAttribute('data-glassmorphism');
  }

  // Set glassmorphism opacity CSS variable
  if (theme.glassmorphism?.enabled && theme.glassmorphism?.opacity !== undefined) {
    root.style.setProperty('--glass-opacity', String(theme.glassmorphism.opacity / 100));
  } else {
    root.style.removeProperty('--glass-opacity');
  }

  // Set glassmorphism blur radius CSS variable (clamped to the supported range)
  if (theme.glassmorphism?.enabled) {
    root.style.setProperty('--glass-blur', `${clampBlur(theme.glassmorphism.blur)}px`);
  } else {
    root.style.removeProperty('--glass-blur');
  }
}

export function getAssetUrl(assetPath: string | null): string {
  if (!assetPath) return '';
  if (assetPath.startsWith('data:') || assetPath.startsWith('http')) {
    return assetPath;
  }
  if (isTauri()) {
    return convertFileSrc(assetPath);
  }
  return assetPath;
}

export function useTheme() {
  const settingsStore = useSettingsStore();

  function applyTheme() {
    const theme = settingsStore.settings.theme_config;
    if (!theme) return;
    applyThemeToRoot(theme);
  }

  onMounted(() => {
    applyTheme();
  });

  // Watch only the fields applyThemeToRoot consumes. video_time updates every
  // second during video playback and must not re-run theme application.
  const unwatch = watch(
    [
      () => settingsStore.settings.theme_config?.preset,
      () => settingsStore.settings.theme_config?.background_color,
      () => settingsStore.settings.theme_config?.accent_color,
      () => settingsStore.settings.theme_config?.glassmorphism,
    ],
    () => {
      applyTheme();
    },
    { deep: true }
  );

  onUnmounted(() => {
    unwatch();
  });

  return {
    getAssetUrl,
  };
}
