import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import FolderSelector from './FolderSelector.vue';
import { selectFolder } from '@/services/note';

const isTauriMock = vi.hoisted(() => vi.fn(() => false));

vi.mock('@/services/tauri', () => ({
  isTauri: isTauriMock,
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => path),
  webStream: vi.fn(),
}));

vi.mock('@/services/note', () => ({
  selectFolder: vi.fn(),
  scanNotes: vi.fn().mockResolvedValue([]),
}));

vi.mock('@/services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn().mockResolvedValue(true),
}));

vi.mock('@/composables/useI18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}));

describe('FolderSelector', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    isTauriMock.mockReturnValue(false);
  });

  it('uses a button instead of a manual path input in web mode', () => {
    const wrapper = mount(FolderSelector);

    expect(wrapper.find('input[type="text"]').exists()).toBe(false);
    expect(wrapper.find('.open-folder-btn').exists()).toBe(true);
  });

  it('never renders a manual path input on desktop', () => {
    isTauriMock.mockReturnValue(true);
    const wrapper = mount(FolderSelector);

    expect(wrapper.find('input[type="text"]').exists()).toBe(false);
  });

  it('opens the folder picker when the button is clicked', async () => {
    vi.mocked(selectFolder).mockResolvedValue('/notes');
    const wrapper = mount(FolderSelector);

    await wrapper.find('.open-folder-btn').trigger('click');
    await flushPromises();

    expect(selectFolder).toHaveBeenCalled();
  });
});
