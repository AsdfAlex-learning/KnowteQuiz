import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { ConnectionTestResult, DataBackupResult, DataRestoreResult, DataStatus, Settings } from '../types/settings'
import {
  backupData,
  getDataStatus,
  getSettings,
  openDataDir,
  probeLlm,
  restoreLatestBackup,
  saveSettings,
  testConnection as testSettingsConnection,
} from '../services/settings'
import { cleanupSessions } from '../services/quiz'
import { defaultSettings } from '../utils/defaults'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>(defaultSettings())
  const loading = ref(false)
  const error = ref<string | null>(null)
  const llmConnected = ref(false)
  const llmConnectionResult = ref<ConnectionTestResult | null>(null)
  const lastBackupResult = ref<DataBackupResult | null>(null)
  const lastRestoreResult = ref<DataRestoreResult | null>(null)
  const dataStatus = ref<DataStatus | null>(null)
  const dataStatusError = ref<string | null>(null)

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
      await loadDataStatus()
      return result
    } catch (e) {
      error.value = String(e)
      lastBackupResult.value = null
      throw e
    } finally {
      loading.value = false
    }
  }

  async function loadDataStatus(): Promise<void> {
    dataStatusError.value = null
    try {
      dataStatus.value = await getDataStatus()
    } catch (e) {
      dataStatus.value = null
      dataStatusError.value = String(e)
    }
  }

  async function restoreLatestBackupNow(): Promise<DataRestoreResult> {
    loading.value = true
    error.value = null
    try {
      const result = await restoreLatestBackup()
      lastRestoreResult.value = result
      await loadDataStatus()
      return result
    } catch (e) {
      error.value = String(e)
      lastRestoreResult.value = null
      throw e
    } finally {
      loading.value = false
    }
  }

  const openDirErr = ref<string | null>(null)

  async function openDataDirNow(): Promise<void> {
    openDirErr.value = null
    try {
      await openDataDir()
    } catch (e) {
      openDirErr.value = e instanceof Error ? e.message : String(e)
    }
  }

  const cleanupResult = ref<{ deleted_count: number; remaining_count: number } | null>(null)
  const cleanupErr = ref<string | null>(null)
  const isCleaningUp = ref(false)

  async function cleanupSessionsNow(): Promise<void> {
    isCleaningUp.value = true
    cleanupErr.value = null
    cleanupResult.value = null
    try {
      const result = await cleanupSessions()
      cleanupResult.value = result
    } catch (e) {
      cleanupErr.value = e instanceof Error ? e.message : String(e)
    } finally {
      isCleaningUp.value = false
    }
  }

  const llmCapabilities = ref<{ available_models: string[]; supports_streaming: boolean; supports_response_format: boolean; default_model: string } | null>(null)
  const probeErr = ref<string | null>(null)

  async function probeLlmNow(): Promise<void> {
    probeErr.value = null
    llmCapabilities.value = null
    try {
      const caps = await probeLlm()
      llmCapabilities.value = caps
    } catch (e) {
      probeErr.value = e instanceof Error ? e.message : String(e)
    }
  }

  return {
    settings,
    loading,
    error,
    llmConnected,
    llmConnectionResult,
    lastBackupResult,
    lastRestoreResult,
    dataStatus,
    dataStatusError,
    loadSettings,
    persistSettings,
    testConnection,
    backupDataNow,
    loadDataStatus,
    restoreLatestBackupNow,
    openDataDirNow,
    openDirErr,
    cleanupSessionsNow,
    cleanupResult,
    cleanupErr,
    isCleaningUp,
    probeLlmNow,
    llmCapabilities,
    probeErr,
  }
})
