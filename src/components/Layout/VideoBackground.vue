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
    video.currentTime = props.savedTime;
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
    if (props.playing) {
      // Small delay to let the new source load before playing
      setTimeout(() => play(), 100);
    }
  }
);

onMounted(() => {
  if (props.playing && videoRef.value) {
    if (props.savedTime > 0) {
      videoRef.value.currentTime = props.savedTime;
    }
    videoRef.value.play().catch(() => {});
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
