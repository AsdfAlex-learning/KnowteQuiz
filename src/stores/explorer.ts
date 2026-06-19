import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { NoteTreeNode } from '../types/note'
import { selectFolder, scanNotes } from '../services/note'
import { getSettings, saveSettings } from '../services/settings'

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
        await persistWorkspace()
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

  async function toggleDir(path: string) {
    if (expandedDirs.value.has(path)) {
      expandedDirs.value.delete(path)
    } else {
      expandedDirs.value.add(path)
    }
    await persistWorkspace()
  }

  async function selectPath(path: string) {
    selectedPath.value = path
    await persistWorkspace()
  }

  async function restoreWorkspace() {
    loading.value = true
    error.value = null
    try {
      const settings = await getSettings()
      rootPath.value = settings.workspace.root_path ?? null
      expandedDirs.value = new Set(settings.workspace.expanded_dirs ?? [])
      selectedPath.value = settings.workspace.selected_path ?? null
      if (rootPath.value) {
        tree.value = await scanNotes(rootPath.value)
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function persistWorkspace() {
    try {
      const settings = await getSettings()
      await saveSettings({
        ...settings,
        workspace: {
          root_path: rootPath.value,
          expanded_dirs: Array.from(expandedDirs.value),
          selected_path: selectedPath.value,
        },
      })
    } catch (e) {
      error.value = String(e)
    }
  }

  return {
    rootPath, tree, expandedDirs, selectedPath, loading, isLoading, error,
    chooseFolder, loadTree, toggleDir, selectPath, restoreWorkspace, persistWorkspace,
  }
})
