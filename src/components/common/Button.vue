<template>
  <button :class="classes" :disabled="disabled" @click="$emit('click', $event)">
    <span v-if="$slots.icon" class="flex items-center justify-center">
      <slot name="icon" />
    </span>
    <slot />
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
    disabled?: boolean;
    fullWidth?: boolean;
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    disabled: false,
    fullWidth: false,
  }
);

defineEmits<{ click: [e: MouseEvent] }>();

const classes = computed(() => [
  'inline-flex items-center justify-center gap-2 font-medium rounded-[var(--radius-md)] transition-colors duration-[var(--transition-fast)] select-none',
  'focus-visible:outline-2 focus-visible:outline-[var(--border-focus)] focus-visible:outline-offset-2',
  'disabled:opacity-40 disabled:cursor-not-allowed',
  {
    primary:
      'bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-purple)]/85 active:bg-[var(--accent-purple)]/70',
    secondary:
      'bg-[var(--bg-elevated)] text-[var(--text-primary)] border border-[var(--border-default)] hover:bg-[var(--bg-hover)] active:bg-[var(--bg-active)]',
    ghost:
      'text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] active:bg-[var(--bg-active)]',
    danger:
      'bg-[var(--color-error)]/15 text-[var(--color-error)] border border-[var(--color-error)]/30 hover:bg-[var(--color-error)]/25 active:bg-[var(--color-error)]/35',
  }[props.variant],
  {
    sm: 'h-7 px-2.5 text-[var(--text-xs)]',
    md: 'h-8 px-3.5 text-[var(--text-sm)]',
    lg: 'h-10 px-5 text-[var(--text-base)]',
  }[props.size],
  props.fullWidth ? 'w-full' : '',
]);
</script>
