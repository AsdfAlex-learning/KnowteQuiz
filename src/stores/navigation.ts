import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useExplorerStore } from './explorer'
import { useReaderStore } from './reader'

export const useNavigationStore = defineStore('navigation', () => {
  const opening = ref(false)
  const error = ref<string | null>(null)

  async function openNote(path: string) {
    opening.value = true
    error.value = null
    const explorerStore = useExplorerStore()
    const readerStore = useReaderStore()
    try {
      await explorerStore.selectPath(path)
      await readerStore.loadNote(path)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      opening.value = false
    }
  }

  return {
    opening,
    error,
    openNote,
  }
})
