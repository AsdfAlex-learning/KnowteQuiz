<template>
  <div class="flex-1 flex flex-col items-center justify-center gap-4 p-6 text-center">
    <svg
      class="w-12 h-12 text-[var(--text-faint)]"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.2"
    >
      <path d="M6 2h7l5 5v14a1 1 0 01-1 1H6a1 1 0 01-1-1V3a1 1 0 011-1z" />
      <path d="M13 2v5h5" />
    </svg>

    <div class="space-y-1">
      <p class="text-sm font-medium text-[var(--text-primary)]">{{ fileName }}</p>
      <p class="text-xs text-[var(--text-muted)]">{{ t('sidebar.document_description') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useReaderStore } from '@/stores/reader';
import { useI18n } from '@/composables/useI18n';

const { t } = useI18n();
const readerStore = useReaderStore();

const fileName = computed(() => {
  const path = readerStore.pendingDocument ?? '';
  return path.split(/[\\/]/).pop() ?? path;
});
</script>
