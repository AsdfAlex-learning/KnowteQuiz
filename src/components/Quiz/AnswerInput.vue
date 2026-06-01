<template>
  <div class="space-y-1.5">
    <label class="text-xs text-[#a6adc8] block">Short Answer</label>
    <textarea
      :value="modelValue"
      class="w-full bg-[#1e1e2e] border border-[#45475a] rounded-lg px-3 py-2 text-sm text-[#f8f8f2] placeholder-[#585b70] focus:outline-none focus:border-[#cba6f7] transition-colors resize-y min-h-[72px]"
      placeholder="Type your answer..."
      :disabled="disabled"
      rows="3"
      @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
  </div>
</template>

<script setup lang="ts">
defineProps<{
  modelValue: string
  disabled?: boolean
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>
