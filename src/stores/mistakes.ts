import { defineStore } from 'pinia'
import { ref } from 'vue'
import {
  exportMistakes as exportMistakesService,
  loadMistakes,
  markMistakeReviewed,
  saveMistake,
} from '../services/mistake'
import type { MistakeEntry, MistakeMode } from '../types/mistake'

const PAGE_SIZE = 20

export const useMistakeStore = defineStore('mistakes', () => {
  const items = ref<MistakeEntry[]>([])
  const loading = ref(false)
  const listError = ref<string | null>(null)
  const modeFilter = ref<MistakeMode | undefined>(undefined)
  const searchText = ref<string | undefined>(undefined)
  const blindSpotTag = ref<string | undefined>(undefined)
  const hasMore = ref(false)
  const savingIds = ref(new Set<string>())
  const savedIds = ref(new Set<string>())
  const errors = ref(new Map<string, string>())
  const reviewingIds = ref(new Set<string>())
  const reviewErrors = ref(new Map<string, string>())

  function isSaving(key: string): boolean {
    return savingIds.value.has(key)
  }

  function isSaved(key: string): boolean {
    return savedIds.value.has(key)
  }

  function errorFor(key: string): string | null {
    return errors.value.get(key) ?? null
  }

  function isReviewing(id: string): boolean {
    return reviewingIds.value.has(id)
  }

  function reviewErrorFor(id: string): string | null {
    return reviewErrors.value.get(id) ?? null
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
      if (!modeFilter.value || entry.mode === modeFilter.value) {
        items.value = [
          entry,
          ...items.value.filter((item) => item.id !== entry.id),
        ]
      }
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
        search_text: searchText.value,
        blind_spot_tag: blindSpotTag.value,
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

  async function setSearchText(text: string): Promise<void> {
    searchText.value = text.trim() || undefined
    await loadPage(0)
  }

  async function setBlindSpotTag(tag: string): Promise<void> {
    blindSpotTag.value = tag.trim() || undefined
    await loadPage(0)
  }

  async function markReviewed(mistakeId: string): Promise<boolean> {
    if (isReviewing(mistakeId)) return false

    reviewingIds.value.add(mistakeId)
    reviewErrors.value.delete(mistakeId)
    try {
      const reviewed = await markMistakeReviewed(mistakeId)
      if (!reviewed) {
        reviewErrors.value.set(mistakeId, 'Mistake was not marked reviewed')
        return false
      }

      const reviewedAt = new Date().toISOString()
      items.value = items.value.map((item) => {
        if (item.id !== mistakeId) return item
        return {
          ...item,
          review_count: item.review_count + 1,
          last_reviewed_at: reviewedAt,
        }
      })
      return true
    } catch (e) {
      reviewErrors.value.set(mistakeId, e instanceof Error ? e.message : String(e))
      return false
    } finally {
      reviewingIds.value.delete(mistakeId)
    }
  }

  const exportError = ref<string | null>(null)
  const isExporting = ref(false)

  async function exportMistakes(format: 'json' | 'markdown'): Promise<boolean> {
    isExporting.value = true
    exportError.value = null
    try {
      await exportMistakesService(format)
      return true
    } catch (e) {
      exportError.value = e instanceof Error ? e.message : String(e)
      return false
    } finally {
      isExporting.value = false
    }
  }

  return {
    items,
    loading,
    listError,
    modeFilter,
    hasMore,
    savingIds,
    savedIds,
    reviewingIds,
    errors,
    reviewErrors,
    isSaving,
    isSaved,
    errorFor,
    isReviewing,
    reviewErrorFor,
    clearSaveState,
    saveEntry,
    loadPage,
    loadNextPage,
    setModeFilter,
    setSearchText,
    setBlindSpotTag,
    markReviewed,
    searchText,
    blindSpotTag,
    exportMistakes,
    isExporting,
    exportError,
  }
})
