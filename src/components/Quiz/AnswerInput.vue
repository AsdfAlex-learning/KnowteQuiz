<template>
  <div class="space-y-1.5">
    <label class="text-xs text-[var(--text-muted)] block">Short Answer</label>
    <textarea
      :value="modelValue"
      class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[#585b70] focus:outline-none focus:border-[var(--border-focus)] transition-colors resize-y min-h-[72px]"
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
