import { describe, expect, it } from 'vitest'
import {
  canSubmitQuizAnswer,
  choiceLettersFromAnswer,
  isChoiceLetterCorrect,
  isQuizAnswerCorrect,
} from './answer'

describe('answer utilities', () => {
  it('extracts choice letters from common model answer formats', () => {
    expect(choiceLettersFromAnswer('A. Alpha')).toEqual(['A'])
    expect(choiceLettersFromAnswer('答案：B')).toEqual(['B'])
    expect(choiceLettersFromAnswer('A,C')).toEqual(['A', 'C'])
  })

  it('checks choice answers by letter instead of full option text', () => {
    expect(isChoiceLetterCorrect('A. Alpha', 'A')).toBe(true)
    expect(isChoiceLetterCorrect('A. Alpha', 'B')).toBe(false)
  })

  it('falls back to normalized text comparison for short answers', () => {
    expect(isQuizAnswerCorrect({
      id: 'q1',
      question_type: 'short',
      question: 'Name it.',
      options: [],
      answer: 'Dependency Injection',
      explanation: '',
    }, ' dependency   injection ')).toBe(true)
  })

  it('requires reasoning before submitting in advanced mode', () => {
    const question = {
      id: 'q1',
      question_type: 'single' as const,
      question: 'Pick one.',
      options: ['A. Alpha', 'B. Beta'],
      answer: 'A',
      explanation: '',
    }

    expect(canSubmitQuizAnswer(question, 0, '', 'advanced', '')).toBe(false)
    expect(canSubmitQuizAnswer(question, 0, '', 'advanced', 'I choose A because...')).toBe(true)
  })
})
