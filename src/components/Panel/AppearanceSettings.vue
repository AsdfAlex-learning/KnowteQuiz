<template>
  <div class="space-y-6">
    <!-- Background Color -->
    <div class="space-y-2">
      <label class="text-sm font-medium text-[var(--text-primary)]">
        {{ t('appearance.background_color') }}
      </label>
      <div class="flex items-center gap-3">
        <input
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
      <label class="text-sm font-medium text-[var(--text-primary)]">
        {{ t('appearance.accent_color') }}
      </label>
      <div class="flex items-center gap-3">
        <input
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
import { ref } from 'vue';
import { useI18n } from '@/composables/useI18n';
import type { ThemeConfig } from '@/types/settings';

const props = defineProps<{
  modelValue: ThemeConfig;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: ThemeConfig];
}>();

const { t } = useI18n();
const fileInput = ref<HTMLInputElement | null>(null);

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
  emit('update:modelValue', {
    ...props.modelValue,
    background_image: null,
  });
  if (fileInput.value) {
    fileInput.value.value = '';
  }
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
  emit('update:modelValue', {
    background_color: '#1e1e2e',
    accent_color: '#cba6f7',
    background_image: null,
    glassmorphism: {
      enabled: false,
      opacity: 20,
      scope: 'global',
    },
  });
}
</script>
