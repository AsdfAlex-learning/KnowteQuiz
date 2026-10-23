import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useExplorationStore } from './exploration';
import * as explorationService from '../services/exploration';
import type { ExplorationSession } from '../types/exploration';

vi.mock('../services/exploration', () => ({
  createExplorationSession: vi.fn(),
  listExplorationSessions: vi.fn(),
  getExplorationSession: vi.fn(),
  deleteExplorationSession: vi.fn(),
}));

function session(overrides: Partial<ExplorationSession> = {}): ExplorationSession {
  return {
    id: 's1',
    title: 'Compiler',
    format_version: 1,
    created_at: '2026-10-23T10:00:00+08:00',
    updated_at: '2026-10-23T10:00:00+08:00',
    status: 'initializing',
    origin: { kind: 'new' },
    source_paths: ['a.md', 'b.md'],
    graph: { concepts: [], domains: [], root_order: [], domains_confirmed: false },
    groups: [],
    active_group: 0,
    domain_warning: null,
    completed_early: false,
    ...overrides,
  };
}

describe('exploration store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('starts with nothing selected', () => {
    const store = useExplorationStore();

    expect(store.selectedCount).toBe(0);
    expect(store.canStart).toBe(false);
  });

  it('toggles a note on and off', () => {
    const store = useExplorationStore();

    store.toggleSelection('a.md');
    expect(store.isSelected('a.md')).toBe(true);
    expect(store.canStart).toBe(true);

    store.toggleSelection('a.md');
    expect(store.isSelected('a.md')).toBe(false);
    expect(store.canStart).toBe(false);
  });

  it('deduplicates select-all input', () => {
    const store = useExplorationStore();

    store.selectAll(['a.md', 'b.md', 'a.md']);

    expect(store.selectedPaths).toEqual(['a.md', 'b.md']);
    expect(store.selectedCount).toBe(2);
  });

  it('clears and resets the selection', () => {
    const store = useExplorationStore();
    store.selectAll(['a.md', 'b.md']);

    store.clearSelection();
    expect(store.selectedPaths).toEqual([]);

    store.selectAll(['c.md']);
    store.reset();
    expect(store.selectedPaths).toEqual([]);
    expect(store.session).toBeNull();
  });

  it('creates a session from the selection and clears it', async () => {
    vi.mocked(explorationService.createExplorationSession).mockResolvedValue(session());
    const store = useExplorationStore();
    store.selectAll(['a.md', 'b.md']);

    await store.startSession();

    expect(explorationService.createExplorationSession).toHaveBeenCalledWith({
      source_paths: ['a.md', 'b.md'],
    });
    expect(store.session?.id).toBe('s1');
    expect(store.selectedPaths).toEqual([]);
    expect(store.isInitializing).toBe(true);
  });

  it('keeps the selection when session creation fails', async () => {
    vi.mocked(explorationService.createExplorationSession).mockRejectedValue(new Error('backend down'));
    const store = useExplorationStore();
    store.selectAll(['a.md']);

    await store.startSession();

    expect(store.session).toBeNull();
    expect(store.selectedPaths).toEqual(['a.md']);
    expect(store.error).toContain('backend down');
  });

  it('does not create a session without a selection', async () => {
    const store = useExplorationStore();

    await store.startSession();

    expect(explorationService.createExplorationSession).not.toHaveBeenCalled();
  });

  it('derives coverage and the visible whiteboard from the active timeline', () => {
    const store = useExplorationStore();
    store.session = session({
      status: 'active',
      graph: {
        concepts: [
          {
            id: 'c1',
            title: 'One',
            summary: '',
            depth: 0,
            prereqs: [],
            source_refs: [],
            covered: true,
          },
          {
            id: 'c2',
            title: 'Two',
            summary: '',
            depth: 1,
            prereqs: ['c1'],
            source_refs: [],
            covered: false,
          },
        ],
        domains: [],
        root_order: ['c1'],
        domains_confirmed: true,
      },
      groups: [
        {
          concept_id: 'c1',
          question: { prompt_md: 'Why?', hint_used: false },
          answer: 'Because.',
          result: null,
          whiteboard: {
            id: 'wb1',
            concept_id: 'c1',
            title: 'Board',
            content_md: '| a | b |',
            group_index: 0,
          },
          covered_after: ['c1'],
        },
        {
          concept_id: 'c2',
          question: { prompt_md: 'And then?', hint_used: false },
          answer: null,
          result: null,
          whiteboard: null,
          covered_after: [],
        },
      ],
      active_group: 1,
    });

    expect(store.coverage).toEqual({ covered: 1, total: 2 });
    expect(store.currentWhiteboard?.id).toBe('wb1');
    expect(store.isInitializing).toBe(false);
  });

  it('deletes a session from the list and clears it when active', async () => {
    vi.mocked(explorationService.deleteExplorationSession).mockResolvedValue(true);
    const store = useExplorationStore();
    store.session = session();
    store.sessions = [
      {
        id: 's1',
        title: 'Compiler',
        status: 'initializing',
        created_at: '2026-10-23T10:00:00+08:00',
        updated_at: '2026-10-23T10:00:00+08:00',
        source_paths: ['a.md'],
        concepts_total: 0,
        concepts_covered: 0,
        group_count: 0,
        completed_early: false,
      },
    ];

    await store.removeSession('s1');

    expect(store.sessions).toEqual([]);
    expect(store.session).toBeNull();
  });
});
