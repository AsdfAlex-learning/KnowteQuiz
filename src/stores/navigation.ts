import { defineStore } from 'pinia';
import { ref } from 'vue';
import { useExplorerStore } from './explorer';
import { useReaderStore } from './reader';
import { selectFolder } from '../services/note';

export const useNavigationStore = defineStore('navigation', () => {
  const opening = ref(false);
  const error = ref<string | null>(null);

  async function openNote(path: string) {
    opening.value = true;
    error.value = null;
    const explorerStore = useExplorerStore();
    const readerStore = useReaderStore();
    try {
      const loaded = await readerStore.loadNote(path);
      if (!loaded) {
        error.value = readerStore.error;
        return;
      }
      await explorerStore.selectPath(path);
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      opening.value = false;
    }
  }

  /// Select a document (PDF/DOCX/PPTX/HTML). No I/O — the reader shows a
  /// conversion prompt instead of rendering the file as Markdown.
  async function openDocument(path: string) {
    error.value = null;
    const explorerStore = useExplorerStore();
    const readerStore = useReaderStore();
    readerStore.openDocumentPath(path);
    await explorerStore.selectPath(path);
  }

  async function openRootPath(path: string) {
    opening.value = true;
    error.value = null;
    const explorerStore = useExplorerStore();
    const readerStore = useReaderStore();
    const previousRoot = explorerStore.rootPath;
    try {
      await explorerStore.openRootPath(path);
      if (previousRoot !== explorerStore.rootPath) {
        readerStore.clearNote();
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      opening.value = false;
    }
  }

  async function restoreSelectedNote() {
    const explorerStore = useExplorerStore();
    const readerStore = useReaderStore();
    if (!explorerStore.selectedPath) return;

    opening.value = true;
    error.value = null;
    try {
      const loaded = await readerStore.loadNote(explorerStore.selectedPath);
      if (!loaded) {
        error.value = readerStore.error;
        await explorerStore.clearSelectedPath();
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      opening.value = false;
    }
  }

  async function chooseFolder() {
    try {
      const path = await selectFolder();
      if (path) {
        await openRootPath(path);
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  return {
    opening,
    error,
    openNote,
    openDocument,
    openRootPath,
    restoreSelectedNote,
    chooseFolder,
  };
});
