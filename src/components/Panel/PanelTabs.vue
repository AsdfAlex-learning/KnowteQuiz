<template>
  <div class="flex border-b border-[#45475a]">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      class="flex-1 px-3 py-2.5 text-sm font-medium transition-colors relative"
      :class="
        modelValue === tab.key
          ? 'text-[#cba6f7]'
          : 'text-[#a6adc8] hover:text-[#f8f8f2]'
      "
      @click="$emit('update:modelValue', tab.key)"
    >
      {{ tab.label }}
      <span
        v-if="modelValue === tab.key"
        class="absolute bottom-0 left-2 right-2 h-0.5 bg-[#cba6f7] rounded-full"
      />
    </button>
  </div>
</template>

<script setup lang="ts">
export type PanelTab = 'quiz' | 'mistakes' | 'settings'

interface TabDef {
  key: PanelTab
  label: string
}

defineProps<{
  modelValue: PanelTab
}>()

defineEmits<{
  'update:modelValue': [value: PanelTab]
}>()

const tabs: TabDef[] = [
  { key: 'quiz', label: 'Quiz' },
  { key: 'mistakes', label: 'Mistakes' },
  { key: 'settings', label: 'Settings' },
]
</script>
