import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useQuizStore } from './quiz'
import { generateQuiz } from '../services/quiz'

vi.mock('../services/quiz', () => ({
  generateQuiz: vi.fn(),
  submitAnswerAdvanced: vi.fn(),
  diagnoseFollowUp: vi.fn(),
}))

describe('quiz store answer evaluation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('scores a choice answer as correct when the model answer includes option text', () => {
    const store = useQuizStore()
    store.addQuestion({
      id: 'q1',
      question_type: 'single',
      question: 'Which letter is first?',
      options: ['A. Alpha', 'B. Beta'],
      answer: 'A. Alpha',
      explanation: 'A is the first letter.',
    })

    store.submitAnswer('q1', 'A')

    expect(store.score).toBe(100)
  })

  it('moves from generation to answering when quiz streaming completes', async () => {
    vi.mocked(generateQuiz).mockImplementation(async (_params, onChunk, onDone) => {
      onChunk({
        id: 'q1',
        question_type: 'single',
        question: 'Which letter is first?',
        options: ['A. Alpha', 'B. Beta'],
        answer: 'A',
        explanation: 'A is first.',
      })
      onDone(1)
    })
    const store = useQuizStore()

    await store.startQuiz({
      path: '/notes/alphabet.md',
      types: ['single'],
      count: 1,
      difficulty: 'easy',
      lang: 'en',
    })

    expect(store.quizState).toBe('answering')
    expect(store.questions).toHaveLength(1)
    expect(store.generatingError).toBeNull()
  })

  it('returns to a retryable state and clears partial questions when generation reports an error', async () => {
    vi.mocked(generateQuiz).mockImplementation(async (_params, onChunk, _onDone, onError) => {
      onChunk({
        id: 'q1',
        question_type: 'single',
        question: 'Partial question',
        options: ['A. Alpha'],
        answer: 'A',
        explanation: 'Partial.',
      })
      onError('Failed to parse quiz response: answer outside options')
    })
    const store = useQuizStore()

    await store.startQuiz({
      path: '/notes/alphabet.md',
      types: ['single'],
      count: 1,
      difficulty: 'easy',
      lang: 'en',
    })

    expect(store.quizState).toBe('idle')
    expect(store.hasQuestions).toBe(false)
    expect(store.generatingError).toContain('answer outside options')
  })
})
