import { mount, type VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ExplorationShell from './ExplorationShell.vue';
import { useExplorationStore } from '@/stores/exploration';

vi.mock('@/services/exploration', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/services/exploration')>();
  return {
    ...actual,
    listExplorationSessions: vi.fn().mockResolvedValue([]),
  };
});

function startButton(wrapper: VueWrapper) {
  return wrapper.findAll('button').find((button) => button.text().includes('Start exploring'));
}

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

  it('enables the start button only when notes are selected', async () => {
    const wrapper = mount(ExplorationShell);

    expect(startButton(wrapper)?.attributes('disabled')).toBeDefined();

    useExplorationStore().selectAll(['a.md']);
    await wrapper.vm.$nextTick();

    expect(startButton(wrapper)?.attributes('disabled')).toBeUndefined();
  });

  it('swaps the onboarding screen for the session view', async () => {
    const explorationStore = useExplorationStore();
    const wrapper = mount(ExplorationShell);

    explorationStore.session = {
      id: 's1',
      title: 'Compiler',
      format_version: 1,
      created_at: '2026-10-23T10:00:00+08:00',
      updated_at: '2026-10-23T10:00:00+08:00',
      status: 'initializing',
      origin: { kind: 'new' },
      baseline: 50,
      source_paths: ['a.md', 'b.md'],
      graph: { concepts: [], domains: [], root_order: [], domains_confirmed: false },
      groups: [],
      active_group: 0,
      domain_warning: null,
      completed_early: false,
    };
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain('Compiler');
    expect(wrapper.text()).toContain('Building the knowledge map…');
    expect(wrapper.text()).not.toContain('Start exploring');
  });
});
