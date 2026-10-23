import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import FileTreeItem from './FileTreeItem.vue';
import { useExplorationStore } from '@/stores/exploration';
import { useLayoutStore } from '@/stores/layout';
import type { NoteTreeNode } from '@/types/note';

vi.mock('@/services/note', () => ({
  selectFolder: vi.fn(),
  scanNotes: vi.fn(),
  readNote: vi.fn(),
}));

vi.mock('@/services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}));

function noteNode(overrides: Partial<NoteTreeNode> = {}): NoteTreeNode {
  return {
    name: 'Note.md',
    path: 'Note.md',
    kind: 'markdown',
    is_dir: false,
    children: [],
    ...overrides,
  };
}

describe('FileTreeItem', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('hides markdown file extensions case-insensitively', () => {
    const wrapper = mount(FileTreeItem, {
      props: {
        depth: 0,
        node: noteNode({ name: 'Longform.Markdown', path: '/notes/Longform.Markdown' }),
      },
    });

    expect(wrapper.text()).toContain('Longform');
    expect(wrapper.text()).not.toContain('Longform.Markdown');
  });

  it('toggles exploration selection instead of opening the note', async () => {
    useLayoutStore().viewMode = 'exploration';
    const explorationStore = useExplorationStore();
    const wrapper = mount(FileTreeItem, {
      props: { depth: 0, node: noteNode({ path: 'notes/a.md' }) },
    });

    await wrapper.find('.tree-item').trigger('click');

    expect(explorationStore.isSelected('notes/a.md')).toBe(true);
    expect(wrapper.find('.explore-check--on').exists()).toBe(true);
  });

  it('keeps unconverted documents unselectable in exploration mode', async () => {
    useLayoutStore().viewMode = 'exploration';
    const explorationStore = useExplorationStore();
    const wrapper = mount(FileTreeItem, {
      props: {
        depth: 0,
        node: noteNode({
          name: 'Paper.pdf',
          path: 'notes/Paper.pdf',
          kind: 'document',
        }),
      },
    });

    await wrapper.find('.tree-item').trigger('click');

    expect(explorationStore.selectedCount).toBe(0);
    expect(wrapper.find('.tree-item').classes()).toContain('tree-item--disabled');
  });
});
