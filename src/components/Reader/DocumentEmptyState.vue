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
      <p v-if="status && status.stale" class="text-xs text-[var(--color-warning)]">
        {{ t('document.stale_hint') }}
      </p>
      <p v-else class="text-xs text-[var(--text-muted)]">{{ t('sidebar.document_description') }}</p>
    </div>

    <!-- Converting -->
    <div v-if="converting" class="flex items-center gap-2 text-xs text-[var(--text-muted)]">
      <span class="w-3.5 h-3.5 border-2 border-[var(--border-focus)] border-t-transparent rounded-full animate-spin" />
      {{ t('document.converting') }}
    </div>

    <!-- Actions -->
    <div v-else class="flex flex-wrap items-center justify-center gap-2">
      <button
        v-if="!status?.converted"
        class="px-4 py-2 rounded-lg text-sm font-medium bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)] transition-colors disabled:opacity-50 disabled:cursor-wait"
        :disabled="converting"
        @click="handleConvert"
      >
        {{ t('document.convert') }}
      </button>

      <template v-else>
        <button
          class="px-4 py-2 rounded-lg text-sm font-medium bg-[var(--accent-purple)] text-[var(--bg-base)] hover:bg-[var(--accent-lavender)] transition-colors"
          @click="openConverted"
        >
          {{ t('document.open_converted') }}
        </button>
        <button
          v-if="status?.stale"
          class="px-4 py-2 rounded-lg text-sm font-medium bg-[var(--color-warning)]/20 text-[var(--color-warning)] hover:bg-[var(--color-warning)]/30 transition-colors"
          :title="t('document.reconvert_overwrites')"
          @click="handleConvert"
        >
          {{ t('document.reconvert') }}
        </button>
      </template>
    </div>

    <!-- Errors / warnings -->
    <p v-if="error" class="text-xs text-[var(--color-error)] max-w-md">{{ error }}</p>
    <div v-if="warnings.length" class="max-w-md text-left w-full">
      <p class="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-faint)] mb-1">
        {{ t('document.warnings') }}
      </p>
      <ul class="space-y-0.5">
        <li v-for="(w, i) in warnings" :key="i" class="text-[11px] text-[var(--text-muted)]">· {{ w }}</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { convertDocument, getDocumentStatus } from '@/services/docConvert';
import { useReaderStore } from '@/stores/reader';
import { useNavigationStore } from '@/stores/navigation';
import { useI18n } from '@/composables/useI18n';
import type { DocumentStatus } from '@/types/docConvert';

const { t } = useI18n();
const readerStore = useReaderStore();
const navigationStore = useNavigationStore();

const status = ref<DocumentStatus | null>(null);
const converting = ref(false);
const error = ref<string | null>(null);
const warnings = ref<string[]>([]);

const fileName = computed(() => {
  const path = readerStore.pendingDocument ?? '';
  return path.split(/[\\/]/).pop() ?? path;
});

watch(
  () => readerStore.pendingDocument,
  async (path) => {
    status.value = null;
    error.value = null;
    warnings.value = [];
    if (!path) return;
    try {
      status.value = await getDocumentStatus(path);
    } catch (e) {
      error.value = String(e);
    }
  },
  { immediate: true }
);

async function handleConvert() {
  const path = readerStore.pendingDocument;
  if (!path || converting.value) return;
  converting.value = true;
  error.value = null;
  warnings.value = [];
  try {
    const outcome = await convertDocument(path);
    warnings.value = outcome.warnings;
    status.value = await getDocumentStatus(path);
  } catch (e) {
    error.value = String(e);
  } finally {
    converting.value = false;
  }
}

function openConverted() {
  const output = status.value?.output_path;
  if (output) void navigationStore.openNote(output);
}
</script>
