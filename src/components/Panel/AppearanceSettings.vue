<template>
  <div class="space-y-6">
    <!-- Background Color -->
    <div class="space-y-2">
      <label for="bg-color-input" class="text-sm font-medium text-[var(--text-primary)]">
        {{ t('appearance.background_color') }}
      </label>
      <div class="flex items-center gap-3">
        <input
          id="bg-color-input"
          type="color"
          :value="modelValue.background_color"
          class="w-10 h-10 rounded-lg border border-[var(--border-default)] cursor-pointer bg-transparent"
          @input="updateBackgroundColor"
        />
        <span class="text-xs text-[var(--text-muted)] font-mono">
          {{ modelValue.background_color }}
        </span>
      </div>
    </div>

    <!-- Accent Color -->
    <div class="space-y-2">
      <label for="accent-color-input" class="text-sm font-medium text-[var(--text-primary)]">
        {{ t('appearance.accent_color') }}
      </label>
      <div class="flex items-center gap-3">
        <input
          id="accent-color-input"
          type="color"
          :value="modelValue.accent_color"
          class="w-10 h-10 rounded-lg border border-[var(--border-default)] cursor-pointer bg-transparent"
          @input="updateAccentColor"
        />
        <span class="text-xs text-[var(--text-muted)] font-mono">
          {{ modelValue.accent_color }}
        </span>
      </div>
    </div>

    <!-- Background Image -->
    <div class="space-y-2">
      <label class="text-sm font-medium text-[var(--text-primary)]">
        {{ t('appearance.background_image') }}
      </label>
      <div class="flex items-center gap-2">
        <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="handleImageSelect" />
        <button
          class="px-3 py-2 rounded-lg text-sm bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
          @click="fileInput?.click()"
        >
          {{ t('appearance.choose_image') }}
        </button>
        <button
          v-if="modelValue.background_image"
          class="px-3 py-2 rounded-lg text-sm text-[var(--color-error)] hover:bg-[var(--color-error)]/10 transition-colors"
          @click="clearImage"
        >
          {{ t('appearance.clear_image') }}
        </button>
      </div>
      <p v-if="modelValue.background_image" class="text-xs text-[var(--text-muted)] truncate">
        {{ modelValue.background_image }}
      </p>
      <p v-if="imageError" class="text-xs text-[var(--color-error)]">
        {{ imageError }}
      </p>
    </div>

    <!-- Background Video -->
    <div class="space-y-3 rounded-lg border border-[var(--border-default)] bg-[var(--bg-elevated)] p-4">
      <div class="flex items-center justify-between">
        <label class="text-sm font-medium text-[var(--text-primary)]">
          {{ t('appearance.background_video') }}
        </label>
        <span v-if="modelValue.background_video" class="text-xs px-2 py-0.5 rounded-full" :class="videoStatusClass">
          {{ videoStatusText }}
        </span>
      </div>

      <!-- Video file input -->
      <div class="flex items-center gap-2">
        <input ref="videoInput" type="file" accept="video/mp4" class="hidden" @change="handleVideoSelect" />
        <button
          class="px-3 py-2 rounded-lg text-sm bg-[var(--bg-base)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
          @click="videoInput?.click()"
        >
          {{ t('appearance.choose_video') }}
        </button>
        <button
          v-if="modelValue.background_video"
          class="px-3 py-2 rounded-lg text-sm text-[var(--color-error)] hover:bg-[var(--color-error)]/10 transition-colors"
          @click="clearVideo"
        >
          {{ t('appearance.clear_video') }}
        </button>
      </div>

      <p v-if="modelValue.background_video" class="text-xs text-[var(--text-muted)] truncate">
        {{ modelValue.background_video }}
      </p>
      <p v-if="videoError" class="text-xs text-[var(--color-error)]">
        {{ videoError }}
      </p>

      <!-- Playback controls -->
      <template v-if="modelValue.background_video">
        <div class="flex items-center gap-2 pt-2 border-t border-[var(--border-default)]">
          <button
            v-if="!modelValue.video_playing"
            class="p-2 rounded-lg bg-[var(--bg-base)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
            :title="t('appearance.play')"
            @click="playVideo"
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z" />
            </svg>
          </button>
          <button
            v-else
            class="p-2 rounded-lg bg-[var(--bg-base)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
            :title="t('appearance.pause')"
            @click="pauseVideo"
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
            </svg>
          </button>
          <button
            class="p-2 rounded-lg bg-[var(--bg-base)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
            :title="t('appearance.stop')"
            @click="stopVideo"
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 6h12v12H6z" />
            </svg>
          </button>
          <button
            class="p-2 rounded-lg bg-[var(--bg-base)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
            :title="t('appearance.resume')"
            @click="resumeVideo"
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M4 18l8.5-6L4 6v12zm9-12v12l8.5-6L13 6z" />
            </svg>
          </button>
        </div>

        <!-- Current time display -->
        <div class="flex items-center justify-between">
          <span class="text-xs text-[var(--text-muted)]">
            {{ t('appearance.current_time') }}: {{ formatTime(modelValue.video_time) }}
          </span>
        </div>

        <!-- Switch buttons -->
        <div class="flex gap-2 pt-2 border-t border-[var(--border-default)]">
          <button
            class="flex-1 px-3 py-2 rounded-lg text-xs bg-[var(--bg-base)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
            @click="switchToStatic"
          >
            {{ t('appearance.switch_static') }}
          </button>
          <button
            class="flex-1 px-3 py-2 rounded-lg text-xs bg-[var(--bg-base)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)]"
            @click="switchToWallpaper"
          >
            {{ t('appearance.switch_wallpaper') }}
          </button>
        </div>
      </template>
    </div>

    <!-- Glassmorphism -->
    <div class="space-y-3 rounded-lg border border-[var(--border-default)] bg-[var(--bg-elevated)] p-4">
      <div class="flex items-center justify-between">
        <label class="text-sm font-medium text-[var(--text-primary)]">
          {{ t('appearance.glassmorphism') }}
        </label>
        <button
          class="relative w-11 h-6 rounded-full transition-colors"
          :class="modelValue.glassmorphism.enabled ? 'bg-[var(--accent-purple)]' : 'bg-[var(--bg-active)]'"
          @click="toggleGlassmorphism"
        >
          <span
            class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform"
            :class="modelValue.glassmorphism.enabled ? 'translate-x-5' : 'translate-x-0'"
          />
        </button>
      </div>

      <template v-if="modelValue.glassmorphism.enabled">
        <!-- Opacity Slider -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <label class="text-xs text-[var(--text-muted)]">
              {{ t('appearance.opacity') }}
            </label>
            <span class="text-xs text-[var(--text-muted)] font-mono"> {{ modelValue.glassmorphism.opacity }}% </span>
          </div>
          <input type="range" min="0" max="100" :value="modelValue.glassmorphism.opacity" @input="updateOpacity" />
        </div>

        <!-- Blur Radius -->
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <label class="text-xs text-[var(--text-muted)]">
              {{ t('appearance.blur') }}
            </label>
            <div class="flex items-center gap-1">
              <input
                type="number"
                min="0"
                max="30"
                step="1"
                class="w-16 px-2 py-1 rounded-md text-xs font-mono bg-[var(--bg-base)] text-[var(--text-primary)] border border-[var(--border-default)]"
                :value="modelValue.glassmorphism.blur"
                @input="updateBlur"
              />
              <span class="text-xs text-[var(--text-muted)]">px</span>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs text-[var(--text-muted)] font-mono">0px</span>
            <input
              type="range"
              min="0"
              max="30"
              step="1"
              class="flex-1"
              :value="modelValue.glassmorphism.blur"
              @input="updateBlur"
            />
            <span class="text-xs text-[var(--text-muted)] font-mono">30px</span>
          </div>
        </div>

        <!-- Scope Selector -->
        <div class="space-y-2">
          <label class="text-xs text-[var(--text-muted)]">
            {{ t('appearance.scope') }}
          </label>
          <div class="flex gap-2">
            <button
              class="flex-1 py-2 rounded-lg text-sm border transition-colors"
              :class="
                modelValue.glassmorphism.scope === 'global'
                  ? 'bg-[var(--accent-purple)] text-[var(--bg-base)] border-[var(--accent-purple)]'
                  : 'bg-[var(--bg-base)] text-[var(--text-primary)] border-[var(--border-default)] hover:bg-[var(--bg-active)]'
              "
              @click="updateScope('global')"
            >
              {{ t('appearance.scope_global') }}
            </button>
            <button
              class="flex-1 py-2 rounded-lg text-sm border transition-colors"
              :class="
                modelValue.glassmorphism.scope === 'content'
                  ? 'bg-[var(--accent-purple)] text-[var(--bg-base)] border-[var(--accent-purple)]'
                  : 'bg-[var(--bg-base)] text-[var(--text-primary)] border-[var(--border-default)] hover:bg-[var(--bg-active)]'
              "
              @click="updateScope('content')"
            >
              {{ t('appearance.scope_content') }}
            </button>
          </div>
        </div>
      </template>
    </div>

    <!-- Preset Library -->
    <div class="space-y-2">
      <label class="text-sm font-medium text-[var(--text-primary)]">
        {{ t('appearance.presets') }}
      </label>
      <div
        class="rounded-lg border border-dashed border-[var(--border-default)] bg-[var(--bg-elevated)] p-4 text-center"
      >
        <p class="text-xs text-[var(--text-muted)]">
          {{ t('appearance.presets_coming_soon') }}
        </p>
      </div>
    </div>

    <!-- Reset to Defaults -->
    <div class="pt-2">
      <button
        class="w-full py-2 rounded-lg text-sm font-medium bg-[var(--bg-elevated)] text-[var(--text-primary)] hover:bg-[var(--bg-active)] transition-colors border border-[var(--border-default)] btn-press"
        @click="resetToDefaults"
      >
        {{ t('appearance.reset_defaults') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from '@/composables/useI18n';
import { requestStop, requestResume, resetVideoState } from '@/composables/useVideoBackground';
import { clampBlur } from '@/utils/glass';
import type { ThemeConfig } from '@/types/settings';

const MAX_IMAGE_BYTES = 5 * 1024 * 1024;
const MAX_VIDEO_BYTES = 50 * 1024 * 1024;

const props = defineProps<{
  modelValue: ThemeConfig;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: ThemeConfig];
}>();

const { t } = useI18n();
const fileInput = ref<HTMLInputElement | null>(null);
const videoInput = ref<HTMLInputElement | null>(null);
const imageError = ref<string | null>(null);
const videoError = ref<string | null>(null);

const videoStatusText = computed(() => {
  if (!props.modelValue.background_video) return '';
  if (props.modelValue.video_playing) return t('appearance.status_playing');
  return t('appearance.status_paused');
});

const videoStatusClass = computed(() => {
  if (props.modelValue.video_playing) {
    return 'bg-green-500/20 text-green-400';
  }
  return 'bg-yellow-500/20 text-yellow-400';
});

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

function updateBackgroundColor(e: Event) {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', {
    ...props.modelValue,
    background_color: target.value,
  });
}

