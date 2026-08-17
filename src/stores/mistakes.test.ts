import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useMistakeStore } from './mistakes';
import * as mistakeService from '../services/mistake';
import type { MistakeEntry, MistakeMode } from '../types/mistake';

vi.mock('../services/mistake', () => ({
  loadMistakes: vi.fn(),
  saveMistake: vi.fn(),
  markMistakeReviewed: vi.fn(),
  exportMistakes: vi.fn(),
}));

function mistake(id = 'm1', mode: MistakeMode = 'basic'): MistakeEntry {
  return {
    id,
    note_path: '/notes/rust.md',
    note_title: 'Rust',
    question: 'Which answer is correct?',
    user_answer: 'A',
    correct_answer: 'B',
    explanation: 'B is correct.',
    mode,
    created_at: '2026-01-01T00:00:00.000Z',
    review_count: 0,
  };
}

describe('mistake store save state', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('prevents duplicate saves while a mistake is already saving', async () => {
    let resolveSave!: (value: boolean) => void;
    vi.mocked(mistakeService.saveMistake).mockReturnValue(
      new Promise((resolve) => {
        resolveSave = resolve;
      })
    );
    const store = useMistakeStore();
    const entry = mistake();

    const first = store.saveEntry('q1', entry);
    const second = await store.saveEntry('q1', entry);
    resolveSave(true);
    const firstResult = await first;

    expect(second).toBe(false);
    expect(firstResult).toBe(true);
    expect(mistakeService.saveMistake).toHaveBeenCalledTimes(1);
    expect(store.isSaved('q1')).toBe(true);
    expect(store.isSaving('q1')).toBe(false);
  });

  it('records save errors without marking the mistake as saved', async () => {
    vi.mocked(mistakeService.saveMistake).mockRejectedValue(new Error('disk full'));
    const store = useMistakeStore();

    const result = await store.saveEntry('q1', mistake());

    expect(result).toBe(false);
    expect(store.isSaved('q1')).toBe(false);
    expect(store.isSaving('q1')).toBe(false);
    expect(store.errorFor('q1')).toContain('disk full');
  });

  it('can clear per-quiz save state so reused question ids can be saved again', async () => {
    vi.mocked(mistakeService.saveMistake).mockResolvedValue(true);
    const store = useMistakeStore();

    await store.saveEntry('q1', mistake('m1'));
    store.clearSaveState();
    const result = await store.saveEntry('q1', mistake('m2'));

    expect(result).toBe(true);
    expect(mistakeService.saveMistake).toHaveBeenCalledTimes(2);
  });

  it('prepends a successfully saved mistake to the current list', async () => {
    vi.mocked(mistakeService.saveMistake).mockResolvedValue(true);
    const store = useMistakeStore();
    store.items = [mistake('old')];
    const entry = mistake('new');

    const result = await store.saveEntry('q1', entry);

    expect(result).toBe(true);
    expect(store.items.map((item) => item.id)).toEqual(['new', 'old']);
  });

  it('replaces an existing listed mistake when saving the same mistake id again', async () => {
    vi.mocked(mistakeService.saveMistake).mockResolvedValue(true);
    const store = useMistakeStore();
    const older = mistake('same');
    older.user_answer = 'A';
    const newer = mistake('same');
    newer.user_answer = 'C';
    store.items = [older, mistake('other')];

    await store.saveEntry('q1', newer);

    expect(store.items.map((item) => item.id)).toEqual(['same', 'other']);
    expect(store.items[0].user_answer).toBe('C');
  });

  it('does not prepend a saved mistake that does not match the active search filter', async () => {
    vi.mocked(mistakeService.saveMistake).mockResolvedValue(true);
    const store = useMistakeStore();
    store.searchText = 'borrowing';
    store.items = [mistake('visible')];
    const entry = mistake('ownership');
    entry.question = 'What moves a String?';
    entry.explanation = 'Ownership transfer moves the value.';

    const result = await store.saveEntry('q1', entry);

    expect(result).toBe(true);
    expect(store.items.map((item) => item.id)).toEqual(['visible']);
  });

  it('does not prepend a saved mistake that does not match the active blind spot tag filter', async () => {
    vi.mocked(mistakeService.saveMistake).mockResolvedValue(true);
    const store = useMistakeStore();
    store.blindSpotTag = 'borrowing';
    store.items = [mistake('visible')];
    const entry = mistake('ownership', 'advanced');
    entry.diagnosis = {
      rounds: 1,
      conversation: [],
      final_report: {
        summary: 'Ownership issue',
        blind_spots: [
          {
            tag: 'Ownership transfer',
            severity: 'high',
            description: 'Move semantics are confused.',
            note_reference: 'Ownership notes',
            suggestion: 'Review moves.',
          },
        ],
        overall_level: 'Needs review',
        next_steps: [],
      },
    };

    const result = await store.saveEntry('q1', entry);

    expect(result).toBe(true);
    expect(store.items.map((item) => item.id)).toEqual(['visible']);
  });

  it('removes an existing listed mistake when a saved replacement no longer matches the active search filter', async () => {
    vi.mocked(mistakeService.saveMistake).mockResolvedValue(true);
    const store = useMistakeStore();
    store.searchText = 'borrowing';
    const older = mistake('same');
    older.question = 'What is borrowing?';
    const newer = mistake('same');
    newer.question = 'What moves a String?';
    newer.explanation = 'Ownership transfer moves the value.';
    store.items = [older, mistake('other')];

    const result = await store.saveEntry('q1', newer);

    expect(result).toBe(true);
    expect(store.items.map((item) => item.id)).toEqual(['other']);
  });

  it('loads the first page with server-side mode filters', async () => {
    vi.mocked(mistakeService.loadMistakes).mockResolvedValue([mistake('m2', 'advanced')]);
    const store = useMistakeStore();

    await store.setModeFilter('advanced');

    expect(mistakeService.loadMistakes).toHaveBeenCalledWith({
      mode: 'advanced',
      offset: 0,
      limit: 20,
    });
    expect(store.items.map((item) => item.id)).toEqual(['m2']);
    expect(store.hasMore).toBe(false);
  });

  it('loads additional pages by appending mistakes with the next offset', async () => {
    vi.mocked(mistakeService.loadMistakes)
      .mockResolvedValueOnce(Array.from({ length: 20 }, (_, i) => mistake(`m${i + 1}`)))
      .mockResolvedValueOnce([mistake('m21')]);
    const store = useMistakeStore();

    await store.loadPage();
    await store.loadNextPage();

    expect(mistakeService.loadMistakes).toHaveBeenNthCalledWith(1, {
      mode: undefined,
      offset: 0,
      limit: 20,
    });
    expect(mistakeService.loadMistakes).toHaveBeenNthCalledWith(2, {
      mode: undefined,
      offset: 20,
      limit: 20,
    });
    expect(store.items).toHaveLength(21);
    expect(store.hasMore).toBe(false);
  });

  it('loads pages with the active blind spot tag filter', async () => {
    vi.mocked(mistakeService.loadMistakes)
      .mockResolvedValueOnce(Array.from({ length: 20 }, (_, i) => mistake(`m${i + 1}`, 'advanced')))
      .mockResolvedValueOnce([mistake('m21', 'advanced')]);
    const store = useMistakeStore();

    await store.setBlindSpotTag('ownership');
    await store.loadNextPage();

    expect(mistakeService.loadMistakes).toHaveBeenNthCalledWith(1, {
      mode: undefined,
      search_text: undefined,
      blind_spot_tag: 'ownership',
      offset: 0,
      limit: 20,
    });
    expect(mistakeService.loadMistakes).toHaveBeenNthCalledWith(2, {
      mode: undefined,
      search_text: undefined,
      blind_spot_tag: 'ownership',
      offset: 20,
      limit: 20,
    });
    expect(store.items).toHaveLength(21);
  });

  it('ignores stale first-page loads when a newer filter finishes first', async () => {
    const firstLoad = deferredMistakes();
    const secondLoad = deferredMistakes();
    vi.mocked(mistakeService.loadMistakes)
      .mockReturnValueOnce(firstLoad.promise)
      .mockReturnValueOnce(secondLoad.promise);
    const store = useMistakeStore();

    const oldSearch = store.setSearchText('ownership');
    const newSearch = store.setSearchText('borrowing');

    secondLoad.resolve([mistake('new')]);
    await newSearch;
    expect(store.items.map((item) => item.id)).toEqual(['new']);
    expect(store.searchText).toBe('borrowing');
    expect(store.loading).toBe(false);

    firstLoad.resolve([mistake('old')]);
    await oldSearch;

    expect(store.items.map((item) => item.id)).toEqual(['new']);
    expect(store.searchText).toBe('borrowing');
    expect(store.loading).toBe(false);
  });

  it('records detailed list load errors from the service', async () => {
    vi.mocked(mistakeService.loadMistakes).mockRejectedValue(new Error('HTTP 500: Failed to parse mistakes.json'));
    const store = useMistakeStore();

    await store.loadPage();

    expect(store.listError).toContain('Failed to parse mistakes.json');
    expect(store.loading).toBe(false);
  });

  it('updates a listed mistake after marking it reviewed', async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-06-26T12:30:00.000Z'));
    vi.mocked(mistakeService.markMistakeReviewed).mockResolvedValue(true);
    const store = useMistakeStore();
    store.items = [mistake('m1'), mistake('m2')];

    const result = await store.markReviewed('m1');

    expect(result).toBe(true);
    expect(mistakeService.markMistakeReviewed).toHaveBeenCalledWith('m1');
    expect(store.items[0].review_count).toBe(1);
    expect(store.items[0].last_reviewed_at).toBe('2026-06-26T12:30:00.000Z');
    expect(store.items[1].review_count).toBe(0);
  });

  it('records review errors and clears the reviewing state', async () => {
    vi.mocked(mistakeService.markMistakeReviewed).mockRejectedValue(new Error('disk full'));
    const store = useMistakeStore();

    const result = await store.markReviewed('m1');

    expect(result).toBe(false);
    expect(store.isReviewing('m1')).toBe(false);
    expect(store.reviewErrorFor('m1')).toBe('disk full');
  });
});

