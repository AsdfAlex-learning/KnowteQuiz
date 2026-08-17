import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useSettingsStore } from './settings';
import * as settingsService from '../services/settings';
import * as quizService from '../services/quiz';

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  testConnection: vi.fn(),
  backupData: vi.fn(),
  getDataStatus: vi.fn(),
  restoreLatestBackup: vi.fn(),
  openDataDir: vi.fn(),
}));

vi.mock('../services/quiz', () => ({
  cleanupSessions: vi.fn(),
}));

describe('settings store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({ ok: false }))
    );
  });

  it('delegates LLM connection testing to the settings service', async () => {
    vi.mocked(settingsService.testConnection).mockResolvedValue({
      ok: true,
      kind: 'ok',
      message: 'Connection successful',
      status: 200,
    });
    const store = useSettingsStore();

    const result = await store.testConnection();

    expect(settingsService.testConnection).toHaveBeenCalledOnce();
    expect(result.ok).toBe(true);
    expect(store.llmConnected).toBe(true);
    expect(store.llmConnectionResult?.message).toBe('Connection successful');
  });

  it('stores data backup results for the settings panel', async () => {
    vi.mocked(settingsService.backupData).mockResolvedValue({
      backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
      files: ['settings.json', 'mistakes.json'],
    });
    const store = useSettingsStore();

    const result = await store.backupDataNow();

    expect(settingsService.backupData).toHaveBeenCalledOnce();
    expect(result.files).toEqual(['settings.json', 'mistakes.json']);
    expect(store.lastBackupResult?.backup_dir).toContain('backups');
    expect(store.error).toBeNull();
  });

  it('loads data file status for the settings panel', async () => {
    vi.mocked(settingsService.getDataStatus).mockResolvedValue({
      data_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz',
      files: [
        {
          name: 'settings.json',
          exists: true,
          size_bytes: 2048,
          modified_at: '2026-06-21T12:00:00Z',
        },
      ],
    });
    const store = useSettingsStore();

    await store.loadDataStatus();

    expect(settingsService.getDataStatus).toHaveBeenCalledOnce();
    expect(store.dataStatus?.files[0].name).toBe('settings.json');
    expect(store.dataStatusError).toBeNull();
  });

  it('stores restore results for the settings panel', async () => {
    vi.mocked(settingsService.restoreLatestBackup).mockResolvedValue({
      backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
      pre_restore_backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-130000',
      files: ['settings.json', 'mistakes.json'],
    });
    const store = useSettingsStore();

    const result = await store.restoreLatestBackupNow();

    expect(settingsService.restoreLatestBackup).toHaveBeenCalledOnce();
    expect(result.files).toEqual(['settings.json', 'mistakes.json']);
    expect(store.lastRestoreResult?.backup_dir).toContain('20260621-120000');
    expect(store.error).toBeNull();
  });
});

describe('settings store open data directory', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({ ok: false }))
    );
  });

  it('calls openDataDir service and clears previous errors', async () => {
    vi.mocked(settingsService.openDataDir).mockResolvedValue('/path/to/data');
    const store = useSettingsStore();
    store.openDirErr = 'previous error';

    await store.openDataDirNow();

    expect(settingsService.openDataDir).toHaveBeenCalledOnce();
    expect(store.openDirErr).toBeNull();
  });

  it('captures errors from the openDataDir service', async () => {
    vi.mocked(settingsService.openDataDir).mockRejectedValue(new Error('Failed to open data directory: access denied'));
    const store = useSettingsStore();

    await store.openDataDirNow();

    expect(store.openDirErr).toBe('Failed to open data directory: access denied');
  });

  it('handles non-Error exceptions when opening data directory', async () => {
    vi.mocked(settingsService.openDataDir).mockRejectedValue('Permission denied');
    const store = useSettingsStore();

    await store.openDataDirNow();

    expect(store.openDirErr).toBe('Permission denied');
  });
});

describe('settings store session cleanup', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({ ok: false }))
    );
  });

  it('delegates session cleanup to the quiz service and stores the result', async () => {
    vi.mocked(quizService.cleanupSessions).mockResolvedValue({
      deleted_count: 3,
      remaining_count: 2,
    });
    const store = useSettingsStore();

    await store.cleanupSessionsNow();

    expect(quizService.cleanupSessions).toHaveBeenCalledOnce();
    expect(store.cleanupResult).toEqual({ deleted_count: 3, remaining_count: 2 });
    expect(store.cleanupErr).toBeNull();
  });

  it('captures errors from session cleanup', async () => {
    vi.mocked(quizService.cleanupSessions).mockRejectedValue(new Error('Failed to read sessions directory'));
    const store = useSettingsStore();

    await store.cleanupSessionsNow();

    expect(store.cleanupErr).toBe('Failed to read sessions directory');
    expect(store.cleanupResult).toBeNull();
  });

  it('sets isCleaningUp true during cleanup and false after completing', async () => {
    vi.mocked(quizService.cleanupSessions).mockResolvedValue({
      deleted_count: 0,
      remaining_count: 1,
    });
    const store = useSettingsStore();

    const promise = store.cleanupSessionsNow();
    expect(store.isCleaningUp).toBe(true);
    await promise;
    expect(store.isCleaningUp).toBe(false);
  });
});
