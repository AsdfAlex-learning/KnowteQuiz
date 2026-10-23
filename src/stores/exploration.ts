import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import {
  askExploration,
  askExplorationAside,
  confirmExplorationDomains,
  continueExploration,
  createExplorationSession,
  deleteExplorationSession,
  exportExplorationSession,
  finishExploration,
  getExplorationSession,
  importExplorationHistory,
  listExplorationSessions,
  nextExplorationQuestion,
  redoExploration,
  saveExplorationToVault,
  skipExploration,
  startExplorationStream,
  undoExploration,
} from '../services/exploration';
import type {
  Domain,
  ExplorationExportResult,
  ExplorationSession,
  ExplorationSessionSummary,
  TurnGroup,
  WhiteboardStep,
} from '../types/exploration';
import { useRequestId } from '../composables/useRequestId';

/**
 * Exploration mode state.
 *
 * Highlights:
 * - `selectedPaths` is the source-note selection made in the explorer.
 * - `session` is the single active session (one at a time by design).
 * - Coverage and the visible whiteboard are derived from the timeline so
 *   `/undo` and `/redo` never need to patch stored state.
 */
export const useExplorationStore = defineStore('exploration', () => {
  const selectedPaths = ref<string[]>([]);
  const session = ref<ExplorationSession | null>(null);
  const sessions = ref<ExplorationSessionSummary[]>([]);
  const creating = ref(false);
  const streaming = ref(false);
  const thinking = ref(false);
  const loadingSessions = ref(false);
  const phase = ref<string | null>(null);
  const pendingDomains = ref<Domain[] | null>(null);
  const error = ref<string | null>(null);
  const lastExport = ref<ExplorationExportResult | null>(null);
  const lastVaultPath = ref<string | null>(null);
  const replayIndex = ref<number | null>(null);
  const skipAsides = ref(false);
  const streamRequest = useRequestId();

  const selectedCount = computed(() => selectedPaths.value.length);
  const canStart = computed(() => selectedPaths.value.length > 0 && !creating.value);

  const isInitializing = computed(
    () => session.value?.status === 'initializing' || session.value?.status === 'awaiting_domain_confirm'
  );

  const hasGraph = computed(() => (session.value?.graph.concepts.length ?? 0) > 0);

  const isReplaying = computed(() => replayIndex.value !== null);

  /** Number of timeline groups the UI should show (replay cursor or live). */
  const displayedGroupCount = computed(() => {
    const current = session.value;
    if (!current) return 0;
    const cursor = replayIndex.value;
    return cursor === null ? current.active_group : Math.min(cursor, current.active_group);
  });

  /** Imported histories stay read-only until the learner continues them. */
  const isReadOnly = computed(() => session.value?.origin.kind === 'imported' && session.value.status === 'completed');

  const coverage = computed(() => {
    const current = session.value;
    if (!current) return { covered: 0, total: 0 };
    const coveredIds = new Set<string>();
    for (const group of current.groups.slice(0, current.active_group)) {
      for (const id of group.covered_after) coveredIds.add(id);
    }
    return {
      covered: current.graph.concepts.filter((concept) => coveredIds.has(concept.id)).length,
      total: current.graph.concepts.length,
    };
  });

  const currentWhiteboard = computed<WhiteboardStep | null>(() => {
    const current = session.value;
    if (!current) return null;
    const active = current.groups.slice(0, displayedGroupCount.value);
    for (let i = active.length - 1; i >= 0; i -= 1) {
      if (active[i].whiteboard) return active[i].whiteboard ?? null;
    }
    return null;
  });

  /** The group waiting for an answer, if any. */
  const currentGroup = computed<TurnGroup | null>(() => {
    const current = session.value;
    if (!current) return null;
    return current.groups[current.active_group] ?? null;
  });

  function isSelected(path: string): boolean {
    return selectedPaths.value.includes(path);
  }

  function toggleSelection(path: string) {
    selectedPaths.value = isSelected(path)
      ? selectedPaths.value.filter((p) => p !== path)
      : [...selectedPaths.value, path];
  }

  function selectAll(paths: string[]) {
    selectedPaths.value = [...new Set(paths)];
  }

  function clearSelection() {
    selectedPaths.value = [];
  }

  async function startSession(title?: string): Promise<void> {
    if (!selectedPaths.value.length || creating.value) return;
    creating.value = true;
    error.value = null;
    try {
      session.value = await createExplorationSession({
        source_paths: [...selectedPaths.value],
        ...(title ? { title } : {}),
      });
      selectedPaths.value = [];
      creating.value = false;
      await runInitialization(session.value.id);
    } catch (e) {
      error.value = String(e);
    } finally {
      creating.value = false;
    }
  }

  /** Runs (or re-runs) the map-reduce initialization for a session. */
  async function runInitialization(sessionId: string): Promise<void> {
    const requestId = streamRequest.next();
    streaming.value = true;
    phase.value = 'reading';
    error.value = null;
    try {
      await startExplorationStream(sessionId, {
        onPhase: (next) => {
          if (streamRequest.isLatest(requestId)) phase.value = next;
        },
        onDomainWarning: (domains) => {
          if (!streamRequest.isLatest(requestId)) return;
          pendingDomains.value = domains;
          if (session.value) {
            session.value = {
              ...session.value,
              status: 'awaiting_domain_confirm',
              domain_warning: 'multiple_domains',
              graph: { ...session.value.graph, domains },
            };
          }
        },
        onGraphReady: (graph) => {
          if (!streamRequest.isLatest(requestId)) return;
          pendingDomains.value = null;
          if (session.value) {
            session.value = {
              ...session.value,
              status: 'active',
              graph,
              domain_warning: null,
            };
          }
        },
        onError: (message) => {
          if (streamRequest.isLatest(requestId)) error.value = message;
        },
      });
      if (streamRequest.isLatest(requestId) && session.value) {
        session.value = await getExplorationSession(sessionId);
      }
    } catch (e) {
      if (streamRequest.isLatest(requestId)) error.value = String(e);
    } finally {
      if (streamRequest.isLatest(requestId)) {
        streaming.value = false;
        phase.value = null;
      }
    }
  }

  async function retryInitialization(): Promise<void> {
    if (!session.value || streaming.value) return;
    await runInitialization(session.value.id);
  }

  /** Submits one learner answer and replaces the session with the result. */
  async function answer(text: string): Promise<boolean> {
    const current = session.value;
    const trimmed = text.trim();
    if (!current || thinking.value || !trimmed) return false;
    thinking.value = true;
    error.value = null;
    try {
      session.value = await askExploration(current.id, trimmed);
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      thinking.value = false;
    }
  }

  /** Generates the pending question when the session is active but has none. */
  async function requestNextQuestion(): Promise<boolean> {
    const current = session.value;
    if (!current || thinking.value) return false;
    thinking.value = true;
    error.value = null;
    try {
      session.value = await nextExplorationQuestion(current.id);
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      thinking.value = false;
    }
  }

  /** Runs one cursor-only session action and replaces the local snapshot. */
  async function runSessionAction(action: (sessionId: string) => Promise<ExplorationSession>): Promise<boolean> {
    const current = session.value;
    if (!current || thinking.value) return false;
    thinking.value = true;
    error.value = null;
    try {
      session.value = await action(current.id);
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    } finally {
      thinking.value = false;
    }
  }

  function undo(): Promise<boolean> {
    return runSessionAction(undoExploration);
  }

  function redo(): Promise<boolean> {
    return runSessionAction(redoExploration);
  }

  function finish(): Promise<boolean> {
    return runSessionAction(finishExploration);
  }

  function skip(): Promise<boolean> {
    return runSessionAction(skipExploration);
  }

  function askAside(question: string): Promise<boolean> {
    const trimmed = question.trim();
    if (!trimmed) return Promise.resolve(false);
    return runSessionAction((sessionId) => askExplorationAside(sessionId, trimmed));
  }

  async function exportSession(): Promise<boolean> {
    const current = session.value;
    if (!current) return false;
    error.value = null;
    try {
      lastExport.value = await exportExplorationSession(current.id);
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  async function saveToVault(fileName: string): Promise<boolean> {
    const current = session.value;
    const name = fileName.trim();
    if (!current || !name) return false;
    error.value = null;
    try {
      lastVaultPath.value = await saveExplorationToVault(current.id, name);
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  async function importHistory(markdown: string, sourceName?: string): Promise<boolean> {
    const text = markdown.trim();
    if (!text) return false;
    error.value = null;
    try {
      session.value = await importExplorationHistory({
        markdown: text,
        source_name: sourceName ?? null,
      });
      replayIndex.value = session.value.active_group;
      return true;
    } catch (e) {
      error.value = String(e);
      return false;
    }
  }

  async function continueSession(): Promise<boolean> {
    const ok = await runSessionAction(continueExploration);
    if (ok) stopReplay();
    return ok;
  }

  function startReplay() {
    const current = session.value;
    if (!current) return;
    replayIndex.value = current.active_group;
  }

  function replayPrev() {
    if (replayIndex.value === null) return;
    replayIndex.value = Math.max(0, replayIndex.value - 1);
  }

  function replayNext() {
    const current = session.value;
    if (replayIndex.value === null || !current) return;
    replayIndex.value = Math.min(current.active_group, replayIndex.value + 1);
  }

  function stopReplay() {
    replayIndex.value = null;
  }

  function toggleSkipAsides() {
    skipAsides.value = !skipAsides.value;
  }

  async function confirmDomains(domains: Domain[]): Promise<void> {
    const current = session.value;
    if (!current || streaming.value) return;
    const requestId = streamRequest.next();
    streaming.value = true;
    error.value = null;
    try {
      await confirmExplorationDomains(
        current.id,
        { domains },
        {
          onGraphReady: (graph) => {
            if (!streamRequest.isLatest(requestId) || !session.value) return;
            pendingDomains.value = null;
            session.value = {
              ...session.value,
              status: 'active',
              graph,
              domain_warning: null,
            };
          },
          onError: (message) => {
            if (streamRequest.isLatest(requestId)) error.value = message;
          },
        }
      );
      if (streamRequest.isLatest(requestId)) {
        session.value = await getExplorationSession(current.id);
      }
    } catch (e) {
      if (streamRequest.isLatest(requestId)) error.value = String(e);
    } finally {
      if (streamRequest.isLatest(requestId)) streaming.value = false;
    }
  }

  async function loadSessions(): Promise<void> {
    loadingSessions.value = true;
    error.value = null;
    try {
      sessions.value = await listExplorationSessions();
    } catch (e) {
      error.value = String(e);
    } finally {
      loadingSessions.value = false;
    }
  }

  async function openSession(sessionId: string): Promise<void> {
    error.value = null;
    try {
      session.value = await getExplorationSession(sessionId);
      pendingDomains.value = session.value.status === 'awaiting_domain_confirm' ? session.value.graph.domains : null;
    } catch (e) {
      error.value = String(e);
    }
  }

  async function removeSession(sessionId: string): Promise<void> {
    error.value = null;
    try {
      await deleteExplorationSession(sessionId);
      sessions.value = sessions.value.filter((row) => row.id !== sessionId);
      if (session.value?.id === sessionId) session.value = null;
    } catch (e) {
      error.value = String(e);
    }
  }

  function closeSession() {
    // Invalidate in-flight stream events so a slow initialization cannot
    // repopulate a session the user just left.
    streamRequest.next();
    streaming.value = false;
    phase.value = null;
    pendingDomains.value = null;
    replayIndex.value = null;
    skipAsides.value = false;
    session.value = null;
  }

  function reset() {
    closeSession();
    selectedPaths.value = [];
    sessions.value = [];
    error.value = null;
    lastExport.value = null;
    lastVaultPath.value = null;
  }

  return {
    selectedPaths,
    session,
    sessions,
    creating,
    streaming,
    thinking,
    loadingSessions,
    phase,
    pendingDomains,
    error,
    lastExport,
    lastVaultPath,
    replayIndex,
    skipAsides,
    selectedCount,
    canStart,
    isInitializing,
    hasGraph,
    isReadOnly,
    isReplaying,
    displayedGroupCount,
    coverage,
    currentWhiteboard,
    currentGroup,
    isSelected,
    toggleSelection,
    selectAll,
    clearSelection,
    startSession,
    retryInitialization,
    answer,
    requestNextQuestion,
    undo,
    redo,
    finish,
    skip,
    askAside,
    exportSession,
    saveToVault,
    importHistory,
    continueSession,
    startReplay,
    replayPrev,
    replayNext,
    stopReplay,
    toggleSkipAsides,
    confirmDomains,
    loadSessions,
    openSession,
    removeSession,
    closeSession,
    reset,
  };
});
