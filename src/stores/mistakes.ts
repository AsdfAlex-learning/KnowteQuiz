import { defineStore } from 'pinia'
import { ref } from 'vue'
import { saveMistake } from '../services/mistake'
import type { MistakeEntry } from '../types/mistake'

export const useMistakeStore = defineStore('mistakes', () => {
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

  return {
    savingIds,
    savedIds,
    errors,
    isSaving,
    isSaved,
    errorFor,
    saveEntry,
  }
})
