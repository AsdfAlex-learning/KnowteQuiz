<template>
  <video
    ref="videoRef"
    class="fixed inset-0 -z-[1] object-cover w-full h-full"
    :src="videoSrc"
    muted
    loop
    playsinline
    @timeupdate="onTimeUpdate"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { convertFileSrc, isTauri } from '@/services/tauri';

const props = defineProps<{
  videoPath: string | null;
  playing: boolean;
  savedTime: number;
}>();

const emit = defineEmits<{
  stopped: [screenshotDataUrl: string];
  timeUpdate: [currentTime: number];
}>();

const videoRef = ref<HTMLVideoElement | null>(null);
const prefersReducedMotion = ref(false);

// Respect OS "reduce motion" preference — video backgrounds are a continuous
// animation.  When enabled we skip auto-play and the play() call so the video
// remains paused on a still frame (or does not mount at all).
if (typeof window !== 'undefined') {
  const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
  prefersReducedMotion.value = mq.matches;
  mq.addEventListener('change', (e) => {
    prefersReducedMotion.value = e.matches;
  });
}

const videoSrc = computed(() => {
  if (!props.videoPath) return '';
  if (props.videoPath.startsWith('data:') || props.videoPath.startsWith('http')) {
    return props.videoPath;
  }
  if (isTauri()) {
    return convertFileSrc(props.videoPath);
  }
  return props.videoPath;
});

function onTimeUpdate() {
  if (videoRef.value) {
    emit('timeUpdate', videoRef.value.currentTime);
  }
}

function play() {
  if (prefersReducedMotion.value) return;
  videoRef.value?.play().catch(() => {});
}

function pause() {
  if (videoRef.value) {
    videoRef.value.pause();
    emit('timeUpdate', videoRef.value.currentTime);
  }
}

function stop() {
  const video = videoRef.value;
  if (!video) return;

  video.pause();

  // Only attempt capture when a frame is actually available
  if (video.readyState >= 2 && video.videoWidth > 0 && video.videoHeight > 0) {
    try {
      const canvas = document.createElement('canvas');
      canvas.width = video.videoWidth;
      canvas.height = video.videoHeight;
      const ctx = canvas.getContext('2d');
      if (ctx) {
        ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
        emit('stopped', canvas.toDataURL('image/png'));
        return;
      }
    } catch {
      /* fall through to stopped-without-screenshot */
    }
  }

  // Fallback: still signal stopped so the video unmounts and memory is released
  emit('stopped', '');
}

function resume() {
  const video = videoRef.value;
  if (!video) return;

  if (props.savedTime > 0) {
    if (video.readyState >= 1 && video.duration > props.savedTime) {
      video.currentTime = props.savedTime;
    } else {
      const seekOnReady = () => {
        if (video.duration > props.savedTime) {
          video.currentTime = props.savedTime;
        }
        video.removeEventListener('loadedmetadata', seekOnReady);
      };
      video.addEventListener('loadedmetadata', seekOnReady);
    }
  }
  video.play().catch(() => {});
}

function seek(time: number) {
  if (videoRef.value) {
    videoRef.value.currentTime = time;
  }
}

function getCurrentTime(): number {
  return videoRef.value?.currentTime ?? 0;
}

// Watch for playing state changes
watch(
  () => props.playing,
  (playing) => {
    if (playing) {
      play();
    } else {
      pause();
    }
  }
);

// Watch for video path changes
watch(
  () => props.videoPath,
  () => {
    const video = videoRef.value;
    if (!video || !props.playing) return;
    const onReady = () => {
      video.removeEventListener('canplay', onReady);
      video.play().catch(() => {});
    };
    video.addEventListener('canplay', onReady);
  }
);

onMounted(() => {
  const video = videoRef.value;
  if (!video) return;

  if (props.playing && props.savedTime > 0) {
    // Wait for metadata before seeking to avoid silent ignore
    const seekOnReady = () => {
      if (video.duration > props.savedTime) {
        video.currentTime = props.savedTime;
      }
      video.removeEventListener('loadedmetadata', seekOnReady);
      video.play().catch(() => {});
    };
    video.addEventListener('loadedmetadata', seekOnReady);
  } else if (props.playing) {
    video.play().catch(() => {});
  }
});

onUnmounted(() => {
  if (videoRef.value) {
    videoRef.value.pause();
    videoRef.value.removeAttribute('src');
    videoRef.value.load();
  }
});

defineExpose({
  play,
  pause,
  stop,
  resume,
  seek,
  getCurrentTime,
});
</script>
