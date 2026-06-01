<template>
  <div>
    <!-- Directory node -->
    <div
      v-if="node.is_dir"
      class="select-none"
    >
      <div
        class="tree-item"
        :style="{ paddingLeft: depth * 12 + 8 + 'px' }"
        :title="node.name"
        @click="explorerStore.toggleDir(node.path)"
      >
        <!-- Chevron -->
        <svg
          class="w-3.5 h-3.5 flex-shrink-0 transition-transform duration-150"
          :class="isExpanded ? 'rotate-90' : ''"
          viewBox="0 0 16 16"
          fill="currentColor"
        >
          <path d="M6 4l4 4-4 4V4z" />
        </svg>

        <!-- Folder icon -->
        <svg class="w-4 h-4 flex-shrink-0 text-[var(--accent-yellow)]" viewBox="0 0 16 16" fill="currentColor">
          <path d="M1 3.5A1.5 1.5 0 012.5 2h3.172a1.5 1.5 0 011.06.44l.708.707a.5.5 0 00.353.146H13.5A1.5 1.5 0 0115 4.793v7.707a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9z" />
        </svg>

        <span class="truncate">{{ node.name }}</span>
      </div>

      <!-- Children (recursive) -->
      <div v-show="isExpanded">
        <FileTreeItem
          v-for="child in node.children"
          :key="child.path"
          :node="child"
          :depth="depth + 1"
        />
      </div>
    </div>

    <!-- File node -->
    <div
      v-else
      class="tree-item"
      :class="{ 'tree-item--active': isSelected }"
      :style="{ paddingLeft: depth * 12 + 20 + 'px' }"
      :title="node.name"
      @click="explorerStore.selectPath(node.path)"
    >
      <!-- File icon -->
      <svg class="w-4 h-4 flex-shrink-0 text-[var(--text-muted)]" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.2">
        <path d="M4 2h5l3 3v8a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1z" />
        <path d="M9 2v3h3" />
      </svg>

      <span class="truncate">{{ displayName }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useExplorerStore } from '@/stores/explorer'
import type { NoteTreeNode } from '@/types/note'

const props = defineProps<{
  node: NoteTreeNode
  depth: number
}>()

const explorerStore = useExplorerStore()

const isExpanded = computed(() => explorerStore.expandedDirs.has(props.node.path))
const isSelected = computed(() => explorerStore.selectedPath === props.node.path)

const displayName = computed(() => {
  // Strip .md extension for display
  return props.node.name.replace(/\.md$/i, '')
})
</script>

<style scoped>
.tree-item {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 26px;
  padding-right: 8px;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.tree-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tree-item--active {
  background: var(--accent-purple) / 12;
  color: var(--accent-purple);
}

.tree-item--active:hover {
  background: var(--accent-purple) / 20;
  color: var(--accent-purple);
}
</style>