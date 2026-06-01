<template>
  <div :class="classes">
    <div v-if="$slots.header || title" class="px-4 py-3 border-b border-[var(--border-subtle)]">
      <slot name="header">
        <h3 class="text-[var(--text-base)] font-semibold text-[var(--text-primary)]">{{ title }}</h3>
        <p v-if="subtitle" class="text-[var(--text-xs)] text-[var(--text-muted)] mt-0.5">{{ subtitle }}</p>
      </slot>
    </div>
    <div :class="bodyClasses">
      <slot />
    </div>
    <div v-if="$slots.footer" class="px-4 py-3 border-t border-[var(--border-subtle)]">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  title?: string
  subtitle?: string
  variant?: 'default' | 'elevated' | 'outlined'
  padding?: 'none' | 'sm' | 'md' | 'lg'
}>(), {
  title: '',
  subtitle: '',
  variant: 'default',
  padding: 'md',
})

const classes = computed(() => [
  'rounded-[var(--radius-lg)] overflow-hidden',
  {
    default: 'bg-[var(--bg-surface)] border border-[var(--border-subtle)]',
    elevated: 'bg-[var(--bg-elevated)] shadow-[var(--shadow-md)]',
    outlined: 'bg-transparent border border-[var(--border-default)]',
  }[props.variant],
])

const bodyClasses = computed(() => [
  {
    none: '',
    sm: 'p-3',
    md: 'p-4',
    lg: 'p-6',
  }[props.padding],
])
</script>