// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, describe, expect, it, vi } from 'vitest'
import QuizGenerator from './QuizGenerator.vue'
import { useExplorerStore } from '@/stores/explorer'
import { useQuizStore } from '@/stores/quiz'
import { useMistakeStore } from '@/stores/mistakes'
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

  function mockGeneratedQuiz() {
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
  }

  it('starts an advanced quiz when advanced mode is selected', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const explorerStore = useExplorerStore()
    explorerStore.selectedPath = '/notes/rust.md'
    mockGeneratedQuiz()

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    })
    await nextTick()

    await wrapper.findAll('button').find((button) => button.text() === 'Advanced')?.trigger('click')
    await wrapper.findAll('button').find((button) => button.text() === 'Start Quiz')?.trigger('click')

    expect(useQuizStore().mode).toBe('advanced')
  })

  it('clears stale mistake save state before starting a new quiz', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const explorerStore = useExplorerStore()
    explorerStore.selectedPath = '/notes/rust.md'
    const mistakeStore = useMistakeStore()
    mistakeStore.savedIds.add('q1')
    mistakeStore.errors.set('q1', 'old error')
    mockGeneratedQuiz()

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    })
    await nextTick()

    const startButton = wrapper.findAll('button').find((button) => button.text() === 'Start Quiz')
    expect(startButton?.exists()).toBe(true)
    expect(startButton?.attributes('disabled')).toBeUndefined()

    await startButton?.trigger('click')

    expect(mistakeStore.isSaved('q1')).toBe(false)
    expect(mistakeStore.errorFor('q1')).toBeNull()
  })
})
