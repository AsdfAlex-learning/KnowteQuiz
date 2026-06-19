<template>
  <div class="space-y-1.5">
    <div class="flex items-center justify-between">
      <label class="text-xs text-[var(--text-muted)]">Your Reasoning</label>
      <button
        class="text-[10px] px-2 py-0.5 rounded bg-[var(--bg-elevated)] text-[var(--text-muted)] hover:bg-[var(--bg-active)] hover:text-[var(--text-primary)] transition-colors"
        :disabled="disabled"
        @click="$emit('update:modelValue', 'I am not sure about this answer.')"
      >
        Not sure
      </button>
    </div>
    <textarea
      :value="modelValue"
      class="w-full bg-[var(--bg-base)] border border-[var(--border-default)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder-[#585b70] focus:outline-none focus:border-[var(--border-focus)] transition-colors resize-y min-h-[96px]"
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
