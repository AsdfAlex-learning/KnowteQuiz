// @vitest-environment jsdom

import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import FolderSelector from './FolderSelector.vue';

const isTauriMock = vi.hoisted(() => vi.fn(() => false));

vi.mock('@/services/tauri', () => ({
  isTauri: isTauriMock,
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => path),
  webStream: vi.fn(),
}));

vi.mock('@/composables/useI18n', () => ({
  useI18n: () => ({ t: (key: string) => key }),
}));

describe('FolderSelector', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    isTauriMock.mockReturnValue(false);
  });

  it('shows the manual path input in web mode without a root', () => {
    const wrapper = mount(FolderSelector);

    expect(wrapper.find('input[type="text"]').exists()).toBe(true);
  });

  it('hides the manual path input on desktop', () => {
    isTauriMock.mockReturnValue(true);
    const wrapper = mount(FolderSelector);

    expect(wrapper.find('input[type="text"]').exists()).toBe(false);
  });
});
