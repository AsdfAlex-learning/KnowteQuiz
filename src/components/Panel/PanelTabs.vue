<template>
  <div class="flex border-b border-[var(--border-default)]">
    <button
      v-for="tab in tabs"
      :key="tab.key"
      class="flex-1 px-3 py-2.5 text-sm font-medium transition-all duration-[var(--transition-base)] relative"
      :class="
        modelValue === tab.key
          ? 'text-[var(--accent-purple)]'
          : 'text-[var(--text-muted)] hover:text-[var(--text-primary)]'
      "
      @click="$emit('update:modelValue', tab.key)"
    >
      {{ tab.label }}
      <span
        v-if="modelValue === tab.key"
        class="absolute bottom-0 left-2 right-2 h-0.5 bg-[var(--accent-purple)] rounded-full"
      />
    </button>
  </div>
</template>

<script setup lang="ts">
export type PanelTab = 'quiz' | 'mistakes' | 'settings';

interface TabDef {
  key: PanelTab;
  label: string;
}

defineProps<{
  modelValue: PanelTab;
}>();

defineEmits<{
  'update:modelValue': [value: PanelTab];
}>();

const tabs: TabDef[] = [
  { key: 'quiz', label: 'Quiz' },
  { key: 'mistakes', label: 'Mistakes' },
  { key: 'settings', label: 'Settings' },
];
</script>
