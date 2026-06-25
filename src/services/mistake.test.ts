import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { exportMistakes, loadMistakes, markMistakeReviewed } from './mistake'
import type { MistakeEntry } from '../types/mistake'

function mockMistake(id = 'm1'): MistakeEntry {
  return {
    id,
    note_path: '/notes/rust.md',
    note_title: 'Rust Ownership',
    question: 'Which statement is correct?',
    user_answer: 'A',
    correct_answer: 'B',
    explanation: 'B is correct.',
    mode: 'basic',
    created_at: '2026-01-01T00:00:00.000Z',
    review_count: 0,
  }
}

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
      search_text: 'owner',
      blind_spot_tag: 'ownership transfer',
      offset: 20,
      limit: 10,
    })

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/mistakes?mode=advanced&note_path=%2Fnotes%2Frust.md&search_text=owner&blind_spot_tag=ownership+transfer&offset=20&limit=10',
    )
  })

  it('includes response text when loading mistakes fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 500,
      text: async () => 'Failed to parse mistakes.json',
    })))

    await expect(loadMistakes()).rejects.toThrow('Failed to parse mistakes.json')
  })

  it('marks a mistake reviewed through the web endpoint', async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => true,
    }))
    vi.stubGlobal('fetch', fetchMock)

    const result = await markMistakeReviewed('m1')

    expect(result).toBe(true)
    expect(fetchMock).toHaveBeenCalledWith('/api/mistakes/review', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ mistake_id: 'm1' }),
    })
  })
})

describe('export mistakes service', () => {
  let createObjectURL: ReturnType<typeof vi.fn>
  let revokeObjectURL: ReturnType<typeof vi.fn>
  let appendChild: ReturnType<typeof vi.fn>
  let removeChild: ReturnType<typeof vi.fn>
  let clickSpy: ReturnType<typeof vi.fn>

  beforeEach(() => {
    createObjectURL = vi.fn(() => 'blob:mock')
    revokeObjectURL = vi.fn()
    clickSpy = vi.fn()
    appendChild = vi.fn()
    removeChild = vi.fn()

    vi.stubGlobal('URL', { ...URL, createObjectURL, revokeObjectURL })
    vi.stubGlobal('document', {
      createElement: vi.fn(() => ({
        href: '',
        download: '',
        click: clickSpy,
      })),
      body: {
        appendChild,
        removeChild,
      },
    })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('loads all mistakes without pagination when exporting as JSON', async () => {
    const fetchMock = vi.fn(async () => ({
      ok: true,
      json: async () => [mockMistake('m1'), mockMistake('m2')],
    }))
    vi.stubGlobal('fetch', fetchMock)

    await exportMistakes('json')

    expect(fetchMock).toHaveBeenCalledWith('/api/mistakes')
    expect(createObjectURL).toHaveBeenCalled()
    const blob: Blob = createObjectURL.mock.calls[0][0]
    const content = await blob.text()
    const parsed = JSON.parse(content)
    expect(parsed).toHaveLength(2)
    expect(parsed[0].id).toBe('m1')
    expect(parsed[1].id).toBe('m2')
  })

  it('triggers a browser download with the correct JSON filename', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      json: async () => [mockMistake()],
    })))
    vi.useFakeTimers()
    vi.setSystemTime(new Date('2026-06-21'))

    await exportMistakes('json')

    expect(clickSpy).toHaveBeenCalled()
    vi.useRealTimers()
  })

  it('exports mistakes as formatted Markdown', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      json: async () => [mockMistake('m1')],
    })))

    await exportMistakes('markdown')

    const blob: Blob = createObjectURL.mock.calls[0][0]
    const content = await blob.text()
    expect(content).toContain('# KnowteQuiz Mistake Export')
    expect(content).toContain('## 1. Which statement is correct?')
    expect(content).toContain('- **Note**: `/notes/rust.md`')
  })

  it('includes diagnosis level in markdown export for advanced mistakes', async () => {
    const adv = mockMistake('adv')
    adv.mode = 'advanced'
    adv.user_reasoning = 'I thought A was right because...'
    adv.diagnosis = {
      rounds: 2,
      conversation: [],
      final_report: {
        summary: '',
        blind_spots: [],
        overall_level: 'Intermediate',
        next_steps: [],
      },
    }
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      json: async () => [adv],
    })))

    await exportMistakes('markdown')

    const blob: Blob = createObjectURL.mock.calls[0][0]
    const content = await blob.text()
    expect(content).toContain('- **Your Reasoning**: I thought A was right because...')
    expect(content).toContain('- **Diagnosis Level**: Intermediate')
  })

  it('rejects with an error when there are no mistakes to export', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      json: async () => [],
    })))

    await expect(exportMistakes('json')).rejects.toThrow('No mistakes to export')
    expect(createObjectURL).not.toHaveBeenCalled()
  })

  it('propagates load errors when exporting fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 500,
      text: async () => 'Corrupt mistakes file',
    })))

    await expect(exportMistakes('json')).rejects.toThrow('Corrupt mistakes file')
  })
})
