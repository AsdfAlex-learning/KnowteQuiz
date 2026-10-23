import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ExplorationShell from './ExplorationShell.vue';
import { useExplorationStore } from '@/stores/exploration';

describe('ExplorationShell', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('explains the flow and shows an empty selection state', () => {
    const wrapper = mount(ExplorationShell);

    expect(wrapper.text()).toContain('Exploratory learning');
    expect(wrapper.text()).toContain('No notes selected');
    expect(wrapper.text()).toContain('Select Markdown notes in the explorer.');
  });

  it('shows the selected note count', async () => {
    const explorationStore = useExplorationStore();
    const wrapper = mount(ExplorationShell);

    explorationStore.selectAll(['a.md', 'b.md']);
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain('2 note(s) selected');
    expect(wrapper.text()).not.toContain('No notes selected');
  });
});
