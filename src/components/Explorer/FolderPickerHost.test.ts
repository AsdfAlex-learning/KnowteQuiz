import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import FolderPickerHost from './FolderPickerHost.vue';
import { closeFolderPicker, openFolderPicker } from '@/composables/useFolderPicker';
import { listDirectories } from '@/services/note';

vi.mock('@/services/note', () => ({
  listDirectories: vi.fn(),
}));

const listing = {
  path: '/root',
  parent: '/',
  directories: [
    { name: 'notes', path: '/root/notes' },
    { name: 'archive', path: '/root/archive' },
  ],
  roots: [{ name: '/', path: '/' }],
};

describe('FolderPickerHost', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    closeFolderPicker(null);
  });

  it('lists directories and resolves the selected folder', async () => {
    vi.mocked(listDirectories).mockResolvedValue(listing);
    const choice = openFolderPicker('/root');
    const wrapper = mount(FolderPickerHost, { global: { stubs: { teleport: true } } });
    await flushPromises();

    expect(listDirectories).toHaveBeenCalledWith('/root');
    expect(wrapper.text()).toContain('notes');
    expect(wrapper.text()).toContain('archive');

    const select = wrapper.findAll('button').find((button) => button.text() === 'Select this folder');
    await select?.trigger('click');

    expect(await choice).toBe('/root');
    wrapper.unmount();
  });

  it('enters a subfolder', async () => {
    vi.mocked(listDirectories)
      .mockResolvedValueOnce(listing)
      .mockResolvedValueOnce({
        ...listing,
        path: '/root/notes',
        parent: '/root',
        directories: [],
      });
    openFolderPicker('/root');
    const wrapper = mount(FolderPickerHost, { global: { stubs: { teleport: true } } });
    await flushPromises();

    const notes = wrapper.findAll('button').find((button) => button.text() === 'notes');
    await notes?.trigger('click');
    await flushPromises();

    expect(listDirectories).toHaveBeenLastCalledWith('/root/notes');
    expect(wrapper.text()).toContain('/root/notes');
    closeFolderPicker(null);
    wrapper.unmount();
  });

  it('resolves null when cancelled', async () => {
    vi.mocked(listDirectories).mockResolvedValue(listing);
    const choice = openFolderPicker('/root');
    const wrapper = mount(FolderPickerHost, { global: { stubs: { teleport: true } } });
    await flushPromises();

    const cancel = wrapper.findAll('button').find((button) => button.text() === 'Cancel');
    await cancel?.trigger('click');

    expect(await choice).toBeNull();
    wrapper.unmount();
  });
});
