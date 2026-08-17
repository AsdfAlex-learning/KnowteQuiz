import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useReaderStore } from './reader';
import * as noteService from '../services/note';
import * as settingsService from '../services/settings';
import { defaultSettings } from '../utils/defaults';

vi.mock('../services/note', () => ({
  readNote: vi.fn(),
}));

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}));

describe('reader store scroll persistence', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('restores saved scroll position for the loaded note', async () => {
    const settings = defaultSettings();
    settings.workspace.scroll_positions = {
      '/notes/vue/reactivity.md': 420,
    };
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings);
    vi.mocked(noteService.readNote).mockResolvedValue({
      path: '/notes/vue/reactivity.md',
      title: 'Reactivity',
      content: '# Reactivity',
      metadata: {},
    });
    const store = useReaderStore();

    await store.loadNote('/notes/vue/reactivity.md');

    expect(store.scrollTop).toBe(420);
  });

  it('loads note content even when scroll settings cannot be read', async () => {
    vi.mocked(settingsService.getSettings).mockRejectedValue(new Error('Failed to parse settings.json'));
    vi.mocked(noteService.readNote).mockResolvedValue({
      path: '/notes/vue/reactivity.md',
      title: 'Reactivity',
      content: '# Reactivity',
      metadata: {},
    });
    const store = useReaderStore();

    await store.loadNote('/notes/vue/reactivity.md');

    expect(store.currentNote?.title).toBe('Reactivity');
    expect(store.scrollTop).toBe(0);
    expect(store.error).toBeNull();
  });

  it('ignores stale note reads when a newer note finishes first', async () => {
    const settings = defaultSettings();
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings);
    let resolveOld!: (note: Awaited<ReturnType<typeof noteService.readNote>>) => void;
    let resolveNew!: (note: Awaited<ReturnType<typeof noteService.readNote>>) => void;
    vi.mocked(noteService.readNote).mockImplementation(
      (path) =>
        new Promise((resolve) => {
          if (path === '/notes/old.md') {
            resolveOld = resolve;
          } else {
            resolveNew = resolve;
          }
        })
    );
    const store = useReaderStore();

    const oldLoad = store.loadNote('/notes/old.md');
    const newLoad = store.loadNote('/notes/new.md');
    resolveNew({
      path: '/notes/new.md',
      title: 'New',
      content: '# New',
      metadata: {},
    });

    await expect(newLoad).resolves.toBe(true);
    resolveOld({
      path: '/notes/old.md',
      title: 'Old',
      content: '# Old',
      metadata: {},
    });

    await expect(oldLoad).resolves.toBe(false);
    expect(store.currentNote?.path).toBe('/notes/new.md');
    expect(store.loading).toBe(false);
  });

  it('ignores an in-flight note read after the reader is cleared', async () => {
    const settings = defaultSettings();
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings);
    let resolveNote!: (note: Awaited<ReturnType<typeof noteService.readNote>>) => void;
    vi.mocked(noteService.readNote).mockReturnValue(
      new Promise((resolve) => {
        resolveNote = resolve;
      })
    );
    const store = useReaderStore();

    const load = store.loadNote('/notes/old.md');
    store.clearNote();
    resolveNote({
      path: '/notes/old.md',
      title: 'Old',
      content: '# Old',
      metadata: {},
    });

    await expect(load).resolves.toBe(false);
    expect(store.currentNote).toBeNull();
    expect(store.scrollTop).toBe(0);
  });

  it('persists scroll position by note path', async () => {
    const settings = defaultSettings();
    settings.workspace.selected_path = '/notes/vue/reactivity.md';
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings);
    vi.mocked(settingsService.saveSettings).mockResolvedValue(true);
    const store = useReaderStore();

    await store.saveScrollPosition('/notes/vue/reactivity.md', 640);

    expect(settingsService.saveSettings).toHaveBeenCalledWith({
      ...settings,
      workspace: {
        ...settings.workspace,
        scroll_positions: {
          '/notes/vue/reactivity.md': 640,
        },
      },
    });
  });

  it('records scroll persistence errors without throwing', async () => {
    vi.mocked(settingsService.getSettings).mockRejectedValue(new Error('Failed to parse settings.json'));
    const store = useReaderStore();

    await expect(store.saveScrollPosition('/notes/vue/reactivity.md', 640)).resolves.toBeUndefined();

    expect(store.error).toContain('Failed to parse settings.json');
    expect(settingsService.saveSettings).not.toHaveBeenCalled();
  });
});
