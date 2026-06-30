// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import QuestionCard from './QuestionCard.vue'
import OptionCard from './OptionCard.vue'

describe('QuestionCard', () => {
  it('highlights the correct option when the model answer is option text', () => {
    const wrapper = mount(QuestionCard, {
      props: {
        question: {
          id: 'q1',
          question_type: 'single',
          question: 'Pick one.',
          options: ['A. Alpha', 'B. Beta'],
          answer: 'Alpha',
          explanation: '',
        },
        currentIndex: 0,
        total: 1,
        selectedOptions: [],
        submitted: true,
      },
    })

    const options = wrapper.findAllComponents(OptionCard)

    expect(options[0].props('state')).toBe('correct')
    expect(options[1].props('state')).toBe('default')
  })

  it('removes repeated option labels from rendered option text', () => {
    const wrapper = mount(QuestionCard, {
      props: {
        question: {
          id: 'q1',
          question_type: 'single',
          question: 'Pick one.',
          options: ['A. Alpha', 'B\uff1aBeta'],
          answer: 'A',
          explanation: '',
        },
        currentIndex: 0,
        total: 1,
        selectedOptions: [],
        submitted: false,
      },
    })

    const options = wrapper.findAllComponents(OptionCard)

    expect(options[0].props('text')).toBe('Alpha')
    expect(options[1].props('text')).toBe('Beta')
  })
})
