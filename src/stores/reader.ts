import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { NoteContent } from '../types/note'
import { readNote } from '../services/note'
import { getSettings, saveSettings } from '../services/settings'

export const useReaderStore = defineStore('reader', () => {
  const currentNote = ref<NoteContent | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const scrollTop = ref(0)
  let activeLoadId = 0

  const isLoading = computed(() => loading.value)
  const wordCount = computed(() => {
    if (!currentNote.value) return 0
    return currentNote.value.content.split(/\s+/).length
  })

  async function loadNote(path: string): Promise<boolean> {
    const loadId = ++activeLoadId
    loading.value = true
    error.value = null
    try {
      const note = await readNote(path)
      if (loadId !== activeLoadId) return false
      try {
        const settings = await getSettings()
        if (loadId !== activeLoadId) return false
        scrollTop.value = settings.workspace.scroll_positions?.[path] ?? 0
      } catch {
        if (loadId !== activeLoadId) return false
        scrollTop.value = 0
      }
      currentNote.value = note
      return true
    } catch (e) {
      if (loadId !== activeLoadId) return false
      error.value = String(e)
      currentNote.value = null
      scrollTop.value = 0
      return false
    } finally {
      if (loadId === activeLoadId) {
        loading.value = false
      }
    }
  }

  async function saveScrollPosition(path: string, top: number) {
    try {
      const settings = await getSettings()
      await saveSettings({
        ...settings,
        workspace: {
          ...settings.workspace,
          scroll_positions: {
            ...(settings.workspace.scroll_positions ?? {}),
            [path]: Math.max(0, Math.round(top)),
          },
        },
      })
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  function clearNote() {
    activeLoadId += 1
    currentNote.value = null
    scrollTop.value = 0
    loading.value = false
  }

  return {
    currentNote,
    loading,
    isLoading,
    error,
    wordCount,
    scrollTop,
    loadNote,
    saveScrollPosition,
    clearNote,
  }
})
