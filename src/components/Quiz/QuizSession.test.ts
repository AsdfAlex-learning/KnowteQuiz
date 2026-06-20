// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, expect, it } from 'vitest'
import QuizSession from './QuizSession.vue'
import { useQuizStore } from '@/stores/quiz'

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
})
