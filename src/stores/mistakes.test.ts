import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useMistakeStore } from './mistakes'
import { loadMistakes, saveMistake } from '../services/mistake'
import type { MistakeEntry, MistakeMode } from '../types/mistake'

vi.mock('../services/mistake', () => ({
  loadMistakes: vi.fn(),
  saveMistake: vi.fn(),
}))

function mistake(id = 'm1', mode: MistakeMode = 'basic'): MistakeEntry {
  return {
    id,
    note_path: '/notes/rust.md',
    note_title: 'Rust',
    question: 'Which answer is correct?',
    user_answer: 'A',
    correct_answer: 'B',
    explanation: 'B is correct.',
    mode,
    created_at: '2026-01-01T00:00:00.000Z',
    review_count: 0,
  }
}

describe('mistake store save state', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('prevents duplicate saves while a mistake is already saving', async () => {
    let resolveSave!: (value: boolean) => void
    vi.mocked(saveMistake).mockReturnValue(new Promise((resolve) => {
      resolveSave = resolve
    }))
    const store = useMistakeStore()
    const entry = mistake()

    const first = store.saveEntry('q1', entry)
    const second = await store.saveEntry('q1', entry)
    resolveSave(true)
    const firstResult = await first

    expect(second).toBe(false)
    expect(firstResult).toBe(true)
    expect(saveMistake).toHaveBeenCalledTimes(1)
    expect(store.isSaved('q1')).toBe(true)
    expect(store.isSaving('q1')).toBe(false)
  })

  it('records save errors without marking the mistake as saved', async () => {
    vi.mocked(saveMistake).mockRejectedValue(new Error('disk full'))
    const store = useMistakeStore()

    const result = await store.saveEntry('q1', mistake())

    expect(result).toBe(false)
    expect(store.isSaved('q1')).toBe(false)
    expect(store.isSaving('q1')).toBe(false)
    expect(store.errorFor('q1')).toContain('disk full')
  })

  it('can clear per-quiz save state so reused question ids can be saved again', async () => {
    vi.mocked(saveMistake).mockResolvedValue(true)
    const store = useMistakeStore()

    await store.saveEntry('q1', mistake('m1'))
    store.clearSaveState()
    const result = await store.saveEntry('q1', mistake('m2'))

    expect(result).toBe(true)
    expect(saveMistake).toHaveBeenCalledTimes(2)
  })

  it('loads the first page with server-side mode filters', async () => {
    vi.mocked(loadMistakes).mockResolvedValue([mistake('m2', 'advanced')])
    const store = useMistakeStore()

    await store.setModeFilter('advanced')

    expect(loadMistakes).toHaveBeenCalledWith({
      mode: 'advanced',
      offset: 0,
      limit: 20,
    })
    expect(store.items.map((item) => item.id)).toEqual(['m2'])
    expect(store.hasMore).toBe(false)
  })

  it('loads additional pages by appending mistakes with the next offset', async () => {
    vi.mocked(loadMistakes)
      .mockResolvedValueOnce(Array.from({ length: 20 }, (_, i) => mistake(`m${i + 1}`)))
      .mockResolvedValueOnce([mistake('m21')])
    const store = useMistakeStore()

    await store.loadPage()
    await store.loadNextPage()

    expect(loadMistakes).toHaveBeenNthCalledWith(1, {
      mode: undefined,
      offset: 0,
      limit: 20,
    })
    expect(loadMistakes).toHaveBeenNthCalledWith(2, {
      mode: undefined,
      offset: 20,
      limit: 20,
    })
    expect(store.items).toHaveLength(21)
    expect(store.hasMore).toBe(false)
  })
})
