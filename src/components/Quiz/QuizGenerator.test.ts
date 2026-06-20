// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it, vi } from 'vitest'
import QuizGenerator from './QuizGenerator.vue'
import { useExplorerStore } from '@/stores/explorer'
import { useQuizStore } from '@/stores/quiz'
import { generateQuiz } from '@/services/quiz'

vi.mock('@/services/quiz', () => ({
  generateQuiz: vi.fn(),
  submitAnswerAdvanced: vi.fn(),
  diagnoseFollowUp: vi.fn(),
  generateDiagnosisReport: vi.fn(),
}))

describe('QuizGenerator', () => {
  afterEach(() => {
    vi.clearAllMocks()
  })

  it('starts an advanced quiz when advanced mode is selected', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const explorerStore = useExplorerStore()
    explorerStore.selectedPath = '/notes/rust.md'
    vi.mocked(generateQuiz).mockImplementation(async (_params, onChunk, onDone) => {
      onChunk({
        id: 'q1',
        question_type: 'single',
        question: 'Which claim is true?',
        options: ['A. Alpha', 'B. Beta'],
        answer: 'A',
        explanation: 'Alpha is true.',
      })
      onDone(1)
    })

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    })

    await wrapper.findAll('button').find((button) => button.text() === 'Advanced')?.trigger('click')
    await wrapper.findAll('button').find((button) => button.text() === 'Start Quiz')?.trigger('click')

    expect(useQuizStore().mode).toBe('advanced')
  })
})
