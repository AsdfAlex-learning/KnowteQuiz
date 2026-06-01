<template>
  <div class="space-y-1.5">
    <div class="flex items-center justify-between">
      <label class="text-xs text-[#a6adc8]">Your Reasoning</label>
      <button
        class="text-[10px] px-2 py-0.5 rounded bg-[#313244] text-[#a6adc8] hover:bg-[#45475a] hover:text-[#f8f8f2] transition-colors"
        :disabled="disabled"
        @click="$emit('update:modelValue', 'I am not sure about this answer.')"
      >
        Not sure
      </button>
    </div>
    <textarea
      :value="modelValue"
      class="w-full bg-[#1e1e2e] border border-[#45475a] rounded-lg px-3 py-2 text-sm text-[#f8f8f2] placeholder-[#585b70] focus:outline-none focus:border-[#cba6f7] transition-colors resize-y min-h-[96px]"
      placeholder="Explain your reasoning. Why did you choose this answer? What logic or formula did you use?"
      :disabled="disabled"
      rows="4"
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
