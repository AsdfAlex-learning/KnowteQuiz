<template>
  <div class="flex items-center border-b border-[var(--border-default)]">
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

    <!-- Settings button -->
    <button
      class="px-3 py-2.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors"
      title="Settings"
      @click="$emit('openSettings')"
    >
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M8 10a2 2 0 100-4 2 2 0 000 4z" />
        <path
          d="M13.5 8c0-.3-.1-.5-.2-.7l1-1.7-1.7-1-1 1a5.3 5.3 0 00-1.1-.6L10.3 3H7.4l-.4 1.7a5.3 5.3 0 00-1.1.6l-1-1-1.7 1 1 1.7c-.1.2-.2.4-.2.7s.1.5.2.7l-1 1.7 1.7 1 1-1c.3.2.7.4 1.1.6l.4 1.7h2.9l.4-1.7c.4-.2.8-.4 1.1-.6l1 1 1.7-1-1-1.7c.1-.2.2-.5.2-.7z"
        />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
interface TabDef {
  key: 'quiz' | 'mistakes';
  label: string;
}

defineProps<{
  modelValue: 'quiz' | 'mistakes';
}>();

defineEmits<{
  'update:modelValue': [value: 'quiz' | 'mistakes'];
  openSettings: [];
}>();

const tabs: TabDef[] = [
  { key: 'quiz', label: 'Quiz' },
  { key: 'mistakes', label: 'Mistakes' },
];
</script>