function updateAccentColor(e: Event) {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', {
    ...props.modelValue,
    accent_color: target.value,
  });
}

function handleImageSelect(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;

  if (file.size > MAX_IMAGE_BYTES) {
    imageError.value = t('appearance.image_too_large');
    target.value = '';
    return;
  }
  imageError.value = null;

  const reader = new FileReader();
  reader.onload = () => {
    emit('update:modelValue', {
      ...props.modelValue,
      background_image: reader.result as string,
    });
  };
  reader.readAsDataURL(file);
}

function clearImage() {
  imageError.value = null;
  emit('update:modelValue', {
    ...props.modelValue,
    background_image: null,
  });
  if (fileInput.value) {
    fileInput.value.value = '';
  }
}

function handleVideoSelect(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file) return;

  if (file.size > MAX_VIDEO_BYTES) {
    videoError.value = t('appearance.video_too_large');
    target.value = '';
    return;
  }
  videoError.value = null;
  resetVideoState();

  const reader = new FileReader();
  reader.onload = () => {
    emit('update:modelValue', {
      ...props.modelValue,
      background_video: reader.result as string,
      video_playing: true,
      video_time: 0,
    });
  };
  reader.readAsDataURL(file);
}

function clearVideo() {
  videoError.value = null;
  resetVideoState();
  emit('update:modelValue', {
    ...props.modelValue,
    background_video: null,
    video_playing: false,
    video_time: 0,
  });
  if (videoInput.value) {
    videoInput.value.value = '';
  }
}

