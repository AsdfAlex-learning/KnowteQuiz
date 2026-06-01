import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { NoteContent } from '../types/note'
import { readNote } from '../services/note'

export const useReaderStore = defineStore('reader', () => {
  const currentNote = ref<NoteContent | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const isLoading = computed(() => loading.value)
  const wordCount = computed(() => {
    if (!currentNote.value) return 0
    return currentNote.value.content.split(/\s+/).length
  })

  async function loadNote(path: string) {
    loading.value = true
    error.value = null
    try {
      currentNote.value = await readNote(path)
    } catch (e) {
      error.value = String(e)
      currentNote.value = null
    } finally {
      loading.value = false
    }
  }

  function clearNote() {
    currentNote.value = null
  }

  return { currentNote, loading, isLoading, error, wordCount, loadNote, clearNote }
})
