import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useQuizStore } from './quiz'
import { diagnoseFollowUp, generateDiagnosisReport, generateQuiz, submitAnswerAdvanced } from '../services/quiz'
import type { DiagnosisReport } from '../types/diagnosis'

vi.mock('../services/quiz', () => ({
  generateQuiz: vi.fn(),
  submitAnswerAdvanced: vi.fn(),
  diagnoseFollowUp: vi.fn(),
  generateDiagnosisReport: vi.fn(),
}))

const report: DiagnosisReport = {
  summary: 'Review concept boundaries.',
  blind_spots: [],
  overall_level: 'Needs review',
  next_steps: ['Re-read the note'],
}

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

  it('scores a choice answer as correct when the model answer is only option text', () => {
    const store = useQuizStore()
    store.addQuestion({
      id: 'q1',
      question_type: 'single',
      question: 'Which letter is first?',
      options: ['A. Alpha', 'B. Beta'],
      answer: 'Alpha',
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

  it('returns to a retryable state when generation completes without questions', async () => {
    vi.mocked(generateQuiz).mockImplementation(async (_params, _onChunk, onDone) => {
      onDone(0)
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
    expect(store.generatingError).toContain('No questions generated')
  })

  it('stores advanced diagnosis messages, session id, and generated report', async () => {
    vi.mocked(submitAnswerAdvanced).mockImplementation(async (_question, _correctAnswer, _answer, _reasoning, _notePath, onInitial, _onFollowUp, onReport) => {
      onInitial({
        role: 'ai',
        content: 'Your reasoning skipped the definition.',
        blind_spots: [],
        follow_up: 'Which definition applies here?',
      })
      onReport(report)
      return 'session-1'
    })
    const store = useQuizStore()

    await store.startDiagnosis('Question?', 'B', 'A', 'Because A', '/notes/rust.md')

    expect(store.sessionId).toBe('session-1')
    expect(store.diagnosisMessages).toEqual([
      {
        role: 'ai',
        content: 'Your reasoning skipped the definition.',
        blind_spots: [],
        follow_up: 'Which definition applies here?',
      },
    ])
    expect(store.diagnosisReport).toEqual(report)
    expect(store.quizState).toBe('report')
  })

  it('returns to answering when initial diagnosis streaming reports an error', async () => {
    vi.mocked(submitAnswerAdvanced).mockImplementation(async (_question, _correctAnswer, _answer, _reasoning, _notePath, _onInitial, _onFollowUp, _onReport, onError) => {
      onError('Failed to parse diagnosis: missing answer_analysis')
      return 'session-1'
    })
    const store = useQuizStore()

    await store.startDiagnosis('Question?', 'B', 'A', 'Because A', '/notes/rust.md')

    expect(store.quizState).toBe('answering')
    expect(store.sessionId).toBeNull()
    expect(store.generatingError).toContain('missing answer_analysis')
  })

  it('generates a diagnosis report from the current session', async () => {
    vi.mocked(generateDiagnosisReport).mockResolvedValue(report)
    const store = useQuizStore()
    store.sessionId = 'session-1'
    store.quizState = 'diagnosing'

    await store.finishDiagnosis()

    expect(generateDiagnosisReport).toHaveBeenCalledWith('session-1')
    expect(store.diagnosisReport).toEqual(report)
    expect(store.quizState).toBe('report')
  })

  it('stores user replies and follow-up questions during diagnosis', async () => {
    vi.mocked(diagnoseFollowUp).mockImplementation(async (_sessionId, userReply, onFollowUp) => {
      onFollowUp({
        question: 'How does the definition change your answer?',
        blind_spots: [],
      })
    })
    const store = useQuizStore()
    store.sessionId = 'session-1'
    store.quizState = 'diagnosing'

    await store.continueDiagnosis('I confused two definitions.')

    expect(diagnoseFollowUp).toHaveBeenCalledWith(
      'session-1',
      'I confused two definitions.',
      expect.any(Function),
      expect.any(Function),
      expect.any(Function),
    )
    expect(store.diagnosisMessages).toEqual([
      { role: 'user', content: 'I confused two definitions.', blind_spots: [] },
      {
        role: 'ai',
        content: 'How does the definition change your answer?',
        blind_spots: [],
        follow_up: 'How does the definition change your answer?',
      },
    ])
  })

  it('records follow-up errors without appending failed replies', async () => {
    vi.mocked(diagnoseFollowUp).mockImplementation(async (_sessionId, _userReply, _onFollowUp, _onReport, onError) => {
      onError('Session session-1 not found')
    })
    const store = useQuizStore()
    store.sessionId = 'session-1'
    store.quizState = 'diagnosing'
    store.diagnosisMessages = [
      { role: 'ai', content: 'Which definition applies?', blind_spots: [], follow_up: 'Which definition applies?' },
    ]

    await store.continueDiagnosis('My reply')

    expect(store.quizState).toBe('diagnosing')
    expect(store.generatingError).toContain('Session session-1 not found')
    expect(store.diagnosisMessages).toEqual([
      { role: 'ai', content: 'Which definition applies?', blind_spots: [], follow_up: 'Which definition applies?' },
    ])
  })

  it('clears diagnosis state before moving to another question', () => {
    const store = useQuizStore()
    store.sessionId = 'session-1'
    store.diagnosisMessages = [{ role: 'ai', content: 'Old diagnosis', blind_spots: [] }]
    store.diagnosisReport = report
    store.quizState = 'report'

    store.clearDiagnosis()

    expect(store.sessionId).toBeNull()
    expect(store.diagnosisMessages).toEqual([])
    expect(store.diagnosisReport).toBeNull()
    expect(store.quizState).toBe('answering')
  })
})
