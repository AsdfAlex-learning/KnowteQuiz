import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import ExplorationShell from './ExplorationShell.vue';
import { useExplorationStore } from '@/stores/exploration';
import * as explorationService from '@/services/exploration';
import * as settingsService from '@/services/settings';
import type { ExplorationSession, ExplorationSessionSummary } from '@/types/exploration';

vi.mock('@/services/exploration', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/services/exploration')>();
  return {
    ...actual,
    listExplorationSessions: vi.fn(),
    createExplorationSession: vi.fn(),
    startExplorationStream: vi.fn(),
    getExplorationSession: vi.fn(),
    deleteExplorationSession: vi.fn(),
  };
});

vi.mock('@/services/settings', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/services/settings')>();
  return {
    ...actual,
    getSettings: vi.fn(),
    saveSettings: vi.fn().mockResolvedValue(true),
  };
});

function startButton(wrapper: VueWrapper) {
  return wrapper.findAll('button').find((button) => button.text().includes('Start exploring'));
}

function summary(overrides: Partial<ExplorationSessionSummary> = {}): ExplorationSessionSummary {
  return {
    id: 's1',
    title: 'Compiler',
    status: 'active',
    created_at: '2026-10-23T10:00:00+08:00',
    updated_at: '2026-10-23T10:00:00+08:00',
    source_paths: ['a.md'],
    concepts_total: 1,
    concepts_covered: 0,
    group_count: 0,
    completed_early: false,
    ...overrides,
  };
}

function sessionFixture(): ExplorationSession {
  return {
    id: 's1',
    title: 'Compiler',
    format_version: 1,
    created_at: '2026-10-23T10:00:00+08:00',
    updated_at: '2026-10-23T10:00:00+08:00',
    status: 'active',
    origin: { kind: 'new' },
    baseline: 25,
    source_paths: ['a.md'],
    graph: {
      concepts: [
        {
          id: 'c1',
          title: 'One',
          summary: '',
          depth: 0,
          prereqs: [],
          source_refs: [],
          covered: false,
        },
      ],
      domains: [],
      root_order: ['c1'],
      domains_confirmed: true,
    },
    groups: [],
    active_group: 0,
    asides: [],
    skipped: [],
    domain_warning: null,
    completed_early: false,
  };
}

describe('ExplorationShell', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    vi.mocked(explorationService.listExplorationSessions).mockResolvedValue([]);
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

  it('persists and sends the baseline chosen on the slider', async () => {
    vi.mocked(explorationService.createExplorationSession).mockResolvedValue(sessionFixture());
    vi.mocked(explorationService.startExplorationStream).mockResolvedValue(undefined);
    vi.mocked(explorationService.getExplorationSession).mockResolvedValue(sessionFixture());
    useExplorationStore().selectAll(['a.md']);
    const wrapper = mount(ExplorationShell);

    await wrapper.find('input[type="range"]').setValue('25');
    await startButton(wrapper)?.trigger('click');
    await flushPromises();

    expect(explorationService.createExplorationSession).toHaveBeenCalledWith({
      source_paths: ['a.md'],
      baseline: 25,
    });
    expect(settingsService.saveSettings).toHaveBeenCalled();
  });

  it('prompts to restore an older session matching the current note', async () => {
    vi.mocked(explorationService.listExplorationSessions).mockResolvedValue([
      summary({ updated_at: new Date(Date.now() - 3 * 24 * 60 * 60 * 1000).toISOString() }),
    ]);
    vi.mocked(explorationService.getExplorationSession).mockResolvedValue(sessionFixture());
    const wrapper = mount(ExplorationShell);
    await flushPromises();

    expect(wrapper.text()).toContain('unfinished session');
    expect(wrapper.text()).toContain('Compiler');

    const restore = wrapper.findAll('button').find((button) => button.text() === 'Restore');
    await restore?.trigger('click');
    await flushPromises();

    expect(explorationService.getExplorationSession).toHaveBeenCalledWith('s1');
  });

  it('lists session history and can delete a session', async () => {
    vi.mocked(explorationService.listExplorationSessions).mockResolvedValue([summary({ status: 'completed' })]);
    vi.mocked(explorationService.deleteExplorationSession).mockResolvedValue(true);
    const wrapper = mount(ExplorationShell);
    await flushPromises();

    expect(wrapper.text()).toContain('Session history');
    expect(wrapper.text()).toContain('Compiler');

    const remove = wrapper.findAll('button').find((button) => button.text() === 'Delete');
    await remove?.trigger('click');
    await flushPromises();

    expect(explorationService.deleteExplorationSession).toHaveBeenCalledWith('s1');
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
