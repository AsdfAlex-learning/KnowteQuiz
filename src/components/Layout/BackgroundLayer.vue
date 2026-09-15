<template>
  <div id="background-layer" class="fixed inset-0 -z-[1] pointer-events-none" :style="layerStyle" />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useSettingsStore } from '@/stores/settings';

const settingsStore = useSettingsStore();

const layerStyle = computed(() => {
  const theme = settingsStore.settings.theme_config;
  if (!theme) return {};

  const styles: Record<string, string> = {};

  // Background color
  if (theme.background_color) {
    styles.backgroundColor = theme.background_color;
  }

  // Background image
  if (theme.background_image) {
    styles.backgroundImage = `url(${theme.background_image})`;
    styles.backgroundSize = 'cover';
    styles.backgroundPosition = 'center';
    styles.backgroundRepeat = 'no-repeat';
  }

  // Glassmorphism overlay
  if (theme.glassmorphism?.enabled) {
    const opacity = (theme.glassmorphism.opacity ?? 20) / 100;
    styles.backdropFilter = 'blur(12px) saturate(180%)';
    styles.WebkitBackdropFilter = 'blur(12px) saturate(180%)';
    styles.backgroundColor = `rgba(30, 30, 46, ${opacity})`;
  }

  return styles;
});
</script>
