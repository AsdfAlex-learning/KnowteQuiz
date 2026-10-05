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

  it('counts and navigates search matches with real highlight marks', async () => {
    vi.useFakeTimers();
    const readerStore = useReaderStore();
    readerStore.currentNote = {
      path: '/notes/ownership.md',
      title: 'Ownership',
      content: ['Ownership moves values.', 'Ownership also moves borrows.'].join('\n\n'),
      metadata: {},
    };

    const wrapper = mount(MarkdownRenderer, { attachTo: document.body });
    const article = document.body.querySelector('article');
    expect(article).not.toBeNull();

    // Open the search bar with the Ctrl+F shortcut before typing.
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'f', ctrlKey: true }));
    await nextTick();

    const input = document.body.querySelector<HTMLInputElement>('input');
    input!.value = 'ownership';
    input!.dispatchEvent(new Event('input'));
    await vi.advanceTimersByTimeAsync(200);

    const hits = document.body.querySelectorAll('mark[data-search-hit]');
    expect(hits).toHaveLength(2);
    expect(hits[0].getAttribute('data-search-active')).toBe('');
    const counter = document.body.querySelector('span.min-w-\\[40px\\]');
    expect(counter?.textContent).toBe('1/2');

    await wrapper.get('button[title="Next match"]').trigger('click');
    expect(document.body.querySelector('span.min-w-\\[40px\\]')?.textContent).toBe('2/2');
    expect(document.body.querySelectorAll('mark[data-search-hit]')[1].hasAttribute('data-search-active')).toBe(true);

    // Closing the search restores the pristine rendered HTML.
    await wrapper.get('button[title="Close search"]').trigger('click');
    expect(document.body.querySelectorAll('mark[data-search-hit]')).toHaveLength(0);
    expect(article!.innerHTML).not.toContain('data-search-hit');

    vi.useRealTimers();
    wrapper.unmount();
  });

  it('shows an explicit zero-count for queries without matches', async () => {
    vi.useFakeTimers();
    const readerStore = useReaderStore();
    readerStore.currentNote = {
      path: '/notes/ownership.md',
      title: 'Ownership',
      content: 'Nothing relevant here.',
      metadata: {},
    };

    const wrapper = mount(MarkdownRenderer, { attachTo: document.body });

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'f', ctrlKey: true }));
    await nextTick();

    const input = document.body.querySelector<HTMLInputElement>('input');
    input!.value = 'missing-word';
    input!.dispatchEvent(new Event('input'));
    await vi.advanceTimersByTimeAsync(200);

    expect(document.body.querySelector('span.min-w-\\[40px\\]')?.textContent).toBe('0/0');
    expect(document.body.querySelectorAll('mark[data-search-hit]')).toHaveLength(0);

    vi.useRealTimers();
    wrapper.unmount();
  });
});
