// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import MarkdownRenderer from './MarkdownRenderer.vue'
import { useReaderStore } from '@/stores/reader'

vi.mock('@/services/tauri', () => ({
  convertFileSrc: (path: string) => path,
  isTauri: () => false,
}))

vi.mock('@/services/note', () => ({
  readNote: vi.fn(),
}))

vi.mock('@/services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}))

describe('MarkdownRenderer', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('renders duplicate headings with the same unique ids used by the outline', async () => {
    const readerStore = useReaderStore()
    readerStore.currentNote = {
      path: '/notes/ownership.md',
      title: 'Ownership',
      content: ['## Overview', 'First section.', '## Overview', 'Second section.'].join('\n'),
      metadata: {},
    }

    const wrapper = mount(MarkdownRenderer)

    expect(wrapper.html()).toContain('<h2 id="overview">Overview</h2>')
    expect(wrapper.html()).toContain('<h2 id="overview-2">Overview</h2>')
    await wrapper.get('button[title="Toggle outline"]').trigger('click')
    await nextTick()

    expect(wrapper.findAll('nav a').map((link) => link.text())).toEqual(['Overview', 'Overview'])
  })
})
