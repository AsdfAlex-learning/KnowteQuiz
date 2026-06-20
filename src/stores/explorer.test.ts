import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useExplorerStore } from './explorer'
import * as noteService from '../services/note'
import * as settingsService from '../services/settings'
import { defaultSettings } from '../utils/defaults'

vi.mock('../services/note', () => ({
  selectFolder: vi.fn(),
  scanNotes: vi.fn(),
}))

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}))

describe('explorer store workspace persistence', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('restores root path, expanded dirs, selected path, and scans the saved tree', async () => {
    const settings = defaultSettings()
    settings.workspace = {
      root_path: '/notes',
      expanded_dirs: ['/notes/rust'],
      selected_path: '/notes/rust/ownership.md',
      scroll_positions: {},
    }
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(noteService.scanNotes).mockResolvedValue([
      { name: 'ownership.md', path: '/notes/rust/ownership.md', is_dir: false, children: [] },
    ])
    const store = useExplorerStore()

    await store.restoreWorkspace()

    expect(store.rootPath).toBe('/notes')
    expect(store.selectedPath).toBe('/notes/rust/ownership.md')
    expect(store.expandedDirs.has('/notes/rust')).toBe(true)
    expect(noteService.scanNotes).toHaveBeenCalledWith('/notes')
  })

  it('persists selected path into workspace settings', async () => {
    const settings = defaultSettings()
    settings.workspace.root_path = '/notes'
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    const store = useExplorerStore()
    store.rootPath = '/notes'

    await store.selectPath('/notes/vue/ref.md')

    expect(settingsService.saveSettings).toHaveBeenCalledWith({
      ...settings,
      workspace: {
        ...settings.workspace,
        root_path: '/notes',
        expanded_dirs: [],
        selected_path: '/notes/vue/ref.md',
      },
    })
  })

  it('scans and persists a manually entered root path', async () => {
    const settings = defaultSettings()
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    vi.mocked(noteService.scanNotes).mockResolvedValue([
      { name: 'rust.md', path: '/notes/rust.md', is_dir: false, children: [] },
    ])
    const store = useExplorerStore()

    await store.openRootPath('/notes')

    expect(noteService.scanNotes).toHaveBeenCalledWith('/notes')
    expect(store.rootPath).toBe('/notes')
    expect(store.tree).toEqual([
      { name: 'rust.md', path: '/notes/rust.md', is_dir: false, children: [] },
    ])
    expect(settingsService.saveSettings).toHaveBeenCalledWith({
      ...settings,
      workspace: {
        ...settings.workspace,
        root_path: '/notes',
        expanded_dirs: [],
        selected_path: null,
      },
    })
  })

  it('does not persist a manually entered root path when scanning fails', async () => {
    vi.mocked(noteService.scanNotes).mockRejectedValue(new Error('Directory does not exist'))
    const store = useExplorerStore()

    await store.openRootPath('/missing')

    expect(store.rootPath).toBeNull()
    expect(store.error).toContain('Directory does not exist')
    expect(settingsService.saveSettings).not.toHaveBeenCalled()
  })
})
