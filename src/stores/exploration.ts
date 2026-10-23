import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

/**
 * Exploration mode state.
 *
 * Branch 1 only tracks the source-note selection that the onboarding screen
 * needs. Session lifecycle (graph, turns, whiteboard) is added on top of this
 * store by the follow-up steps.
 */
export const useExplorationStore = defineStore('exploration', () => {
  const selectedPaths = ref<string[]>([]);

  const selectedCount = computed(() => selectedPaths.value.length);
  const canStart = computed(() => selectedPaths.value.length > 0);

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

  function reset() {
    selectedPaths.value = [];
  }

  return {
    selectedPaths,
    selectedCount,
    canStart,
    isSelected,
    toggleSelection,
    selectAll,
    clearSelection,
    reset,
  };
});
