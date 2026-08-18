<template>
  <div class="flex bg-[var(--bg-base)] rounded-lg p-0.5 border border-[var(--border-default)] min-w-0">
    <button
      v-for="opt in options"
      :key="opt.value"
      class="flex-1 min-w-0 py-1.5 px-3 text-xs font-medium rounded-md transition-all btn-press whitespace-nowrap overflow-hidden text-ellipsis"
      :class="
        modelValue === opt.value
          ? 'bg-[var(--accent-purple)] text-[var(--bg-base)] shadow-sm'
          : 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'
      "
      @click="$emit('update:modelValue', opt.value)"
    >
      {{ opt.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '@/composables/useI18n';

export type QuizMode = 'basic' | 'advanced';

defineProps<{
  modelValue: QuizMode;
}>();

defineEmits<{
  'update:modelValue': [value: QuizMode];
}>();

const { t } = useI18n();

const options = [
  { value: 'basic' as QuizMode, label: t('settings_page.basic') },
  { value: 'advanced' as QuizMode, label: t('settings_page.advanced') },
];
</script>
