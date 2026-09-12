import { defineStore } from 'pinia';
import { ref } from 'vue';
import type { NoteTreeNode } from '../types/note';
import { selectFolder, scanNotes } from '../services/note';
import { getSettings, saveSettings } from '../services/settings';
import { useRequestId } from '../composables/useRequestId';

export const useExplorerStore = defineStore('explorer', () => {
  const rootPath = ref<string | null>(null);
  const tree = ref<NoteTreeNode[]>([]);
  const expandedDirs = ref<Set<string>>(new Set());
  const selectedPath = ref<string | null>(null);
  const loading = ref(false);
  const isLoading = loading;
  const error = ref<string | null>(null);
  const scanRequest = useRequestId();

  async function chooseFolder() {
    try {
      const path = await selectFolder();
      if (path) {
        await openRootPath(path);
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  async function openRootPath(path: string) {
    const requestId = scanRequest.next();
    loading.value = true;
    error.value = null;
    try {
      const nextTree = await scanNotes(path);
      if (!scanRequest.isLatest(requestId)) return;
      const rootChanged = rootPath.value !== path;
      rootPath.value = path;
      if (rootChanged) {
        expandedDirs.value = new Set();
        selectedPath.value = null;
      }
      tree.value = nextTree;
      await persistWorkspace();
    } catch (e) {
      if (!scanRequest.isLatest(requestId)) return;
      error.value = String(e);
    } finally {
      if (scanRequest.isLatest(requestId)) {
        loading.value = false;
      }
    }
  }

  async function loadTree() {
    if (!rootPath.value) return;
    const requestId = scanRequest.next();
    const path = rootPath.value;
    loading.value = true;
    error.value = null;
    try {
      const nextTree = await scanNotes(path);
      if (!scanRequest.isLatest(requestId)) return;
      tree.value = nextTree;
    } catch (e) {
      if (!scanRequest.isLatest(requestId)) return;
      error.value = String(e);
    } finally {
      if (scanRequest.isLatest(requestId)) {
        loading.value = false;
      }
    }
  }

  async function toggleDir(path: string) {
    if (expandedDirs.value.has(path)) {
      expandedDirs.value.delete(path);
    } else {
      expandedDirs.value.add(path);
    }
    await persistWorkspace();
  }

  async function selectPath(path: string) {
    selectedPath.value = path;
    await persistWorkspace();
  }

  async function clearSelectedPath() {
    selectedPath.value = null;
    await persistWorkspace();
  }

  async function restoreWorkspace() {
    const requestId = scanRequest.next();
    loading.value = true;
    error.value = null;
    try {
      const settings = await getSettings();
      if (!scanRequest.isLatest(requestId)) return;
      rootPath.value = settings.workspace.root_path ?? null;
      expandedDirs.value = new Set(settings.workspace.expanded_dirs ?? []);
      selectedPath.value = settings.workspace.selected_path ?? null;
      if (rootPath.value) {
        const nextTree = await scanNotes(rootPath.value);
        if (!scanRequest.isLatest(requestId)) return;
        tree.value = nextTree;
      }
    } catch (e) {
      if (!scanRequest.isLatest(requestId)) return;
      error.value = String(e);
    } finally {
      if (scanRequest.isLatest(requestId)) {
        loading.value = false;
      }
    }
  }

  async function persistWorkspace() {
    try {
      const settings = await getSettings();
      await saveSettings({
        ...settings,
        workspace: {
          ...settings.workspace,
          root_path: rootPath.value,
          expanded_dirs: Array.from(expandedDirs.value),
          selected_path: selectedPath.value,
        },
      });
    } catch (e) {
      error.value = String(e);
    }
  }

  return {
    rootPath,
    tree,
    expandedDirs,
    selectedPath,
    loading,
    isLoading,
    error,
    chooseFolder,
    openRootPath,
    loadTree,
    toggleDir,
    selectPath,
    clearSelectedPath,
    restoreWorkspace,
    persistWorkspace,
  };
});
