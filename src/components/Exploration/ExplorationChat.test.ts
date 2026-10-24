import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ExplorationChat from './ExplorationChat.vue';
import { useExplorationStore } from '@/stores/exploration';
import type { ExplorationSession } from '@/types/exploration';

function session(): ExplorationSession {
  return {
    id: 's1',
    title: 'Compiler',
    format_version: 1,
    created_at: '2026-10-23T10:00:00+08:00',
    updated_at: '2026-10-23T10:00:00+08:00',
    status: 'active',
    origin: { kind: 'new' },
    baseline: 50,
    source_paths: ['a.md'],
    graph: { concepts: [], domains: [], root_order: [], domains_confirmed: false },
    groups: [],
    active_group: 0,
    asides: [],
    skipped: [],
    domain_warning: null,
    completed_early: false,
  };
}

describe('ExplorationChat', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('marks a session that was auto-resumed', () => {
    const store = useExplorationStore();
    store.session = session();
    store.autoResumed = true;

    const wrapper = mount(ExplorationChat);

    expect(wrapper.text()).toContain('Compiler');
    expect(wrapper.text()).toContain('Restored');
  });

  it('does not show the restored badge for a fresh session', () => {
    const store = useExplorationStore();
    store.session = session();
    store.autoResumed = false;

    const wrapper = mount(ExplorationChat);

    expect(wrapper.text()).not.toContain('Restored');
  });
});
