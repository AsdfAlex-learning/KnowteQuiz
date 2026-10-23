import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { useExplorationStore } from './exploration';
import * as explorationService from '../services/exploration';
import type { Domain, ExplorationSession, KnowledgeGraph } from '../types/exploration';

vi.mock('../services/exploration', () => ({
  createExplorationSession: vi.fn(),
  listExplorationSessions: vi.fn(),
  getExplorationSession: vi.fn(),
  deleteExplorationSession: vi.fn(),
  startExplorationStream: vi.fn(),
  confirmExplorationDomains: vi.fn(),
  askExploration: vi.fn(),
  nextExplorationQuestion: vi.fn(),
  undoExploration: vi.fn(),
  redoExploration: vi.fn(),
  finishExploration: vi.fn(),
  skipExploration: vi.fn(),
  askExplorationAside: vi.fn(),
  exportExplorationSession: vi.fn(),
  saveExplorationToVault: vi.fn(),
  importExplorationHistory: vi.fn(),
  continueExploration: vi.fn(),
}));

function graph(): KnowledgeGraph {
  return {
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
    domains: [{ id: 'd1', title: 'Compilers', concept_ids: ['c1', 'c2'] }],
    root_order: ['c1', 'c2'],
    domains_confirmed: true,
  };
}

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

  it('creates a session, runs initialization and clears the selection', async () => {
    vi.mocked(explorationService.createExplorationSession).mockResolvedValue(session());
    vi.mocked(explorationService.startExplorationStream).mockImplementation(async (_id, handlers) => {
      handlers.onPhase?.('merging');
      handlers.onGraphReady?.(graph());
    });
    vi.mocked(explorationService.getExplorationSession).mockResolvedValue(
      session({ status: 'active', graph: graph() })
    );

    const store = useExplorationStore();
    store.selectAll(['a.md', 'b.md']);

    await store.startSession();

    expect(explorationService.createExplorationSession).toHaveBeenCalledWith({
      source_paths: ['a.md', 'b.md'],
    });
    expect(explorationService.startExplorationStream).toHaveBeenCalledWith('s1', expect.any(Object));
    expect(store.session?.status).toBe('active');
    expect(store.hasGraph).toBe(true);
    expect(store.selectedPaths).toEqual([]);
    expect(store.streaming).toBe(false);
  });

  it('keeps the selection when session creation fails', async () => {
    vi.mocked(explorationService.createExplorationSession).mockRejectedValue(new Error('backend down'));
    const store = useExplorationStore();
    store.selectAll(['a.md']);

    await store.startSession();

    expect(store.session).toBeNull();
    expect(store.selectedPaths).toEqual(['a.md']);
    expect(store.error).toContain('backend down');
    expect(explorationService.startExplorationStream).not.toHaveBeenCalled();
  });

  it('does not create a session without a selection', async () => {
    const store = useExplorationStore();

    await store.startSession();

    expect(explorationService.createExplorationSession).not.toHaveBeenCalled();
  });

  it('blocks on domain confirmation when initialization detects several areas', async () => {
    const domains: Domain[] = [
      { id: 'd1', title: 'Compilers', concept_ids: ['c1'] },
      { id: 'd2', title: 'Cooking', concept_ids: ['c2'] },
    ];
    vi.mocked(explorationService.createExplorationSession).mockResolvedValue(session());
    vi.mocked(explorationService.startExplorationStream).mockImplementation(async (_id, handlers) => {
      handlers.onDomainWarning?.(domains, 'multiple_domains');
    });
    vi.mocked(explorationService.getExplorationSession).mockResolvedValue(
      session({ status: 'awaiting_domain_confirm', graph: graph(), domain_warning: 'multiple_domains' })
    );

    const store = useExplorationStore();
    store.selectAll(['a.md', 'b.md']);
    await store.startSession();

    expect(store.pendingDomains).toEqual(domains);
    expect(store.session?.status).toBe('awaiting_domain_confirm');
    expect(store.isInitializing).toBe(true);
  });

  it('confirms domains and activates the session', async () => {
    vi.mocked(explorationService.confirmExplorationDomains).mockImplementation(async (_id, _request, handlers) => {
      handlers.onGraphReady?.(graph());
    });
    vi.mocked(explorationService.getExplorationSession).mockResolvedValue(
      session({ status: 'active', graph: graph() })
    );

    const store = useExplorationStore();
    store.session = session({ status: 'awaiting_domain_confirm', graph: graph() });
    store.pendingDomains = graph().domains;

    await store.confirmDomains(graph().domains);

    expect(explorationService.confirmExplorationDomains).toHaveBeenCalledWith(
      's1',
      { domains: graph().domains },
      expect.any(Object)
    );
    expect(store.pendingDomains).toBeNull();
    expect(store.session?.status).toBe('active');
  });

  it('surfaces initialization errors and allows a retry', async () => {
    vi.mocked(explorationService.createExplorationSession).mockResolvedValue(session());
    vi.mocked(explorationService.startExplorationStream).mockImplementation(async (_id, handlers) => {
      handlers.onError?.('model unreachable');
    });
    vi.mocked(explorationService.getExplorationSession).mockResolvedValue(session());

    const store = useExplorationStore();
    store.selectAll(['a.md']);
    await store.startSession();

    expect(store.error).toContain('model unreachable');
    expect(store.streaming).toBe(false);
  });

  it('derives coverage and the visible whiteboard from the active timeline', () => {
    const store = useExplorationStore();
    store.session = session({
      status: 'active',
      graph: graph(),
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

  it('submits an answer and replaces the session', async () => {
    const updated = session({ status: 'active', graph: graph(), active_group: 1 });
    vi.mocked(explorationService.askExploration).mockResolvedValue(updated);
    const store = useExplorationStore();
    store.session = session({ status: 'active', graph: graph() });

    const ok = await store.answer('  Because.  ');

    expect(ok).toBe(true);
    expect(explorationService.askExploration).toHaveBeenCalledWith('s1', 'Because.');
    expect(store.session).toEqual(updated);
    expect(store.thinking).toBe(false);
  });

  it('keeps the session and reports errors when an answer fails', async () => {
    vi.mocked(explorationService.askExploration).mockRejectedValue(new Error('model down'));
    const store = useExplorationStore();
    const original = session({ status: 'active', graph: graph() });
    store.session = original;

    const ok = await store.answer('Because.');

    expect(ok).toBe(false);
    expect(store.session).toEqual(original);
    expect(store.error).toContain('model down');
  });

  it('requests the next question', async () => {
    const updated = session({ status: 'active', graph: graph(), active_group: 1 });
    vi.mocked(explorationService.nextExplorationQuestion).mockResolvedValue(updated);
    const store = useExplorationStore();
    store.session = session({ status: 'active', graph: graph() });

    const ok = await store.requestNextQuestion();

    expect(ok).toBe(true);
    expect(explorationService.nextExplorationQuestion).toHaveBeenCalledWith('s1');
  });

  it('undoes through the service and replaces the session', async () => {
    const undone = session({ status: 'active', graph: graph(), active_group: 0 });
    vi.mocked(explorationService.undoExploration).mockResolvedValue(undone);
    const store = useExplorationStore();
    store.session = session({ status: 'active', graph: graph(), active_group: 1 });

    const ok = await store.undo();

    expect(ok).toBe(true);
    expect(explorationService.undoExploration).toHaveBeenCalledWith('s1');
    expect(store.session?.active_group).toBe(0);
  });

  it('asks a side question through the aside service', async () => {
    const withAside = session({ status: 'active', graph: graph() });
    withAside.asides = [
      {
        id: 'a1',
        question: 'Why?',
        answer_md: 'Because.',
        group_index: 0,
        created_at: '2026-10-23T10:00:00+08:00',
      },
    ];
    vi.mocked(explorationService.askExplorationAside).mockResolvedValue(withAside);
    const store = useExplorationStore();
    store.session = session({ status: 'active', graph: graph() });

    const ok = await store.askAside('  Why?  ');

    expect(ok).toBe(true);
    expect(explorationService.askExplorationAside).toHaveBeenCalledWith('s1', 'Why?');
    expect(store.session?.asides).toHaveLength(1);
  });

  it('exports the session and records the result', async () => {
    const result = {
      path: 'C:/data/exploration/exports/x.md',
      file_name: 'x.md',
      markdown: '# history',
    };
    vi.mocked(explorationService.exportExplorationSession).mockResolvedValue(result);
    const store = useExplorationStore();
    store.session = session({ status: 'completed', graph: graph() });

    const ok = await store.exportSession();

    expect(ok).toBe(true);
    expect(explorationService.exportExplorationSession).toHaveBeenCalledWith('s1');
    expect(store.lastExport).toEqual(result);
  });

  it('imports a history as a read-only session', async () => {
    const imported = session({
      status: 'completed',
      graph: graph(),
      origin: { kind: 'imported', source_path: 'friend.md' },
    });
    vi.mocked(explorationService.importExplorationHistory).mockResolvedValue(imported);
    const store = useExplorationStore();

    const ok = await store.importHistory('# history', 'friend.md');

    expect(ok).toBe(true);
    expect(explorationService.importExplorationHistory).toHaveBeenCalledWith({
      markdown: '# history',
      source_name: 'friend.md',
    });
    expect(store.isReadOnly).toBe(true);
  });

  it('continues an imported session', async () => {
    const continued = session({
      status: 'active',
      graph: graph(),
      origin: { kind: 'imported', source_path: 'friend.md' },
    });
    vi.mocked(explorationService.continueExploration).mockResolvedValue(continued);
    const store = useExplorationStore();
    store.session = session({
      status: 'completed',
      graph: graph(),
      origin: { kind: 'imported', source_path: 'friend.md' },
    });

    const ok = await store.continueSession();

    expect(ok).toBe(true);
    expect(store.session?.status).toBe('active');
    expect(store.isReadOnly).toBe(false);
    expect(store.isReplaying).toBe(false);
  });

  it('replays the timeline step by step with the matching whiteboard', () => {
    const store = useExplorationStore();
    store.session = session({
      status: 'completed',
      graph: graph(),
      active_group: 2,
      groups: [
        {
          concept_id: 'c1',
          question: { prompt_md: 'Q1', hint_used: false },
          answer: 'A1',
          result: null,
          whiteboard: {
            id: 'w1',
            concept_id: 'c1',
            title: '',
            content_md: 'one',
            group_index: 0,
          },
          covered_after: ['c1'],
        },
        {
          concept_id: 'c2',
          question: { prompt_md: 'Q2', hint_used: false },
          answer: 'A2',
          result: null,
          whiteboard: {
            id: 'w2',
            concept_id: 'c2',
            title: '',
            content_md: 'two',
            group_index: 1,
          },
          covered_after: ['c2'],
        },
      ],
    });

    store.startReplay();
    expect(store.isReplaying).toBe(true);
    expect(store.displayedGroupCount).toBe(2);
    expect(store.currentWhiteboard?.id).toBe('w2');

    store.replayPrev();
    expect(store.displayedGroupCount).toBe(1);
    expect(store.currentWhiteboard?.id).toBe('w1');

    store.replayPrev();
    expect(store.displayedGroupCount).toBe(0);
    expect(store.currentWhiteboard).toBeNull();

    store.replayPrev();
    expect(store.displayedGroupCount).toBe(0);

    store.replayNext();
    expect(store.displayedGroupCount).toBe(1);

    store.stopReplay();
    expect(store.isReplaying).toBe(false);
    expect(store.displayedGroupCount).toBe(2);
  });
});
