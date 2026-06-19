import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useSettingsStore } from './settings'
import * as settingsService from '../services/settings'

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  testConnection: vi.fn(),
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
})
