import { defineStore } from 'pinia';
import { ref } from 'vue';
import type {
  ConnectionTestResult,
  DataBackupResult,
  DataRestoreResult,
  DataStatus,
  LlmConfig,
  Settings,
} from '../types/settings';
import {
  backupData,
  getDataStatus,
  getSettings,
  openDataDir,
  probeLlm,
  restoreLatestBackup,
  saveMediaBytes,
  saveSettings,
  testConnection as testSettingsConnection,
} from '../services/settings';
import { cleanupSessions } from '../services/quiz';
import { updateStreak } from '../utils/streak';
import { defaultSettings } from '../utils/defaults';

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>(defaultSettings());
  const loading = ref(false);
  const error = ref<string | null>(null);
  const llmConnected = ref(false);
  const llmConnectionResult = ref<ConnectionTestResult | null>(null);
  const lastBackupResult = ref<DataBackupResult | null>(null);
  const lastRestoreResult = ref<DataRestoreResult | null>(null);
  const dataStatus = ref<DataStatus | null>(null);
  const dataStatusError = ref<string | null>(null);

  async function loadSettings() {
    loading.value = true;
    error.value = null;
    try {
      settings.value = await getSettings();
      void migrateLegacyThemeMedia();
    } catch (e) {
      error.value = String(e);
      settings.value = defaultSettings();
    } finally {
      loading.value = false;
    }
  }

  // Legacy settings embedded theme media as base64 data URLs (up to ~67 MB
  // inside settings.json). Move them into the media dir once and store the
  // file path instead; failures keep the data URL and retry on next load.
  async function migrateLegacyThemeMedia(): Promise<void> {
    const theme = settings.value.theme_config;
    if (!theme) return;
    try {
      const next = { ...theme };
      let changed = false;
      for (const key of ['background_image', 'background_video'] as const) {
        const value = next[key];
        if (!value || !value.startsWith('data:')) continue;
        const [head, base64] = value.split(',');
        const mime = head.slice(5).split(';')[0] ?? '';
        const extension = (mime.split('/')[1] ?? 'bin').toLowerCase();
        const kind = key === 'background_video' ? 'video' : 'image';
        const saved = await saveMediaBytes(
          Uint8Array.from(atob(base64 ?? ''), (c) => c.charCodeAt(0)),
          extension,
          kind
        );
        next[key] = saved.path;
        changed = true;
      }
      if (changed) {
        await queueUpdate((s) => {
          s.theme_config = next;
        });
      }
    } catch {
      // keep the data URLs; migration is retried on the next settings load
    }
  }

  async function persistSettings() {
    loading.value = true;
    error.value = null;
    try {
      await saveSettings(settings.value);
    } catch (e) {
      error.value = String(e);
      // Callers that care (queueUpdate) need the failure; fire-and-forget
      // callers wrap queueUpdate in their own catch.
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // Serializes read-modify-write cycles on the single reactive settings
  // object. Concurrent persisters (scroll positions, layout, workspace,
  // streaks, locale) used to snapshot the file independently and clobber
  // each other with stale whole-file writes; every mutation now lands on
  // the shared object and is saved in enqueue order. Rejects when the
  // underlying save fails so callers can surface the error.
  let writeQueue: Promise<void> = Promise.resolve();

  function queueUpdate(mutator: (s: Settings) => void): Promise<void> {
    const task = writeQueue.then(() => {
      mutator(settings.value);
      return persistSettings();
    });
    writeQueue = task.catch(() => undefined);
    return task;
  }

  function recordActivity() {
    void queueUpdate((s) => updateStreak(s.workspace)).catch(() => undefined);
  }

  async function testConnection(llm?: LlmConfig): Promise<ConnectionTestResult> {
    try {
      const result = await testSettingsConnection(llm);
      llmConnectionResult.value = result;
      llmConnected.value = result.ok;
      return result;
    } catch (e) {
      const result: ConnectionTestResult = {
        ok: false,
        kind: 'network',
        message: String(e),
        status: null,
      };
      llmConnectionResult.value = result;
      llmConnected.value = false;
      return result;
    }
  }

  async function backupDataNow(): Promise<DataBackupResult> {
    loading.value = true;
    error.value = null;
    try {
      const result = await backupData();
      lastBackupResult.value = result;
      await loadDataStatus();
      return result;
    } catch (e) {
      error.value = String(e);
      lastBackupResult.value = null;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadDataStatus(): Promise<void> {
    dataStatusError.value = null;
    try {
      dataStatus.value = await getDataStatus();
    } catch (e) {
      dataStatus.value = null;
      dataStatusError.value = String(e);
    }
  }

  async function restoreLatestBackupNow(): Promise<DataRestoreResult> {
    loading.value = true;
    error.value = null;
    try {
      const result = await restoreLatestBackup();
      lastRestoreResult.value = result;
      await loadDataStatus();
      return result;
    } catch (e) {
      error.value = String(e);
      lastRestoreResult.value = null;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  const openDirErr = ref<string | null>(null);

  async function openDataDirNow(): Promise<void> {
    openDirErr.value = null;
    try {
      await openDataDir();
    } catch (e) {
      openDirErr.value = e instanceof Error ? e.message : String(e);
    }
  }

  const cleanupResult = ref<{ deleted_count: number; remaining_count: number } | null>(null);
  const cleanupErr = ref<string | null>(null);
  const isCleaningUp = ref(false);

  async function cleanupSessionsNow(): Promise<void> {
    isCleaningUp.value = true;
    cleanupErr.value = null;
    cleanupResult.value = null;
    try {
      const result = await cleanupSessions();
      cleanupResult.value = result;
    } catch (e) {
      cleanupErr.value = e instanceof Error ? e.message : String(e);
    } finally {
      isCleaningUp.value = false;
    }
  }

  const llmCapabilities = ref<{
    available_models: string[];
    supports_streaming: boolean;
    supports_response_format: boolean;
    default_model: string;
  } | null>(null);
  const probeErr = ref<string | null>(null);

  async function probeLlmNow(): Promise<void> {
    probeErr.value = null;
    llmCapabilities.value = null;
    try {
      const caps = await probeLlm();
      llmCapabilities.value = caps;
    } catch (e) {
      probeErr.value = e instanceof Error ? e.message : String(e);
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
    queueUpdate,
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
    recordActivity,
  };
});
