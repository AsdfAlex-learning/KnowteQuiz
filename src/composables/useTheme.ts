import { watch, onMounted, onUnmounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import type { ThemeConfig } from '@/types/settings';
import { isTauri, convertFileSrc } from '@/services/tauri';
import { clampBlur } from '@/utils/glass';

const CSS_VAR_BG_BASE = '--bg-base';
const CSS_VAR_ACCENT_PURPLE = '--accent-purple';
const CSS_VAR_BORDER_FOCUS = '--border-focus';

const DEFAULT_BG_BASE = '#1e1e2e';
const DEFAULT_ACCENT_PURPLE = '#cba6f7';
const DEFAULT_BORDER_FOCUS = '#cba6f7';

function applyThemeToRoot(theme: ThemeConfig) {
  const root = document.documentElement;

  // Apply background color
  root.style.setProperty(CSS_VAR_BG_BASE, theme.background_color || DEFAULT_BG_BASE);

  // Apply accent color
  const accent = theme.accent_color || DEFAULT_ACCENT_PURPLE;
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

  // Set glassmorphism blur radius CSS variable (clamped to the supported 0-30 px range)
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

  const unwatch = watch(
    () => settingsStore.settings.theme_config,
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
