import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useNavigationStore } from './navigation'
import { useExplorerStore } from './explorer'
import { useReaderStore } from './reader'
import * as noteService from '../services/note'
import * as settingsService from '../services/settings'
import { defaultSettings } from '../utils/defaults'

vi.mock('../services/note', () => ({
  selectFolder: vi.fn(),
  scanNotes: vi.fn(),
  readNote: vi.fn(),
}))

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}))

describe('navigation store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('opens a note through explorer selection and reader loading', async () => {
    const settings = defaultSettings()
    settings.workspace.root_path = '/notes'
    settings.workspace.scroll_positions = {
      '/notes/rust/ownership.md': 180,
    }
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    vi.mocked(noteService.readNote).mockResolvedValue({
      path: '/notes/rust/ownership.md',
      title: 'Ownership',
      content: '# Ownership',
      metadata: {},
    })
    const navigationStore = useNavigationStore()
    const explorerStore = useExplorerStore()
    const readerStore = useReaderStore()
    explorerStore.rootPath = '/notes'

    await navigationStore.openNote('/notes/rust/ownership.md')

    expect(explorerStore.selectedPath).toBe('/notes/rust/ownership.md')
    expect(readerStore.currentNote?.title).toBe('Ownership')
    expect(readerStore.scrollTop).toBe(180)
    expect(settingsService.saveSettings).toHaveBeenCalledWith({
      ...settings,
      workspace: {
        ...settings.workspace,
        root_path: '/notes',
        expanded_dirs: [],
        selected_path: '/notes/rust/ownership.md',
      },
    })
  })
})
