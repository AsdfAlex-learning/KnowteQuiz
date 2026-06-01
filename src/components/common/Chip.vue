<template>
  <span :class="classes">
    <slot />
    <button
      v-if="removable"
      class="ml-1 -mr-0.5 w-3.5 h-3.5 flex items-center justify-center rounded-full hover:bg-white/20 transition-colors"
      aria-label="Remove"
      @click.stop="$emit('remove')"
    >
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M3 3l4 4M7 3l-4 4" />
      </svg>
    </button>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  variant?: 'default' | 'purple' | 'green' | 'pink' | 'cyan' | 'yellow' | 'red'
  removable?: boolean
}>(), {
  variant: 'default',
  removable: false,
})

defineEmits<{ remove: [] }>()

const variantStyles: Record<string, string> = {
  default: 'bg-[var(--bg-elevated)] text-[var(--text-secondary)] border-[var(--border-default)]',
  purple: 'bg-[var(--accent-purple)]/15 text-[var(--accent-purple)] border-[var(--accent-purple)]/30',
  green: 'bg-[var(--accent-green)]/15 text-[var(--accent-green)] border-[var(--accent-green)]/30',
  pink: 'bg-[var(--accent-pink)]/15 text-[var(--accent-pink)] border-[var(--accent-pink)]/30',
  cyan: 'bg-[var(--accent-cyan)]/15 text-[var(--accent-cyan)] border-[var(--accent-cyan)]/30',
  yellow: 'bg-[var(--accent-yellow)]/15 text-[var(--accent-yellow)] border-[var(--accent-yellow)]/30',
  red: 'bg-[var(--color-error)]/15 text-[var(--color-error)] border-[var(--color-error)]/30',
}

const classes = computed(() => [
  'inline-flex items-center gap-1 px-2 py-0.5 text-[var(--text-xs)] font-medium rounded-[var(--radius-sm)] border',
  'transition-colors duration-[var(--transition-fast)]',
  variantStyles[props.variant],
])
</script>