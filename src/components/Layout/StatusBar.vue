<template>
  <div
    class="h-[var(--statusbar-height)] flex items-center justify-between px-3 bg-[var(--bg-sidebar)] border-t border-[var(--border-subtle)] text-[var(--text-xs)] text-[var(--text-muted)]"
  >
    <!-- Left: Current note path -->
    <div class="flex items-center gap-2 min-w-0 flex-1">
      <span v-if="readerStore.currentNote" class="truncate" :title="readerStore.currentNote.path">
        {{ readerStore.currentNote.path }}
      </span>
      <span v-else class="text-[var(--text-faint)]">No file open</span>
    </div>

    <!-- Center: Word count -->
    <div v-if="readerStore.currentNote" class="flex items-center gap-3 text-[var(--text-faint)]">
      <span>{{ wordCount }} words</span>
    </div>

    <!-- Right: LLM status -->
    <div class="flex items-center gap-2 flex-1 justify-end">
      <span class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full" :class="statusDotClass" />
        {{ statusLabel }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useReaderStore } from '@/stores/reader';
import { useSettingsStore } from '@/stores/settings';

const readerStore = useReaderStore();
const settingsStore = useSettingsStore();

const wordCount = computed(() => readerStore.wordCount);

const statusDotClass = computed(() => {
  if (settingsStore.llmConnected) return 'bg-[var(--color-success)]';
  if (settingsStore.settings.llm.api_key || settingsStore.settings.llm.base_url.includes('localhost')) {
    return 'bg-[var(--color-warning)]';
  }
  return 'bg-[var(--text-faint)]';
});

const statusLabel = computed(() => {
  if (settingsStore.llmConnected) return 'LLM connected';
  if (settingsStore.settings.llm.api_key || settingsStore.settings.llm.base_url.includes('localhost')) {
    return 'LLM ready';
  }
  return 'LLM not configured';
});
</script>
