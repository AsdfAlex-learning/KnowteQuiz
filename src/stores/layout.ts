import { defineStore } from 'pinia'
import { ref } from 'vue'
import { getSettings, saveSettings } from '../services/settings'

export const useLayoutStore = defineStore('layout', () => {
  const leftPanelOpen = ref(true)
  const rightPanelOpen = ref(true)
  const explorerWidth = ref(280)
  const readerWidth = ref(360)

  function toggleLeftPanel() {
    leftPanelOpen.value = !leftPanelOpen.value
    persistLayout()
  }

  function toggleRightPanel() {
    rightPanelOpen.value = !rightPanelOpen.value
    persistLayout()
  }

  function setExplorerWidth(width: number) {
    explorerWidth.value = Math.max(150, Math.min(500, width))
  }

  function setReaderWidth(width: number) {
    readerWidth.value = Math.max(200, Math.min(600, width))
  }

  async function loadLayout() {
    try {
      const settings = await getSettings()
      leftPanelOpen.value = settings.ui.layout.left_visible
      rightPanelOpen.value = settings.ui.layout.right_visible
      setExplorerWidth(settings.ui.layout.left_width)
      setReaderWidth(settings.ui.layout.right_width)
    } catch {
      // use defaults
    }
  }

  async function persistLayout() {
    try {
      const settings = await getSettings()
      settings.ui.layout.left_visible = leftPanelOpen.value
      settings.ui.layout.right_visible = rightPanelOpen.value
      settings.ui.layout.left_width = explorerWidth.value
      settings.ui.layout.right_width = readerWidth.value
      await saveSettings(settings)
    } catch { /* ignore */ }
  }

  return {
    leftPanelOpen, rightPanelOpen, explorerWidth, readerWidth,
    toggleLeftPanel, toggleRightPanel, setExplorerWidth, setReaderWidth,
    loadLayout, persistLayout,
  }
})
