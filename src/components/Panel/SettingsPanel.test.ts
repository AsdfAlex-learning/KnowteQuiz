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
  backupData: vi.fn(),
  getDataStatus: vi.fn(),
  restoreLatestBackup: vi.fn(),
}))

vi.mock('@/services/mistake', () => ({
  listPromptTemplates: vi.fn(async () => []),
}))

describe('SettingsPanel', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(settingsService.getSettings).mockResolvedValue(defaultSettings())
    vi.mocked(settingsService.getDataStatus).mockResolvedValue({
      data_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz',
      files: [],
    })
    vi.stubGlobal('confirm', vi.fn(() => true))
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

  it('shows settings persistence errors after saving fails', async () => {
    vi.mocked(settingsService.saveSettings).mockRejectedValue(new Error('HTTP 500: Failed to write settings.json'))
    const wrapper = mount(SettingsPanel, {
      global: {
        plugins: [createPinia()],
      },
    })

    await wrapper.findAll('button').find((button) => button.text() === 'Save Settings')?.trigger('click')
    await vi.dynamicImportSettled()

    expect(wrapper.text()).toContain('Failed to write settings.json')
  })

  it('shows data backup results after a manual backup succeeds', async () => {
    vi.mocked(settingsService.backupData).mockResolvedValue({
      backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
      files: ['settings.json', 'mistakes.json'],
    })
    const wrapper = mount(SettingsPanel, {
      global: {
        plugins: [createPinia()],
      },
    })

    await wrapper.findAll('button').find((button) => button.text() === 'Backup Data Now')?.trigger('click')
    await vi.dynamicImportSettled()

    expect(wrapper.text()).toContain('Backed up 2 files')
    expect(wrapper.text()).toContain('20260621-120000')
  })

  it('shows restore results after restoring the latest backup', async () => {
    vi.mocked(settingsService.restoreLatestBackup).mockResolvedValue({
      backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-120000',
      pre_restore_backup_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz/backups/20260621-130000',
      files: ['settings.json', 'mistakes.json'],
    })
    const wrapper = mount(SettingsPanel, {
      global: {
        plugins: [createPinia()],
      },
    })

    await wrapper.findAll('button').find((button) => button.text() === 'Restore Latest Backup')?.trigger('click')
    await vi.dynamicImportSettled()

    expect(wrapper.text()).toContain('Restored 2 files')
    expect(wrapper.text()).toContain('20260621-120000')
  })

  it('shows data file sizes from the settings store', async () => {
    vi.mocked(settingsService.getDataStatus).mockResolvedValue({
      data_dir: 'C:/Users/Alex/AppData/Roaming/knowtequiz',
      files: [
        {
          name: 'settings.json',
          exists: true,
          size_bytes: 2048,
          modified_at: '2026-06-21T12:00:00Z',
        },
        {
          name: 'mistakes.json',
          exists: false,
          size_bytes: 0,
          modified_at: null,
        },
      ],
    })
    const wrapper = mount(SettingsPanel, {
      global: {
        plugins: [createPinia()],
      },
    })

    await vi.dynamicImportSettled()

    expect(wrapper.text()).toContain('Data Files')
    expect(wrapper.text()).toContain('settings.json')
    expect(wrapper.text()).toContain('2 KB')
    expect(wrapper.text()).toContain('2026-06-21 12:00')
    expect(wrapper.text()).toContain('mistakes.json')
    expect(wrapper.text()).toContain('Missing')
  })
})