function deferredMistakes() {
  let resolve!: (value: MistakeEntry[]) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<MistakeEntry[]>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe('mistake store export', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('delegates JSON export to the service and returns true on success', async () => {
    vi.mocked(mistakeService.exportMistakes).mockResolvedValue(undefined);
    const store = useMistakeStore();

    const result = await store.exportMistakes('json');

    expect(result).toBe(true);
    expect(mistakeService.exportMistakes).toHaveBeenCalledWith('json');
  });

  it('delegates Markdown export to the service and returns true on success', async () => {
    vi.mocked(mistakeService.exportMistakes).mockResolvedValue(undefined);
    const store = useMistakeStore();

    const result = await store.exportMistakes('markdown');

    expect(result).toBe(true);
    expect(mistakeService.exportMistakes).toHaveBeenCalledWith('markdown');
  });

  it('sets isExporting true during export and false after completing', async () => {
    vi.mocked(mistakeService.exportMistakes).mockResolvedValue(undefined);
    const store = useMistakeStore();

    const promise = store.exportMistakes('json');
    expect(store.isExporting).toBe(true);
    await promise;
    expect(store.isExporting).toBe(false);
  });

  it('captures export errors and returns false', async () => {
    vi.mocked(mistakeService.exportMistakes).mockRejectedValue(new Error('No mistakes to export'));
    const store = useMistakeStore();

    const result = await store.exportMistakes('json');

    expect(result).toBe(false);
    expect(store.exportError).toBe('No mistakes to export');
    expect(store.isExporting).toBe(false);
  });

  it('clears a previous export error on a new export attempt', async () => {
    vi.mocked(mistakeService.exportMistakes)
      .mockRejectedValueOnce(new Error('First failure'))
      .mockResolvedValueOnce(undefined);
    const store = useMistakeStore();

    await store.exportMistakes('json');
    expect(store.exportError).toBe('First failure');

    const result = await store.exportMistakes('json');
    expect(result).toBe(true);
    expect(store.exportError).toBeNull();
  });

  it('handles non-Error exceptions during export', async () => {
    vi.mocked(mistakeService.exportMistakes).mockRejectedValue('Unknown failure');
    const store = useMistakeStore();

    const result = await store.exportMistakes('json');

    expect(result).toBe(false);
    expect(store.exportError).toBe('Unknown failure');
  });
});