function playVideo() {
  emit('update:modelValue', {
    ...props.modelValue,
    video_playing: true,
  });
}

function pauseVideo() {
  emit('update:modelValue', {
    ...props.modelValue,
    video_playing: false,
  });
}

function stopVideo() {
  requestStop();
  emit('update:modelValue', {
    ...props.modelValue,
    video_playing: false,
  });
}

function resumeVideo() {
  requestResume();
  emit('update:modelValue', {
    ...props.modelValue,
    video_playing: true,
  });
}

function switchToStatic() {
  requestStop();
  emit('update:modelValue', {
    ...props.modelValue,
    video_playing: false,
  });
}

function switchToWallpaper() {
  requestStop();
  emit('update:modelValue', {
    ...props.modelValue,
    background_video: null,
    video_playing: false,
    video_time: 0,
    background_image: props.modelValue.background_image,
  });
}

function toggleGlassmorphism() {
  emit('update:modelValue', {
    ...props.modelValue,
    glassmorphism: {
      ...props.modelValue.glassmorphism,
      enabled: !props.modelValue.glassmorphism.enabled,
    },
  });
}

function updateOpacity(e: Event) {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', {
    ...props.modelValue,
    glassmorphism: {
      ...props.modelValue.glassmorphism,
      opacity: Number(target.value),
    },
  });
}

function updateBlur(e: Event) {
  const target = e.target as HTMLInputElement;
  emit('update:modelValue', {
    ...props.modelValue,
    glassmorphism: {
      ...props.modelValue.glassmorphism,
      blur: clampBlur(target.value),
    },
  });
}

function updateScope(scope: 'global' | 'content') {
  emit('update:modelValue', {
    ...props.modelValue,
    glassmorphism: {
      ...props.modelValue.glassmorphism,
      scope,
    },
  });
}

function resetToDefaults() {
  imageError.value = null;
  videoError.value = null;
  resetVideoState();
  emit('update:modelValue', {
    background_color: '#1e1e2e',
    accent_color: '#cba6f7',
    background_image: null,
    glassmorphism: {
      enabled: false,
      opacity: 20,
      blur: 12,
      scope: 'global',
    },
    background_video: null,
    video_playing: false,
    video_time: 0,
  });
}
</script>
