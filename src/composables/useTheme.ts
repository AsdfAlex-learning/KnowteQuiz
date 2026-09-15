import { watch, onMounted, onUnmounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import type { ThemeConfig } from '@/types/settings';

const CSS_VAR_BG_BASE = '--bg-base';
const CSS_VAR_ACCENT_PURPLE = '--accent-purple';
const CSS_VAR_BORDER_FOCUS = '--border-focus';

function applyThemeToRoot(theme: ThemeConfig) {
  const root = document.documentElement;

  // Apply background color
  if (theme.background_color) {
    root.style.setProperty(CSS_VAR_BG_BASE, theme.background_color);
  }

  // Apply accent color
  if (theme.accent_color) {
    root.style.setProperty(CSS_VAR_ACCENT_PURPLE, theme.accent_color);
    root.style.setProperty(CSS_VAR_BORDER_FOCUS, theme.accent_color);
  }

  // Handle glassmorphism data attribute
  if (theme.glassmorphism?.enabled) {
    document.body.setAttribute('data-glassmorphism', theme.glassmorphism.scope === 'content' ? 'content' : 'global');
  } else {
    document.body.removeAttribute('data-glassmorphism');
  }

  // Set glassmorphism opacity CSS variable
  if (theme.glassmorphism?.opacity !== undefined) {
    root.style.setProperty('--glass-opacity', String(theme.glassmorphism.opacity / 100));
  }
}

function applyBackgroundImage(theme: ThemeConfig) {
  const existingLayer = document.getElementById('theme-background-image');
  if (existingLayer) {
    existingLayer.remove();
  }

  if (!theme.background_image) {
    return;
  }

  const layer = document.createElement('div');
  layer.id = 'theme-background-image';
  layer.style.position = 'fixed';
  layer.style.inset = '0';
  layer.style.zIndex = '-2';
  layer.style.backgroundImage = `url(${theme.background_image})`;
  layer.style.backgroundSize = 'cover';
  layer.style.backgroundPosition = 'center';
  layer.style.backgroundRepeat = 'no-repeat';
  document.body.appendChild(layer);
}

export function useTheme() {
  const settingsStore = useSettingsStore();

  function applyTheme() {
    const theme = settingsStore.settings.theme_config;
    if (!theme) return;
    applyThemeToRoot(theme);
    applyBackgroundImage(theme);
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
}
