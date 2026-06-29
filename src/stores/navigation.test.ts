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

  it('does not select or persist a note path when reading the note fails', async () => {
    const settings = defaultSettings()
    settings.workspace.root_path = '/notes'
    settings.workspace.selected_path = '/notes/current.md'
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    vi.mocked(noteService.readNote).mockRejectedValue(new Error('File not found'))
    const navigationStore = useNavigationStore()
    const explorerStore = useExplorerStore()
    const readerStore = useReaderStore()
    explorerStore.rootPath = '/notes'
    explorerStore.selectedPath = '/notes/current.md'

    await navigationStore.openNote('/notes/missing.md')

    expect(explorerStore.selectedPath).toBe('/notes/current.md')
    expect(readerStore.currentNote).toBeNull()
    expect(navigationStore.error).toContain('File not found')
    expect(settingsService.saveSettings).not.toHaveBeenCalled()
  })

  it('clears a saved selected path when workspace restore cannot read it', async () => {
    const settings = defaultSettings()
    settings.workspace.root_path = '/notes'
    settings.workspace.selected_path = '/notes/deleted.md'
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    vi.mocked(noteService.readNote).mockRejectedValue(new Error('File not found'))
    const navigationStore = useNavigationStore()
    const explorerStore = useExplorerStore()
    const readerStore = useReaderStore()
    explorerStore.rootPath = '/notes'
    explorerStore.selectedPath = '/notes/deleted.md'

    await navigationStore.restoreSelectedNote()

    expect(readerStore.currentNote).toBeNull()
    expect(explorerStore.selectedPath).toBeNull()
    expect(navigationStore.error).toContain('File not found')
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

  it('opens a new root path and clears the note from the previous workspace', async () => {
    const settings = defaultSettings()
    settings.workspace = {
      ...settings.workspace,
      root_path: '/old',
      selected_path: '/old/topic.md',
    }
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    vi.mocked(noteService.scanNotes).mockResolvedValue([
      { name: 'new.md', path: '/new/new.md', is_dir: false, children: [] },
    ])
    const navigationStore = useNavigationStore()
    const explorerStore = useExplorerStore()
    const readerStore = useReaderStore()
    explorerStore.rootPath = '/old'
    explorerStore.selectedPath = '/old/topic.md'
    readerStore.currentNote = {
      path: '/old/topic.md',
      title: 'Old Topic',
      content: '# Old Topic',
      metadata: {},
    }
    readerStore.scrollTop = 240

    await navigationStore.openRootPath('/new')

    expect(explorerStore.rootPath).toBe('/new')
    expect(explorerStore.selectedPath).toBeNull()
    expect(readerStore.currentNote).toBeNull()
    expect(readerStore.scrollTop).toBe(0)
  })

  it('chooses a folder through the picker and opens it as a workspace root', async () => {
    const settings = defaultSettings()
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings)
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true)
    vi.mocked(noteService.selectFolder).mockResolvedValue('/picked')
    vi.mocked(noteService.scanNotes).mockResolvedValue([
      { name: 'picked.md', path: '/picked/picked.md', is_dir: false, children: [] },
    ])
    const navigationStore = useNavigationStore()
    const explorerStore = useExplorerStore()

    await navigationStore.chooseFolder()

    expect(noteService.selectFolder).toHaveBeenCalled()
    expect(explorerStore.rootPath).toBe('/picked')
    expect(explorerStore.tree).toEqual([
      { name: 'picked.md', path: '/picked/picked.md', is_dir: false, children: [] },
    ])
  })
})
