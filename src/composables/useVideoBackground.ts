import { ref } from 'vue';

export interface VideoControls {
  play(): void;
  pause(): void;
  stop(): void;
  resume(): void;
}

const controls = ref<VideoControls | null>(null);
const isStopped = ref(false);
const screenshotUrl = ref<string | null>(null);

export function registerVideoControls(c: VideoControls | null) {
  controls.value = c;
}

export function requestStop() {
  controls.value?.stop();
}

export function requestResume() {
  isStopped.value = false;
  screenshotUrl.value = null;
}

export function resetVideoState() {
  isStopped.value = false;
  screenshotUrl.value = null;
}

export function useVideoBackground() {
  return {
    controls,
    isStopped,
    screenshotUrl,
    registerVideoControls,
    requestStop,
    requestResume,
    resetVideoState,
  };
}
