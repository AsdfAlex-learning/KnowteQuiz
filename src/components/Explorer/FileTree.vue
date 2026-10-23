<template>
  <div class="flex flex-col h-full bg-[var(--bg-sidebar)]" data-glass-fill>
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)]">
      <h2 class="text-[var(--text-xs)] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
        {{ t('sidebar.explorer') }}
      </h2>
      <div class="flex items-center gap-1">
        <template v-if="isExploration">
          <button
            class="icon-btn"
            :title="t('exploration.select_all')"
            :disabled="selectablePaths.length === 0"
            @click="explorationStore.selectAll(selectablePaths)"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M1.5 7.5l2 2 3.5-4.5" />
              <path d="M6 7.5l2 2 4.5-5" />
            </svg>
          </button>
          <button
            class="icon-btn"
            :title="t('exploration.clear_selection')"
            :disabled="explorationStore.selectedCount === 0"
            @click="explorationStore.clearSelection()"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M3.5 3.5l7 7M10.5 3.5l-7 7" />
            </svg>
          </button>
        </template>
        <button
          class="icon-btn"
          :title="t('sidebar.refresh')"
          :disabled="explorerStore.isLoading"
          @click="explorerStore.loadTree()"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 14 14"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            :class="{ 'animate-spin': explorerStore.isLoading }"
          >
            <path d="M1.5 7a5.5 5.5 0 0 1 9.3-4M12.5 7a5.5 5.5 0 0 1-9.3 4" />
            <path d="M11 1.5h2v2M1 10.5h2v2" />
          </svg>
        </button>
      </div>
    </div>

    <!-- Folder selector -->
    <FolderSelector />

    <!-- Error state -->
    <div
      v-if="explorerStore.error"
      class="px-3 py-2 text-[var(--text-xs)] text-[var(--color-error)] bg-[var(--color-error)]/10"
    >
      {{ explorerStore.error }}
    </div>

    <!-- File tree -->
    <div class="flex-1 overflow-y-auto px-1 py-1">
      <template v-if="explorerStore.tree.length > 0">
        <FileTreeItem v-for="node in explorerStore.tree" :key="node.path" :node="node" :depth="0" />
      </template>
      <div
        v-else-if="explorerStore.rootPath"
        class="px-3 py-8 text-center text-[var(--text-xs)] text-[var(--text-faint)]"
      >
        {{ t('sidebar.no_files') }}
      </div>
    </div>

    <!-- Mode toggle -->
    <div class="border-t border-[var(--border-subtle)] p-2">
      <button class="mode-toggle" :title="modeTitle" @click="layoutStore.toggleViewMode()">
        <svg
          v-if="isExploration"
          class="w-4 h-4 shrink-0"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.4"
        >
          <path d="M2 3.5A1.5 1.5 0 013.5 2H8v12H3.5A1.5 1.5 0 012 12.5v-9z" />
          <path d="M8 2h4.5A1.5 1.5 0 0114 3.5v9a1.5 1.5 0 01-1.5 1.5H8" />
        </svg>
        <svg v-else class="w-4 h-4 shrink-0" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4">
          <circle cx="8" cy="8" r="5.5" />
          <path d="M10.6 5.4l-1.4 3.8-3.8 1.4 1.4-3.8 3.8-1.4z" />
        </svg>
        <span class="truncate">
          {{ isExploration ? t('exploration.mode_reader') : t('exploration.mode_exploration') }}
        </span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useExplorationStore } from '@/stores/exploration';
import { useExplorerStore } from '@/stores/explorer';
import { useLayoutStore } from '@/stores/layout';
import { useI18n } from '@/composables/useI18n';
import { collectMarkdownPaths } from '@/utils/noteTree';
import FileTreeItem from './FileTreeItem.vue';
import FolderSelector from './FolderSelector.vue';

const explorationStore = useExplorationStore();
const explorerStore = useExplorerStore();
const layoutStore = useLayoutStore();
const { t } = useI18n();

const isExploration = computed(() => layoutStore.viewMode === 'exploration');
const selectablePaths = computed(() => collectMarkdownPaths(explorerStore.tree));
const modeTitle = computed(() =>
  isExploration.value ? t('exploration.switch_to_reader') : t('exploration.switch_to_exploration')
);
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
  transition: all var(--transition-fast);
}

.icon-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.icon-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.mode-toggle {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  font-size: var(--text-sm);
  color: var(--text-secondary);
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
}

.mode-toggle:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
