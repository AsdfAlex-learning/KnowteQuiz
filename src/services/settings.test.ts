import { afterEach, describe, expect, it, vi } from 'vitest'
import { getSettings, saveSettings, testConnection } from './settings'

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
})
