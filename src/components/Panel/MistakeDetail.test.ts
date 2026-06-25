// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import MistakeDetail from './MistakeDetail.vue'
import type { MistakeEntry } from '@/types/mistake'

vi.mock('@/services/mistake', () => ({
  markMistakeReviewed: vi.fn(),
}))

function advancedMistake(): MistakeEntry {
  return {
    id: 'm1',
    note_path: '/notes/rust.md',
    note_title: 'Rust',
    question: 'Why did ownership move?',
    user_answer: 'A',
    correct_answer: 'B',
    explanation: 'Because the value was moved.',
    mode: 'advanced',
    user_reasoning: 'I thought it copied.',
    diagnosis: {
      rounds: 1,
      conversation: [],
      final_report: {
        summary: 'You confused move semantics with copy semantics.',
        blind_spots: [
          {
            tag: 'Ownership transfer',
            severity: 'high',
            description: 'Moves invalidate the previous binding.',
            note_reference: 'Ownership section',
            suggestion: 'Review move examples.',
          },
        ],
        overall_level: 'Needs review',
        next_steps: ['Re-read ownership notes'],
      },
    },
    created_at: '2026-01-01T00:00:00.000Z',
    review_count: 0,
  }
}

describe('MistakeDetail', () => {
  it('renders diagnosis final report from advanced mistake context', () => {
    const wrapper = mount(MistakeDetail, {
      props: {
        mistake: advancedMistake(),
      },
    })

    expect(wrapper.text()).toContain('You confused move semantics with copy semantics.')
    expect(wrapper.text()).toContain('Ownership transfer')
    expect(wrapper.text()).toContain('Needs review')
    expect(wrapper.text()).toContain('Re-read ownership notes')
  })
})
