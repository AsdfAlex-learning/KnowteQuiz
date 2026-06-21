import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useSettingsStore } from './settings'
import * as settingsService from '../services/settings'

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  testConnection: vi.fn(),
  backupData: vi.fn(),
}))

describe('settings store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.stubGlobal('fetch', vi.fn(async () => ({ ok: false })))
  })

  it('delegates LLM connection testing to the settings service', async () => {
    vi.mocked(settingsService.testConnection).mockResolvedValue({
      ok: true,
      kind: 'ok',
      message: 'Connection successful',
      status: 200,
    })
    const store = useSettingsStore()

    const result = await store.testConnection()

    expect(settingsService.testConnection).toHaveBeenCalledOnce()
    expect(result.ok).toBe(true)
    expect(store.llmConnected).toBe(true)
    expect(store.llmConnectionResult?.message).toBe('Connection successful')
  })

  it('stores data backup results for the settings panel', async () => {
    vi.mocked(settingsService.backupData).mockResolvedValue({
      backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
      files: ['settings.json', 'mistakes.json'],
    })
    const store = useSettingsStore()

    const result = await store.backupDataNow()

    expect(settingsService.backupData).toHaveBeenCalledOnce()
    expect(result.files).toEqual(['settings.json', 'mistakes.json'])
    expect(store.lastBackupResult?.backup_dir).toContain('backups')
    expect(store.error).toBeNull()
  })
})
