<template>
  <div class="flex bg-[#1e1e2e] rounded-lg p-0.5 border border-[#45475a]">
    <button
      v-for="opt in options"
      :key="opt.value"
      class="flex-1 py-1.5 text-xs font-medium rounded-md transition-all"
      :class="
        modelValue === opt.value
          ? 'bg-[#cba6f7] text-[#1e1e2e] shadow-sm'
          : 'text-[#a6adc8] hover:text-[#f8f8f2]'
      "
      @click="$emit('update:modelValue', opt.value)"
    >
      {{ opt.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
export type QuizMode = 'basic' | 'advanced'

defineProps<{
  modelValue: QuizMode
}>()

defineEmits<{
  'update:modelValue': [value: QuizMode]
}>()

const options: { value: QuizMode; label: string }[] = [
  { value: 'basic', label: 'Basic' },
  { value: 'advanced', label: 'Advanced' },
]
</script>
