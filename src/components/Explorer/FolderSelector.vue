<template>
  <div class="px-3 py-2">
    <div v-if="!isTauri && !explorerStore.rootPath" class="mb-2">
      <input
        v-model="manualPath"
        type="text"
        placeholder="Enter notes folder path..."
        class="w-full px-2 py-1.5 text-xs bg-[#1e1e2e] text-[#f8f8f2] border border-[#44475a] rounded focus:border-[#bd93f9] focus:outline-none"
        @keydown.enter="setManualPath"
      />
    </div>

    <button
      v-if="explorerStore.rootPath"
      class="folder-btn"
      :title="explorerStore.rootPath"
      @click="explorerStore.chooseFolder()"
    >
      <svg class="w-4 h-4 flex-shrink-0 text-[var(--accent-yellow)]" viewBox="0 0 16 16" fill="currentColor">
        <path d="M1 3.5A1.5 1.5 0 012.5 2h3.172a1.5 1.5 0 011.06.44l.708.707a.5.5 0 00.353.146H13.5A1.5 1.5 0 0115 4.793v7.707a1.5 1.5 0 01-1.5 1.5h-11A1.5 1.5 0 011 12.5v-9z" />
      </svg>
      <span class="truncate flex-1 text-left">{{ shortPath }}</span>
      <svg class="w-3.5 h-3.5 flex-shrink-0 text-[var(--text-muted)]" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M6 4l4 4-4 4" />
      </svg>
    </button>

    <button
      v-else
      class="open-folder-btn"
      @click="explorerStore.chooseFolder()"
    >
      <svg class="w-5 h-5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
        <path d="M8 2v6m0 0v6m0-6h6m-6 0H2" />
      </svg>
      <span>Open Folder</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useExplorerStore } from '@/stores/explorer'
import { isTauri } from '@/services/tauri'

const explorerStore = useExplorerStore()
const manualPath = ref('')

const shortPath = computed(() => {
  const path = explorerStore.rootPath
  if (!path) return ''
  const parts = path.replace(/\\/g, '/').split('/')
  return parts[parts.length - 1] || path
})

async function setManualPath() {
  const path = manualPath.value.trim()
  if (path) {
    explorerStore.rootPath = path
    await explorerStore.loadTree()
    manualPath.value = ''
  }
}
</script>

<style scoped>
.folder-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 4px 8px;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast);
}

.folder-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.open-folder-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 10px 16px;
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--accent-purple);
  background: var(--accent-purple) / 10;
  border: 1px dashed var(--accent-purple) / 30;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.open-folder-btn:hover {
  background: var(--accent-purple) / 20;
  border-color: var(--accent-purple) / 50;
}
</style>
