import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ConnectionTestResult, DataBackupResult, Settings } from '../types/settings'
import { backupData, getSettings, saveSettings, testConnection as testSettingsConnection } from '../services/settings'
import { defaultSettings } from '../utils/defaults'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>(defaultSettings())
  const loading = ref(false)
  const error = ref<string | null>(null)
  const llmConnected = ref(false)
  const llmConnectionResult = ref<ConnectionTestResult | null>(null)
  const lastBackupResult = ref<DataBackupResult | null>(null)

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

  async function testConnection(): Promise<ConnectionTestResult> {
    try {
      const result = await testSettingsConnection()
      llmConnectionResult.value = result
      llmConnected.value = result.ok
      return result
    } catch (e) {
      const result: ConnectionTestResult = {
        ok: false,
        kind: 'network',
        message: String(e),
        status: null,
      }
      llmConnectionResult.value = result
      llmConnected.value = false
      return result
    }
  }

  async function backupDataNow(): Promise<DataBackupResult> {
    loading.value = true
    error.value = null
    try {
      const result = await backupData()
      lastBackupResult.value = result
      return result
    } catch (e) {
      error.value = String(e)
      lastBackupResult.value = null
      throw e
    } finally {
      loading.value = false
    }
  }

  return {
    settings,
    loading,
    error,
    llmConnected,
    llmConnectionResult,
    lastBackupResult,
    loadSettings,
    persistSettings,
    testConnection,
    backupDataNow,
  }
})
