// @vitest-environment jsdom

import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ErrorBook from './ErrorBook.vue';
import * as mistakeService from '@/services/mistake';

vi.mock('@/services/mistake', () => ({
  loadMistakes: vi.fn(),
  exportMistakes: vi.fn(),
  markMistakeReviewed: vi.fn(),
  saveMistake: vi.fn(),
}));

describe('ErrorBook', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    setActivePinia(createPinia());
    vi.mocked(mistakeService.loadMistakes).mockResolvedValue([]);
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it('filters mistakes by blind spot tag from the filter bar', async () => {
    const wrapper = mount(ErrorBook, {
      global: {
        plugins: [createPinia()],
        stubs: {
          MistakeItem: true,
          MistakeDetail: true,
        },
      },
    });

    await wrapper.get('input[placeholder="Filter blind spot tag..."]').setValue('ownership');
    await vi.advanceTimersByTimeAsync(250);

    expect(mistakeService.loadMistakes).toHaveBeenLastCalledWith({
      mode: undefined,
      search_text: undefined,
      blind_spot_tag: 'ownership',
      offset: 0,
      limit: 20,
    });
  });
});
