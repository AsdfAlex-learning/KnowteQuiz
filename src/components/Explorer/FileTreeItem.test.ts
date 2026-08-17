// @vitest-environment jsdom

import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import FileTreeItem from './FileTreeItem.vue';

vi.mock('@/services/note', () => ({
  selectFolder: vi.fn(),
  scanNotes: vi.fn(),
  readNote: vi.fn(),
}));

vi.mock('@/services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}));

describe('FileTreeItem', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('hides markdown file extensions case-insensitively', () => {
    const wrapper = mount(FileTreeItem, {
      props: {
        depth: 0,
        node: {
          name: 'Longform.Markdown',
          path: '/notes/Longform.Markdown',
          is_dir: false,
          children: [],
        },
      },
    });

    expect(wrapper.text()).toContain('Longform');
    expect(wrapper.text()).not.toContain('Longform.Markdown');
  });
});
