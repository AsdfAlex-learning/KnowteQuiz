import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import {
  createExplorationSession,
  deleteExplorationSession,
  getExplorationSession,
  listExplorationSessions,
} from '../services/exploration';
import type { ExplorationSession, ExplorationSessionSummary, WhiteboardStep } from '../types/exploration';

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
  const loadingSessions = ref(false);
  const error = ref<string | null>(null);

  const selectedCount = computed(() => selectedPaths.value.length);
  const canStart = computed(() => selectedPaths.value.length > 0 && !creating.value);

  const isInitializing = computed(
    () => session.value?.status === 'initializing' || session.value?.status === 'awaiting_domain_confirm'
  );

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
    const active = current.groups.slice(0, current.active_group);
    for (let i = active.length - 1; i >= 0; i -= 1) {
      if (active[i].whiteboard) return active[i].whiteboard ?? null;
    }
    return null;
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
    } catch (e) {
      error.value = String(e);
    } finally {
      creating.value = false;
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
    session.value = null;
  }

  function reset() {
    selectedPaths.value = [];
    session.value = null;
    sessions.value = [];
    error.value = null;
  }

  return {
    selectedPaths,
    session,
    sessions,
    creating,
    loadingSessions,
    error,
    selectedCount,
    canStart,
    isInitializing,
    coverage,
    currentWhiteboard,
    isSelected,
    toggleSelection,
    selectAll,
    clearSelection,
    startSession,
    loadSessions,
    openSession,
    removeSession,
    closeSession,
    reset,
  };
});
