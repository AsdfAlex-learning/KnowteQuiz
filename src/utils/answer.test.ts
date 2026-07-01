import { describe, expect, it } from 'vitest'
import {
  canSubmitQuizAnswer,
  choiceLettersFromAnswer,
  correctChoiceLetters,
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

  it('scores a choice answer as correct when the answer is option text', () => {
    const question = {
      id: 'q1',
      question_type: 'single' as const,
      question: 'Pick one.',
      options: ['A. Alpha', 'B. Beta'],
      answer: 'Alpha',
      explanation: '',
    }

    expect(isQuizAnswerCorrect(question, 'A')).toBe(true)
  })

  it('maps option text answers back to their choice letters for highlighting', () => {
    const question = {
      id: 'q1',
      question_type: 'single' as const,
      question: 'Pick one.',
      options: ['A. Alpha', 'B. Beta'],
      answer: 'Alpha',
      explanation: '',
    }

    expect(correctChoiceLetters(question)).toEqual(['A'])
  })

  it('maps localized option text answers back to choice letters', () => {
    const question = {
      id: 'q1',
      question_type: 'multiple' as const,
      question: 'Pick all true claims.',
      options: ['A\uff1aAlpha', 'B\uff1aBeta', 'C\uff1aGamma'],
      answer: 'Alpha\u3001Gamma',
      explanation: '',
    }

    expect(correctChoiceLetters(question)).toEqual(['A', 'C'])
    expect(isQuizAnswerCorrect(question, 'C,A')).toBe(true)
  })

  it('maps option text answers joined by words back to choice letters', () => {
    const question = {
      id: 'q1',
      question_type: 'multiple' as const,
      question: 'Pick all true claims.',
      options: ['A. Alpha', 'B. Beta', 'C. Gamma'],
      answer: 'Alpha and Gamma',
      explanation: '',
    }

    expect(correctChoiceLetters(question)).toEqual(['A', 'C'])
    expect(isQuizAnswerCorrect(question, 'A,C')).toBe(true)
  })

  it('does not split a single option text answer that contains and', () => {
    const question = {
      id: 'q1',
      question_type: 'single' as const,
      question: 'Pick one.',
      options: ['A. Research and Development', 'B. Operations'],
      answer: 'Research and Development',
      explanation: '',
    }

    expect(correctChoiceLetters(question)).toEqual(['A'])
    expect(isQuizAnswerCorrect(question, 'A')).toBe(true)
  })

  it('does not split a single option text answer that contains a comma', () => {
    const question = {
      id: 'q1',
      question_type: 'single' as const,
      question: 'Pick one.',
      options: ['A. Research, Development', 'B. Operations'],
      answer: 'Research, Development',
      explanation: '',
    }

    expect(correctChoiceLetters(question)).toEqual(['A'])
    expect(isQuizAnswerCorrect(question, 'A')).toBe(true)
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

  it('checks multiple choice answers as an unordered letter set', () => {
    const question = {
      id: 'q1',
      question_type: 'multiple' as const,
      question: 'Pick all true claims.',
      options: ['A. Alpha', 'B. Beta', 'C. Gamma'],
      answer: 'A,C',
      explanation: '',
    }

    expect(isQuizAnswerCorrect(question, 'C,A')).toBe(true)
    expect(isQuizAnswerCorrect(question, 'A')).toBe(false)
  })

  it('scores multiple choice answers when the answer lists option text', () => {
    const question = {
      id: 'q1',
      question_type: 'multiple' as const,
      question: 'Pick all true claims.',
      options: ['A. Alpha', 'B. Beta', 'C. Gamma'],
      answer: 'Alpha, Gamma',
      explanation: '',
    }

    expect(isQuizAnswerCorrect(question, 'C,A')).toBe(true)
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

    expect(canSubmitQuizAnswer(question, [0], '', 'advanced', '')).toBe(false)
    expect(canSubmitQuizAnswer(question, [0], '', 'advanced', 'I choose A because...')).toBe(true)
  })

  it('requires at least one selected option for multiple choice questions', () => {
    const question = {
      id: 'q1',
      question_type: 'multiple' as const,
      question: 'Pick all true claims.',
      options: ['A. Alpha', 'B. Beta'],
      answer: 'A,B',
      explanation: '',
    }

    expect(canSubmitQuizAnswer(question, [], '', 'basic', '')).toBe(false)
    expect(canSubmitQuizAnswer(question, [0, 1], '', 'basic', '')).toBe(true)
  })
})
