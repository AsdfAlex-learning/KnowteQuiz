import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useLayoutStore } from './layout';
import { useSettingsStore } from './settings';
import * as settingsService from '../services/settings';
import { defaultSettings } from '../utils/defaults';

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}));

describe('layout store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('clamps loaded sidebar widths to usable bounds', async () => {
    const settings = defaultSettings();
    settings.ui.layout.left_width = 20;
    settings.ui.layout.right_width = 5000;
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings);
    const store = useLayoutStore();

    await store.loadLayout();

    expect(store.explorerWidth).toBe(150);
    expect(store.readerWidth).toBe(600);
  });

  it('restores the persisted exploration mode', async () => {
    const settings = defaultSettings();
    settings.ui.last_mode = 'exploration';
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings);
    const store = useLayoutStore();

    await store.loadLayout();

    expect(store.viewMode).toBe('exploration');
  });

  it('falls back to the reader for unknown persisted modes', async () => {
    const settings = defaultSettings();
    (settings.ui as { last_mode?: string }).last_mode = 'unknown-mode';
    vi.mocked(settingsService.getSettings).mockResolvedValue(settings);
    const store = useLayoutStore();

    await store.loadLayout();

    expect(store.viewMode).toBe('reader');
  });

  it('persists the mode through the settings write queue', async () => {
    const store = useLayoutStore();
    const settingsStore = useSettingsStore();

    store.setViewMode('exploration');

    await vi.waitFor(() => expect(settingsStore.settings.ui.last_mode).toBe('exploration'));
    expect(settingsService.saveSettings).toHaveBeenCalled();
  });

  it('toggles between reader and exploration', () => {
    const store = useLayoutStore();

    store.toggleViewMode();
    expect(store.viewMode).toBe('exploration');

    store.toggleViewMode();
    expect(store.viewMode).toBe('reader');
  });
});
