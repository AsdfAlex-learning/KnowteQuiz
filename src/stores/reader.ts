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

  const isLoading = computed(() => loading.value)
  const wordCount = computed(() => {
    if (!currentNote.value) return 0
    return currentNote.value.content.split(/\s+/).length
  })

  async function loadNote(path: string) {
    loading.value = true
    error.value = null
    try {
      const note = await readNote(path)
      try {
        const settings = await getSettings()
        scrollTop.value = settings.workspace.scroll_positions?.[path] ?? 0
      } catch {
        scrollTop.value = 0
      }
      currentNote.value = note
    } catch (e) {
      error.value = String(e)
      currentNote.value = null
      scrollTop.value = 0
    } finally {
      loading.value = false
    }
  }

  async function saveScrollPosition(path: string, top: number) {
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
  }

  function clearNote() {
    currentNote.value = null
    scrollTop.value = 0
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
