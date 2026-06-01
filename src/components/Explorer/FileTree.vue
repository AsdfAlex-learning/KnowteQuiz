<template>
  <div class="flex flex-col h-full bg-[var(--bg-sidebar)]">
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-[var(--border-subtle)]">
      <h2 class="text-[var(--text-xs)] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
        Explorer
      </h2>
      <div class="flex items-center gap-1">
        <button
          class="icon-btn"
          title="Refresh"
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
    <div v-if="explorerStore.error" class="px-3 py-2 text-[var(--text-xs)] text-[var(--color-error)] bg-[var(--color-error)]/10">
      {{ explorerStore.error }}
    </div>

    <!-- File tree -->
    <div class="flex-1 overflow-y-auto px-1 py-1">
      <template v-if="explorerStore.tree.length > 0">
        <FileTreeItem
          v-for="node in explorerStore.tree"
          :key="node.path"
          :node="node"
          :depth="0"
        />
      </template>
      <div v-else-if="explorerStore.rootPath" class="px-3 py-8 text-center text-[var(--text-xs)] text-[var(--text-faint)]">
        No markdown files found
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useExplorerStore } from '@/stores/explorer'
import FileTreeItem from './FileTreeItem.vue'
import FolderSelector from './FolderSelector.vue'

const explorerStore = useExplorerStore()
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
</style>