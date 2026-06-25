// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, expect, it, vi } from 'vitest'
import QuizResult from './QuizResult.vue'
import { useMistakeStore } from '@/stores/mistakes'
import { useQuizStore } from '@/stores/quiz'
import type { DiagnosisReport } from '@/types/diagnosis'

const firstReport: DiagnosisReport = {
  summary: 'First question diagnosis.',
  blind_spots: [],
  overall_level: 'Needs ownership review',
  next_steps: ['Review moves'],
}

const lastReport: DiagnosisReport = {
  summary: 'Last question diagnosis.',
  blind_spots: [],
  overall_level: 'Needs borrowing review',
  next_steps: ['Review borrows'],
}

describe('QuizResult', () => {
  it('saves advanced mistakes with the diagnosis context for that question', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const quizStore = useQuizStore()
    const mistakeStore = useMistakeStore()
    mistakeStore.saveEntry = vi.fn().mockResolvedValue(true)
    vi.stubGlobal('crypto', { randomUUID: () => 'mistake-1' })

    quizStore.questions = [
      {
        id: 'q1',
        question_type: 'single',
        question: 'Which ownership claim is true?',
        options: ['A. Values always copy', 'B. Moves transfer ownership'],
        answer: 'B',
        explanation: 'Non-Copy values move by default.',
      },
      {
        id: 'q2',
        question_type: 'single',
        question: 'Which borrow claim is true?',
        options: ['A. Shared borrows mutate freely', 'B. Mutable borrow is exclusive'],
        answer: 'B',
        explanation: 'A mutable borrow must be exclusive.',
      },
    ]
    quizStore.answers = new Map([
      ['q1', 'A'],
      ['q2', 'B'],
    ])
    quizStore.recordAdvancedContext('q1', 'I thought all values copy.', [
      { role: 'ai', content: 'Check Copy vs Move.', blind_spots: [] },
    ], firstReport)

    const wrapper = mount(QuizResult, {
      props: {
        mode: 'advanced',
        reasoning: 'last question reasoning',
        diagnosisReport: lastReport,
      },
      global: {
        plugins: [pinia],
      },
    })

    await wrapper.get('button.text-\\[11px\\]').trigger('click')

    expect(mistakeStore.saveEntry).toHaveBeenCalledWith(
      'q1',
      expect.objectContaining({
        user_reasoning: 'I thought all values copy.',
        diagnosis: {
          rounds: 1,
          conversation: [
            { role: 'ai', content: 'Check Copy vs Move.', blind_spots: [] },
          ],
          final_report: firstReport,
        },
      }),
    )
  })
})
