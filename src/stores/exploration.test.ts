import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import { useExplorationStore } from './exploration';

describe('exploration store selection', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
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
  });
});
