import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useMistakeStore } from './mistakes'
import { saveMistake } from '../services/mistake'
import type { MistakeEntry } from '../types/mistake'

vi.mock('../services/mistake', () => ({
  saveMistake: vi.fn(),
}))

function mistake(id = 'm1'): MistakeEntry {
  return {
    id,
    note_path: '/notes/rust.md',
    note_title: 'Rust',
    question: 'Which answer is correct?',
    user_answer: 'A',
    correct_answer: 'B',
    explanation: 'B is correct.',
    mode: 'basic',
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
})
