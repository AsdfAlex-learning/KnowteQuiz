import { defineStore } from 'pinia'
import { ref } from 'vue'
import { loadMistakes, saveMistake } from '../services/mistake'
import type { MistakeEntry, MistakeMode } from '../types/mistake'

const PAGE_SIZE = 20

export const useMistakeStore = defineStore('mistakes', () => {
  const items = ref<MistakeEntry[]>([])
  const loading = ref(false)
  const listError = ref<string | null>(null)
  const modeFilter = ref<MistakeMode | undefined>(undefined)
  const hasMore = ref(false)
  const savingIds = ref(new Set<string>())
  const savedIds = ref(new Set<string>())
  const errors = ref(new Map<string, string>())

  function isSaving(key: string): boolean {
    return savingIds.value.has(key)
  }

  function isSaved(key: string): boolean {
    return savedIds.value.has(key)
  }

  function errorFor(key: string): string | null {
    return errors.value.get(key) ?? null
  }

  function clearSaveState(): void {
    savingIds.value = new Set()
    savedIds.value = new Set()
    errors.value = new Map()
  }

  async function saveEntry(key: string, entry: MistakeEntry): Promise<boolean> {
    if (isSaving(key) || isSaved(key)) return false

    savingIds.value.add(key)
    errors.value.delete(key)
    try {
      const saved = await saveMistake(entry)
      if (!saved) {
        errors.value.set(key, 'Mistake was not saved')
        return false
      }
      savedIds.value.add(key)
      return true
    } catch (e) {
      errors.value.set(key, e instanceof Error ? e.message : String(e))
      return false
    } finally {
      savingIds.value.delete(key)
    }
  }

  async function loadPage(offset = 0): Promise<void> {
    loading.value = true
    listError.value = null
    try {
      const page = await loadMistakes({
        mode: modeFilter.value,
        offset,
        limit: PAGE_SIZE,
      })
      items.value = offset === 0 ? page : [...items.value, ...page]
      hasMore.value = page.length === PAGE_SIZE
    } catch (e) {
      listError.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function loadNextPage(): Promise<void> {
    if (loading.value || !hasMore.value) return
    await loadPage(items.value.length)
  }

  async function setModeFilter(mode: MistakeMode | undefined): Promise<void> {
    modeFilter.value = mode
    await loadPage(0)
  }

  return {
    items,
    loading,
    listError,
    modeFilter,
    hasMore,
    savingIds,
    savedIds,
    errors,
    isSaving,
    isSaved,
    errorFor,
    clearSaveState,
    saveEntry,
    loadPage,
    loadNextPage,
    setModeFilter,
  }
})
