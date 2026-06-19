import { afterEach, describe, expect, it, vi } from 'vitest'
import { loadMistakes } from './mistake'

describe('mistake service', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('passes mistake filters as web query parameters', async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => [],
    }))
    vi.stubGlobal('fetch', fetchMock)

    await loadMistakes({
      mode: 'advanced',
      note_path: '/notes/rust.md',
      offset: 20,
      limit: 10,
    } as any)

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/mistakes?mode=advanced&note_path=%2Fnotes%2Frust.md&offset=20&limit=10',
    )
  })
})
