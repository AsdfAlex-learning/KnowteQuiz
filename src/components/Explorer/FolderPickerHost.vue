<template>
  <Teleport to="body">
    <div
      v-if="state.open"
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50 p-4"
      role="dialog"
      aria-modal="true"
      :aria-label="t('folder_picker.title')"
      @click.self="cancel"
    >
      <div
        class="w-full max-w-lg max-h-[75vh] flex flex-col rounded-[var(--radius-lg)] border border-[var(--border-default)] bg-[var(--bg-base)] shadow-[var(--shadow-lg)]"
      >
        <header class="px-4 py-3 border-b border-[var(--border-subtle)] flex items-center justify-between gap-3">
          <h2 class="text-[var(--text-sm)] font-semibold text-[var(--text-primary)]">
            {{ t('folder_picker.title') }}
          </h2>
          <button
            class="icon-btn"
            :title="t('folder_picker.cancel')"
            :aria-label="t('folder_picker.cancel')"
            @click="cancel"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6">
              <path d="M2.5 2.5l7 7M9.5 2.5l-7 7" />
            </svg>
          </button>
        </header>

        <div class="px-3 py-2 border-b border-[var(--border-subtle)] flex items-center gap-2">
          <button
            class="chip shrink-0"
            :disabled="!listing?.parent"
            :title="t('folder_picker.up')"
            @click="load(listing?.parent ?? undefined)"
          >
            ↑ {{ t('folder_picker.up') }}
          </button>
          <p class="min-w-0 truncate text-[11px] font-mono text-[var(--text-muted)]">
            {{ listing?.path ?? '' }}
          </p>
        </div>

        <div class="flex-1 overflow-auto px-2 py-2">
          <div v-if="roots.length" class="flex flex-wrap gap-1.5 px-1 pb-2">
            <button v-for="root in roots" :key="root.path" class="chip" :title="root.path" @click="load(root.path)">
              {{ root.name }}
            </button>
          </div>

          <button
            v-for="directory in listing?.directories ?? []"
            :key="directory.path"
            class="dir-row"
            :title="directory.path"
            @click="load(directory.path)"
          >
            <svg class="w-4 h-4 shrink-0 text-[var(--accent-yellow)]" viewBox="0 0 16 16" fill="currentColor">
              <path
                d="M1 3.5A1.5 1.5 0 012.5 2h3.172a1.5 1.5 0 011.06.44l.708.707a.5.5 0 00.353.146H13.5A1.5 1.5 0 0115 4.793v7.707a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9z"
              />
            </svg>
            <span class="truncate">{{ directory.name }}</span>
          </button>

          <p v-if="loading" class="px-2 py-6 text-center text-[var(--text-xs)] text-[var(--text-muted)]">
            {{ t('folder_picker.loading') }}
          </p>
          <p
            v-else-if="listing && listing.directories.length === 0"
            class="px-2 py-6 text-center text-[var(--text-xs)] text-[var(--text-faint)]"
          >
            {{ t('folder_picker.empty') }}
          </p>
          <p v-if="error" class="px-2 py-2 text-[var(--text-xs)] text-[var(--color-error)]">
            {{ error }}
          </p>
        </div>

        <footer class="px-4 py-3 border-t border-[var(--border-subtle)] flex items-center justify-end gap-2">
          <button class="secondary-btn" @click="cancel">{{ t('folder_picker.cancel') }}</button>
          <button class="primary-btn" :disabled="!listing || loading" @click="select">
            {{ t('folder_picker.select') }}
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { listDirectories } from '@/services/note';
import { closeFolderPicker, folderPickerState } from '@/composables/useFolderPicker';
import { useI18n } from '@/composables/useI18n';
import type { DirectoryEntry, DirectoryListing } from '@/types/note';

const { t } = useI18n();
const state = folderPickerState();
const listing = ref<DirectoryListing | null>(null);
const roots = ref<DirectoryEntry[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

watch(
  () => state.open,
  (open) => {
    if (open) {
      listing.value = null;
      error.value = null;
      void load(state.startPath ?? undefined);
    }
  },
  { immediate: true }
);

async function load(path?: string) {
  loading.value = true;
  error.value = null;
  try {
    const next = await listDirectories(path);
    listing.value = next;
    roots.value = next.roots;
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function cancel() {
  closeFolderPicker(null);
}

function select() {
  if (listing.value) closeFolderPicker(listing.value.path);
}

function onKeydown(event: KeyboardEvent) {
  if (state.open && event.key === 'Escape') {
    event.preventDefault();
    cancel();
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown));
onUnmounted(() => window.removeEventListener('keydown', onKeydown));
</script>

<style scoped>
.icon-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--text-muted);
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.chip {
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.chip:hover:not(:disabled) {
  background: var(--bg-active);
  color: var(--text-primary);
}

.chip:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.dir-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  text-align: left;
}

.dir-row:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.secondary-btn {
  padding: 6px 14px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.secondary-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.primary-btn {
  padding: 6px 14px;
  border-radius: var(--radius-md);
  background: var(--accent-purple);
  color: var(--bg-base);
  font-size: var(--text-sm);
  font-weight: 500;
}

.primary-btn:hover:not(:disabled) {
  background: var(--accent-lavender);
}

.primary-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
