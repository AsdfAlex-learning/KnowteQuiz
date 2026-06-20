import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useLayoutStore } from './layout'
import * as settingsService from '../services/settings'
import { defaultSettings } from '../utils/defaults'

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}))

describe('layout store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('clamps loaded sidebar widths to usable bounds', async () => {
    const settings = defaultSettings()
    settings.ui.layout.left_width = 20
    settings.ui.layout.right_width = 5000
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    const store = useLayoutStore()

    await store.loadLayout()

    expect(store.explorerWidth).toBe(150)
    expect(store.readerWidth).toBe(600)
  })
})
