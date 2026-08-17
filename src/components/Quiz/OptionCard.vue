<template>
  <button
    class="w-full text-left px-3 py-2.5 rounded-lg border transition-all btn-press"
    :class="optionClasses"
    :disabled="disabled"
    @click="!disabled && $emit('select')"
  >
    <div class="flex items-start gap-2.5">
      <span
        class="shrink-0 w-6 h-6 rounded-md flex items-center justify-center text-xs font-bold mt-0.5"
        :class="letterClasses"
      >
        {{ letter }}
      </span>
      <span class="text-sm leading-relaxed" :class="textClasses">
        {{ text }}
      </span>
    </div>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';

export type OptionState = 'default' | 'selected' | 'correct' | 'incorrect';

const props = defineProps<{
  letter: string;
  text: string;
  state: OptionState;
  disabled?: boolean;
}>();

defineEmits<{
  select: [];
}>();

const optionClasses = computed(() => {
  const base = 'border-';
  switch (props.state) {
    case 'correct':
      return 'border-[var(--accent-green)] bg-[var(--accent-green)]/10';
    case 'incorrect':
      return 'border-[var(--color-error)] bg-[var(--color-error)]/10';
    case 'selected':
      return 'border-[var(--border-focus)] bg-[var(--accent-purple)]/10';
    default:
      return props.disabled
        ? 'border-[var(--border-subtle)] opacity-60'
        : 'border-[var(--border-default)] hover:border-[var(--text-faint)] hover:bg-[var(--bg-elevated)]/50';
  }
});

const letterClasses = computed(() => {
  switch (props.state) {
    case 'correct':
      return 'bg-[var(--accent-green)] text-[var(--bg-base)]';
    case 'incorrect':
      return 'bg-[var(--color-error)] text-[var(--bg-base)]';
    case 'selected':
      return 'bg-[var(--accent-purple)] text-[var(--bg-base)]';
    default:
      return 'bg-[var(--bg-elevated)] text-[var(--text-muted)]';
  }
});

const textClasses = computed(() => {
  switch (props.state) {
    case 'correct':
      return 'text-[var(--accent-green)]';
    case 'incorrect':
      return 'text-[var(--color-error)]';
    case 'selected':
      return 'text-[var(--accent-purple)]';
    default:
      return 'text-[var(--text-secondary)]';
  }
});
</script>
