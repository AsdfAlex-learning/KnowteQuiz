<template>
  <div class="flex flex-col gap-1.5">
    <label v-if="label" :for="inputId" class="text-[var(--text-sm)] text-[var(--text-secondary)] font-medium">
      {{ label }}
    </label>
    <div class="relative">
      <span v-if="$slots.prefix" class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]">
        <slot name="prefix" />
      </span>
      <input
        :id="inputId"
        ref="inputRef"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :class="classes"
        @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        @focus="$emit('focus', $event)"
        @blur="$emit('blur', $event)"
        @keydown.enter="$emit('enter')"
      />
      <span v-if="$slots.suffix" class="absolute right-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)]">
        <slot name="suffix" />
      </span>
    </div>
    <p v-if="error" class="text-[var(--text-xs)] text-[var(--color-error)]">{{ error }}</p>
    <p v-else-if="hint" class="text-[var(--text-xs)] text-[var(--text-muted)]">{{ hint }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  type?: string
  placeholder?: string
  label?: string
  error?: string
  hint?: string
  disabled?: boolean
  readonly?: boolean
  size?: 'sm' | 'md'
}>(), {
  modelValue: '',
  type: 'text',
  placeholder: '',
  label: '',
  error: '',
  hint: '',
  disabled: false,
  readonly: false,
  size: 'md',
})

defineEmits<{
  'update:modelValue': [value: string]
  focus: [e: FocusEvent]
  blur: [e: FocusEvent]
  enter: []
}>()

const inputRef = ref<HTMLInputElement | null>(null)
const inputId = computed(() => `input-${Math.random().toString(36).slice(2, 9)}`)

const classes = computed(() => [
  'w-full rounded-[var(--radius-md)] border bg-[var(--bg-surface)] text-[var(--text-primary)] placeholder:text-[var(--text-muted)] transition-colors duration-[var(--transition-fast)]',
  'focus:outline-none focus:border-[var(--border-focus)] focus:ring-1 focus:ring-[var(--border-focus)]/30',
  'disabled:opacity-40 disabled:cursor-not-allowed',
  props.error
    ? 'border-[var(--color-error)]'
    : 'border-[var(--border-default)]',
  {
    sm: 'h-7 px-2.5 text-[var(--text-xs)]',
    md: 'h-9 px-3 text-[var(--text-sm)]',
  }[props.size],
])

defineExpose({ inputRef })
</script>