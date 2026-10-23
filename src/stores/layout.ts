import { defineStore } from 'pinia';
import { ref } from 'vue';
import { getSettings } from '../services/settings';
import type { ViewMode } from '../types/settings';
import { useSettingsStore } from './settings';

export const useLayoutStore = defineStore('layout', () => {
  const leftPanelOpen = ref(true);
  const rightPanelOpen = ref(true);
  const explorerWidth = ref(280);
  const readerWidth = ref(360);
  const viewMode = ref<ViewMode>('reader');

  function toggleLeftPanel() {
    leftPanelOpen.value = !leftPanelOpen.value;
    persistLayout();
  }

  function toggleRightPanel() {
    rightPanelOpen.value = !rightPanelOpen.value;
    persistLayout();
  }

  function setExplorerWidth(width: number) {
    explorerWidth.value = Math.max(150, Math.min(500, width));
  }

  function setReaderWidth(width: number) {
    readerWidth.value = Math.max(200, Math.min(600, width));
  }

  function setViewMode(mode: ViewMode) {
    viewMode.value = mode;
    void persistLayout();
  }

  function toggleViewMode() {
    setViewMode(viewMode.value === 'reader' ? 'exploration' : 'reader');
  }

  async function loadLayout() {
    try {
      const settings = await getSettings();
      leftPanelOpen.value = settings.ui.layout.left_visible;
      rightPanelOpen.value = settings.ui.layout.right_visible;
      setExplorerWidth(settings.ui.layout.left_width);
      setReaderWidth(settings.ui.layout.right_width);
      // Persisted unknown values (or a legacy file without the field) fall
      // back to the reader instead of failing.
      viewMode.value = settings.ui.last_mode === 'exploration' ? 'exploration' : 'reader';
    } catch {
      // use defaults
    }
  }

  async function persistLayout() {
    try {
      await useSettingsStore().queueUpdate((s) => {
        s.ui.layout.left_visible = leftPanelOpen.value;
        s.ui.layout.right_visible = rightPanelOpen.value;
        s.ui.layout.left_width = explorerWidth.value;
        s.ui.layout.right_width = readerWidth.value;
        s.ui.last_mode = viewMode.value;
      });
    } catch {
      /* ignore */
    }
  }

  return {
    leftPanelOpen,
    rightPanelOpen,
    explorerWidth,
    readerWidth,
    viewMode,
    toggleLeftPanel,
    toggleRightPanel,
    setExplorerWidth,
    setReaderWidth,
    setViewMode,
    toggleViewMode,
    loadLayout,
    persistLayout,
  };
});
