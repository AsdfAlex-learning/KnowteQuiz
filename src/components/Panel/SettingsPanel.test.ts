// @vitest-environment jsdom

import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsPanel from './SettingsPanel.vue'
import { defaultSettings } from '@/utils/defaults'
import * as settingsService from '@/services/settings'

vi.mock('@/services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
  testConnection: vi.fn(),
}))

vi.mock('@/services/mistake', () => ({
  listPromptTemplates: vi.fn(async () => []),
}))

describe('SettingsPanel', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(settingsService.getSettings).mockResolvedValue(defaultSettings())
  })

  it('renders failed connection tests with the error state', async () => {
    vi.mocked(settingsService.testConnection).mockResolvedValue({
      ok: false,
      kind: 'auth',
      message: 'LLM endpoint rejected the API key',
      status: 401,
    })
    const wrapper = mount(SettingsPanel, {
      global: {
        plugins: [createPinia()],
      },
    })

    await wrapper.findAll('button').find((button) => button.text() === 'Test Connection')?.trigger('click')
    await vi.dynamicImportSettled()

    expect(wrapper.text()).toContain('LLM endpoint rejected the API key')
    expect(wrapper.find('.bg-\\[var\\(--color-error\\)\\]\\/10').exists()).toBe(true)
  })
})
