<template>
  <div class="fixed inset-0 -z-[1] pointer-events-none">
    <!-- Video background (when active) -->
    <VideoBackground
      v-if="theme.background_video && !isStopped"
      ref="videoBgRef"
      :video-path="theme.background_video"
      :playing="theme.video_playing"
      :saved-time="theme.video_time"
      @stopped="onVideoStopped"
      @time-update="onVideoTimeUpdate"
    />

    <!-- Static screenshot background (when video stopped) -->
    <div v-if="isStopped && screenshotUrl" class="absolute inset-0" :style="screenshotStyle" />

    <!-- Base background layer -->
    <div id="background-layer" class="absolute inset-0" :style="layerStyle" />

    <!-- Readability overlay: flat dark tint over the wallpaper, behind the panels -->
    <div
      v-if="theme.overlay_opacity > 0"
      class="absolute inset-0 pointer-events-none z-[1]"
      :style="{ backgroundColor: `rgba(0, 0, 0, ${theme.overlay_opacity / 100})` }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onUnmounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useVideoBackground } from '@/composables/useVideoBackground';
import { getAssetUrl } from '@/composables/useTheme';
import VideoBackground from './VideoBackground.vue';

const settingsStore = useSettingsStore();
const videoBgRef = ref<InstanceType<typeof VideoBackground> | null>(null);
const { isStopped, screenshotUrl, registerVideoControls } = useVideoBackground();

const theme = computed(() => settingsStore.settings.theme_config);

const layerStyle = computed(() => {
  const t = theme.value;
  if (!t) return {};

  const styles: Record<string, string> = {};

  // While a video wallpaper is playing, keep the base layer transparent so the
  // video underneath stays visible (frosted-glass panels blur it themselves).
  if (t.background_video && !isStopped.value) return styles;

  // Background color
  if (t.background_color) {
    styles.backgroundColor = t.background_color;
  }

  // Background image (only when no video is active or video is stopped without screenshot)
  if (t.background_image && (!t.background_video || (isStopped.value && !screenshotUrl.value))) {
    styles.backgroundImage = `url(${getAssetUrl(t.background_image)})`;
    styles.backgroundSize = 'cover';
    styles.backgroundPosition = 'center';
    styles.backgroundRepeat = 'no-repeat';
  }

  return styles;
});

const screenshotStyle = computed(() => ({
  backgroundImage: `url(${screenshotUrl.value})`,
  backgroundSize: 'cover',
  backgroundPosition: 'center',
  backgroundRepeat: 'no-repeat',
}));

const pendingTime = ref<number | null>(null);
let timeUpdateTimer: ReturnType<typeof setTimeout> | null = null;

function flushTimeUpdate() {
  if (timeUpdateTimer) {
    clearTimeout(timeUpdateTimer);
    timeUpdateTimer = null;
  }
  if (pendingTime.value !== null) {
    settingsStore.settings.theme_config.video_time = pendingTime.value;
    pendingTime.value = null;
  }
}

function onVideoTimeUpdate(currentTime: number) {
  pendingTime.value = currentTime;
  if (timeUpdateTimer) clearTimeout(timeUpdateTimer);
  timeUpdateTimer = setTimeout(() => {
    if (pendingTime.value !== null) {
      settingsStore.settings.theme_config.video_time = pendingTime.value;
      pendingTime.value = null;
    }
    timeUpdateTimer = null;
  }, 1000);
}

function onVideoStopped(dataUrl: string) {
  isStopped.value = true;
  screenshotUrl.value = dataUrl || null;
  flushTimeUpdate();
}

// Register/unregister video controls as the component mounts/unmounts
watch(videoBgRef, (ref) => {
  if (ref) {
    registerVideoControls({
      play: ref.play,
      pause: ref.pause,
      stop: ref.stop,
      resume: ref.resume,
    });
  } else {
    registerVideoControls(null);
  }
});

// Reset stopped state when video path changes
watch(
  () => theme.value.background_video,
  () => {
    isStopped.value = false;
    screenshotUrl.value = null;
  }
);

// Resume from stopped when playing is set to true
watch(
  () => theme.value.video_playing,
  (playing) => {
    if (!playing) {
      flushTimeUpdate();
    }
    if (playing && isStopped.value) {
      isStopped.value = false;
      screenshotUrl.value = null;
    }
  }
);

onUnmounted(() => {
  flushTimeUpdate();
  registerVideoControls(null);
});

defineExpose({
  videoBgRef,
});
</script>
