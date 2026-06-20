import { afterEach, describe, expect, it, vi } from 'vitest'
import { readNote, scanNotes } from './note'

describe('note service', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('includes the response body when web note scanning fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 404,
      text: async () => 'Directory does not exist: D:/missing',
    })))

    await expect(scanNotes('D:/missing')).rejects.toThrow('HTTP 404: Directory does not exist: D:/missing')
  })

  it('includes the response body when web note reading fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 500,
      text: async () => 'Failed to read file D:/notes/a.md',
    })))

    await expect(readNote('D:/notes/a.md')).rejects.toThrow('HTTP 500: Failed to read file D:/notes/a.md')
  })
})
