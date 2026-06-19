import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import type { Settings } from '../types/settings'
import { getSettings, saveSettings, testConnection as testSettingsConnection } from '../services/settings'
import { defaultSettings } from '../utils/defaults'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>(defaultSettings())
  const loading = ref(false)
  const error = ref<string | null>(null)
  const llmConnected = ref(false)

  async function loadSettings() {
    loading.value = true
    error.value = null
    try {
      settings.value = await getSettings()
    } catch (e) {
      error.value = String(e)
      settings.value = defaultSettings()
    } finally {
      loading.value = false
    }
  }

  async function persistSettings() {
    loading.value = true
    error.value = null
    try {
      await saveSettings(settings.value)
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function testConnection(): Promise<boolean> {
    try {
      llmConnected.value = await testSettingsConnection()
      return llmConnected.value
    } catch {
      llmConnected.value = false
      return false
    }
  }

  return { settings, loading, error, llmConnected, loadSettings, persistSettings, testConnection }
})
