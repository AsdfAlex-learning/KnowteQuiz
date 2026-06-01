<template>
  <button
    class="w-full text-left px-3 py-2.5 rounded-lg border transition-all"
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
import { computed } from 'vue'

export type OptionState = 'default' | 'selected' | 'correct' | 'incorrect'

const props = defineProps<{
  letter: string
  text: string
  state: OptionState
  disabled?: boolean
}>()

defineEmits<{
  select: []
}>()

const optionClasses = computed(() => {
  const base = 'border-'
  switch (props.state) {
    case 'correct':
      return 'border-[#a6e3a1] bg-[#a6e3a1]/10'
    case 'incorrect':
      return 'border-[#f38ba8] bg-[#f38ba8]/10'
    case 'selected':
      return 'border-[#cba6f7] bg-[#cba6f7]/10'
    default:
      return props.disabled
        ? 'border-[#313244] opacity-60'
        : 'border-[#45475a] hover:border-[#585b70] hover:bg-[#313244]/50'
  }
})

const letterClasses = computed(() => {
  switch (props.state) {
    case 'correct':
      return 'bg-[#a6e3a1] text-[#1e1e2e]'
    case 'incorrect':
      return 'bg-[#f38ba8] text-[#1e1e2e]'
    case 'selected':
      return 'bg-[#cba6f7] text-[#1e1e2e]'
    default:
      return 'bg-[#313244] text-[#a6adc8]'
  }
})

const textClasses = computed(() => {
  switch (props.state) {
    case 'correct':
      return 'text-[#a6e3a1]'
    case 'incorrect':
      return 'text-[#f38ba8]'
    case 'selected':
      return 'text-[#cba6f7]'
    default:
      return 'text-[#cdd6f4]'
  }
})
</script>
