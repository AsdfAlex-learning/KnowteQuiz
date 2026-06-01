import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { NoteTreeNode } from '../types/note'
import { selectFolder, scanNotes } from '../services/note'

export const useExplorerStore = defineStore('explorer', () => {
  const rootPath = ref<string | null>(null)
  const tree = ref<NoteTreeNode[]>([])
  const expandedDirs = ref<Set<string>>(new Set())
  const selectedPath = ref<string | null>(null)
  const loading = ref(false)
  const isLoading = loading
  const error = ref<string | null>(null)

  async function chooseFolder() {
    try {
      const path = await selectFolder()
      if (path) {
        rootPath.value = path
        await loadTree()
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadTree() {
    if (!rootPath.value) return
    loading.value = true
    error.value = null
    try {
      tree.value = await scanNotes(rootPath.value)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  function toggleDir(path: string) {
    if (expandedDirs.value.has(path)) {
      expandedDirs.value.delete(path)
    } else {
      expandedDirs.value.add(path)
    }
  }

  function selectPath(path: string) {
    selectedPath.value = path
  }

  return {
    rootPath, tree, expandedDirs, selectedPath, loading, isLoading, error,
    chooseFolder, loadTree, toggleDir, selectPath,
  }
})
