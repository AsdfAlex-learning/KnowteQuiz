import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useReaderStore } from './reader'
import * as noteService from '../services/note'
import * as settingsService from '../services/settings'
import { defaultSettings } from '../utils/defaults'

vi.mock('../services/note', () => ({
  readNote: vi.fn(),
}))

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}))

describe('reader store scroll persistence', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('restores saved scroll position for the loaded note', async () => {
    const settings = defaultSettings()
    settings.workspace.scroll_positions = {
      '/notes/vue/reactivity.md': 420,
    }
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(noteService.readNote).mockResolvedValue({
      path: '/notes/vue/reactivity.md',
      title: 'Reactivity',
      content: '# Reactivity',
      metadata: {},
    })
    const store = useReaderStore()

    await store.loadNote('/notes/vue/reactivity.md')

    expect(store.scrollTop).toBe(420)
  })

  it('persists scroll position by note path', async () => {
    const settings = defaultSettings()
    settings.workspace.selected_path = '/notes/vue/reactivity.md'
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    const store = useReaderStore()

    await store.saveScrollPosition('/notes/vue/reactivity.md', 640)

    expect(settingsService.saveSettings).toHaveBeenCalledWith({
      ...settings,
      workspace: {
        ...settings.workspace,
        scroll_positions: {
          '/notes/vue/reactivity.md': 640,
        },
      },
    })
  })
})
