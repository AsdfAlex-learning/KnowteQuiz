import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useQuizStore } from './quiz'

vi.mock('../services/quiz', () => ({
  generateQuiz: vi.fn(),
  submitAnswerAdvanced: vi.fn(),
  diagnoseFollowUp: vi.fn(),
}))

describe('quiz store answer evaluation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
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
})
