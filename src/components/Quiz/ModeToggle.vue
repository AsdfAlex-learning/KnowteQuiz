<template>
  <div class="flex bg-[var(--bg-base)] rounded-lg p-0.5 border border-[var(--border-default)]">
    <button
      v-for="opt in options"
      :key="opt.value"
      class="flex-1 py-1.5 text-xs font-medium rounded-md transition-all btn-press"
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
export type QuizMode = 'basic' | 'advanced';

defineProps<{
  modelValue: QuizMode;
}>();

defineEmits<{
  'update:modelValue': [value: QuizMode];
}>();

const options: { value: QuizMode; label: string }[] = [
  { value: 'basic', label: 'Basic' },
  { value: 'advanced', label: 'Advanced' },
];
</script>
