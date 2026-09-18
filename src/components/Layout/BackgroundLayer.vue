<template>
  <div class="fixed inset-0 -z-[1] pointer-events-none">
    <!-- Video background (when active) -->
    <VideoBackground
      v-if="theme.background_video && !videoStopped"
      ref="videoBgRef"
      :video-path="theme.background_video"
      :playing="theme.video_playing"
      :saved-time="theme.video_time"
      @stopped="onVideoStopped"
      @time-update="onVideoTimeUpdate"
    />

    <!-- Static screenshot background (when video stopped) -->
    <div v-if="videoStopped && screenshotUrl" class="absolute inset-0" :style="screenshotStyle" />

    <!-- Base background layer -->
    <div id="background-layer" class="absolute inset-0" :style="layerStyle" />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { getAssetUrl } from '@/composables/useTheme';
import { withAlpha } from '@/utils/color';
import VideoBackground from './VideoBackground.vue';

const settingsStore = useSettingsStore();
const videoBgRef = ref<InstanceType<typeof VideoBackground> | null>(null);
const videoStopped = ref(false);
const screenshotUrl = ref<string | null>(null);

const theme = computed(() => settingsStore.settings.theme_config);

const layerStyle = computed(() => {
  const t = theme.value;
  if (!t) return {};

  const styles: Record<string, string> = {};

  // Background color
  if (t.background_color) {
    styles.backgroundColor = t.background_color;
  }

  // Background image (only when no video is active or video is stopped without screenshot)
  if (t.background_image && (!t.background_video || (videoStopped.value && !screenshotUrl.value))) {
    styles.backgroundImage = `url(${getAssetUrl(t.background_image)})`;
    styles.backgroundSize = 'cover';
    styles.backgroundPosition = 'center';
    styles.backgroundRepeat = 'no-repeat';
  }

  // Glassmorphism overlay
  if (t.glassmorphism?.enabled) {
    const opacity = (t.glassmorphism.opacity ?? 20) / 100;
    styles.backdropFilter = 'blur(12px) saturate(180%)';
    styles.WebkitBackdropFilter = 'blur(12px) saturate(180%)';
    styles.backgroundColor = withAlpha(t.background_color || '#1e1e2e', opacity);
  }

  return styles;
});

const screenshotStyle = computed(() => ({
  backgroundImage: `url(${screenshotUrl.value})`,
  backgroundSize: 'cover',
  backgroundPosition: 'center',
  backgroundRepeat: 'no-repeat',
}));

function onVideoStopped(dataUrl: string) {
  videoStopped.value = true;
  screenshotUrl.value = dataUrl;
}

function onVideoTimeUpdate(currentTime: number) {
  settingsStore.settings.theme_config.video_time = currentTime;
}

// Reset stopped state when video path changes or video is set to play
watch(
  () => theme.value.background_video,
  () => {
    videoStopped.value = false;
    screenshotUrl.value = null;
  }
);

watch(
  () => theme.value.video_playing,
  (playing) => {
    if (playing && videoStopped.value) {
      videoStopped.value = false;
      screenshotUrl.value = null;
    }
  }
);

defineExpose({
  videoBgRef,
});
</script>
