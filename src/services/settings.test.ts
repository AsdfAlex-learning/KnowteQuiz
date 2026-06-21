import { afterEach, describe, expect, it, vi } from 'vitest'
import { backupData, getSettings, saveSettings, testConnection } from './settings'

describe('settings service', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('returns structured web connection test results', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      json: async () => ({
        ok: false,
        kind: 'auth',
        message: 'LLM endpoint rejected the API key',
        status: 401,
      }),
    })))

    const result = await testConnection()

    expect(result).toEqual({
      ok: false,
      kind: 'auth',
      message: 'LLM endpoint rejected the API key',
      status: 401,
    })
  })

  it('includes the response body when loading web settings fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 500,
      text: async () => 'Failed to parse settings.json',
    })))

    await expect(getSettings()).rejects.toThrow('HTTP 500: Failed to parse settings.json')
  })

  it('includes the response body when saving web settings fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 400,
      text: async () => 'Invalid settings payload',
    })))

    await expect(saveSettings({} as never)).rejects.toThrow('HTTP 400: Invalid settings payload')
  })

  it('includes the response body when web connection testing fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 502,
      text: async () => 'LLM endpoint unavailable',
    })))

    await expect(testConnection()).rejects.toThrow('HTTP 502: LLM endpoint unavailable')
  })

  it('posts to the web data backup endpoint and returns the backup details', async () => {
    const fetch = vi.fn(async () => ({
      ok: true,
      json: async () => ({
        backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
        files: ['settings.json', 'mistakes.json'],
      }),
    }))
    vi.stubGlobal('fetch', fetch)

    const result = await backupData()

    expect(fetch).toHaveBeenCalledWith('/api/data/backup', { method: 'POST' })
    expect(result.files).toEqual(['settings.json', 'mistakes.json'])
    expect(result.backup_dir).toContain('backups')
  })
})
