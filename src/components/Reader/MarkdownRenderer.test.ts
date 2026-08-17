// @vitest-environment jsdom

import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { nextTick } from 'vue';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import MarkdownRenderer from './MarkdownRenderer.vue';
import { useReaderStore } from '@/stores/reader';

vi.mock('@/composables/useI18n', () => ({
  useI18n: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'reader.find_in_note': 'Find in note...',
        'reader.outline': 'Outline',
        'reader.loading': 'Loading note...',
        'reader.no_content': 'Select a note...',
        'app.name': 'KnowteQuiz',
        'sidebar.open_folder': 'Open Folder',
      };
      return translations[key] || key;
    },
    locale: { value: 'en' },
    availableLocales: [],
    setLocale: () => {},
  }),
}));

vi.mock('@/services/tauri', () => ({
  convertFileSrc: (path: string) => path,
  isTauri: () => false,
}));

vi.mock('@/services/note', () => ({
  readNote: vi.fn(),
}));

vi.mock('@/services/settings', () => ({
  getSettings: vi.fn(),
  saveSettings: vi.fn(),
}));

describe('MarkdownRenderer', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('renders duplicate headings with the same unique ids used by the outline', async () => {
    const readerStore = useReaderStore();
    readerStore.currentNote = {
      path: '/notes/ownership.md',
      title: 'Ownership',
      content: ['## Overview', 'First section.', '## Overview', 'Second section.'].join('\n'),
      metadata: {},
    };

    const wrapper = mount(MarkdownRenderer);

    expect(wrapper.html()).toContain('<h2 id="overview">Overview</h2>');
    expect(wrapper.html()).toContain('<h2 id="overview-2">Overview</h2>');
    await wrapper.get('button[title="Outline"]').trigger('click');
    await nextTick();

    expect(wrapper.findAll('nav a').map((link) => link.text())).toEqual(['Overview', 'Overview']);
  });
});
