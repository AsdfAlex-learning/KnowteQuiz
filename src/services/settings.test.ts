import { afterEach, describe, expect, it, vi } from 'vitest'
import { testConnection } from './settings'

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
})
