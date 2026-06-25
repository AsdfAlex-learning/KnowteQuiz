// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, expect, it } from 'vitest'
import QuizSession from './QuizSession.vue'
import { useQuizStore } from '@/stores/quiz'
import type { DiagnosisReport } from '@/types/diagnosis'

const report: DiagnosisReport = {
  summary: 'You confused the ownership transfer rule.',
  blind_spots: [],
  overall_level: 'Needs targeted review',
  next_steps: ['Re-read the move semantics section'],
}

describe('QuizSession', () => {
  it('shows diagnosis errors inside an active quiz session', () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const quizStore = useQuizStore()
    quizStore.questions = [{
      id: 'q1',
      question_type: 'single',
      question: 'Which claim is true?',
      options: ['A. Alpha', 'B. Beta'],
      answer: 'A',
      explanation: 'Alpha is true.',
    }]
    quizStore.quizState = 'diagnosing'
    quizStore.generatingError = 'Session session-1 not found'

    const wrapper = mount(QuizSession, {
      props: { mode: 'advanced' },
      global: {
        plugins: [pinia],
      },
    })

    expect(wrapper.text()).toContain('Session session-1 not found')
  })

  it('keeps the final diagnosis report when moving from the last report to results', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const quizStore = useQuizStore()
    quizStore.questions = [{
      id: 'q1',
      question_type: 'single',
      question: 'Which ownership claim is true?',
      options: ['A. Values always copy', 'B. Moves transfer ownership'],
      answer: 'B',
      explanation: 'Ownership moves by default for non-Copy values.',
    }]
    quizStore.answers = new Map([['q1', 'A']])
    quizStore.quizState = 'report'
    quizStore.diagnosisReport = report

    const wrapper = mount(QuizSession, {
      props: { mode: 'advanced' },
      global: {
        plugins: [pinia],
      },
    })

    await wrapper.get('button:last-of-type').trigger('click')

    expect(quizStore.showResults).toBe(true)
    expect(quizStore.diagnosisReport).toEqual(report)
  })
})
