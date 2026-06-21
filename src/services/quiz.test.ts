import { afterEach, describe, expect, it, vi } from 'vitest'
import { generateDiagnosisReport, generateQuiz, submitAnswerAdvanced } from './quiz'
import { webStream } from './tauri'
import type { QuizStreamParams } from '../types/quiz'

vi.mock('./tauri', () => ({
  isTauri: () => false,
  invoke: vi.fn(),
  webStream: vi.fn(),
}))

describe('quiz service', () => {
  afterEach(() => {
    vi.clearAllMocks()
    vi.unstubAllGlobals()
  })

  it('sends quiz generation params directly to the web endpoint', async () => {
    const params: QuizStreamParams = {
      path: '/notes/rust.md',
      types: ['single'],
      count: 2,
      difficulty: 'medium',
      lang: 'zh',
    }
    vi.mocked(webStream).mockResolvedValue(undefined)

    await generateQuiz(params, vi.fn(), vi.fn(), vi.fn(), vi.fn())

    expect(webStream).toHaveBeenCalledWith(
      '/api/quiz/generate',
      params,
      expect.any(Function),
    )
  })

  it('sends the known correct answer to the web diagnosis endpoint', async () => {
    vi.mocked(webStream).mockResolvedValue(undefined)

    await submitAnswerAdvanced(
      'Which claim is true?',
      'B',
      'A',
      'I guessed.',
      '/notes/rust.md',
      vi.fn(),
      vi.fn(),
      vi.fn(),
      vi.fn(),
    )

    expect(webStream).toHaveBeenCalledWith(
      '/api/quiz/diagnose',
      expect.objectContaining({
        question: 'Which claim is true?',
        correct_answer: 'B',
        user_answer: 'A',
      }),
      expect.any(Function),
    )
  })

  it('includes the response body when web diagnosis report generation fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 404,
      text: async () => 'Session session-1 not found',
    })))

    await expect(generateDiagnosisReport('session-1')).rejects.toThrow('HTTP 404: Session session-1 not found')
  })
})
