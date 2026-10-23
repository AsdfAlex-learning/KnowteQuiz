import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ExplorationGraph from './ExplorationGraph.vue';
import type { KnowledgeGraph } from '@/types/exploration';

function concept(id: string, title: string, depth: number) {
  return {
    id,
    title,
    summary: '',
    depth,
    prereqs: [],
    source_refs: [],
    covered: false,
  };
}

describe('ExplorationGraph', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('renders concepts in root_order and falls back to the rest', () => {
    const graph: KnowledgeGraph = {
      concepts: [concept('c1', 'One', 0), concept('c2', 'Two', 1), concept('c3', 'Three', 0)],
      domains: [{ id: 'd1', title: 'Compilers', concept_ids: ['c1', 'c2', 'c3'] }],
      root_order: ['c3', 'c1'],
      domains_confirmed: true,
    };

    const wrapper = mount(ExplorationGraph, { props: { graph } });
    const text = wrapper.text();

    expect(text.indexOf('Three')).toBeLessThan(text.indexOf('One'));
    expect(text.indexOf('One')).toBeLessThan(text.indexOf('Two'));
    expect(text).toContain('Compilers · 3');
  });

  it('indents concepts by depth', () => {
    const graph: KnowledgeGraph = {
      concepts: [concept('c1', 'One', 0), concept('c2', 'Two', 2)],
      domains: [],
      root_order: ['c1', 'c2'],
      domains_confirmed: true,
    };

    const wrapper = mount(ExplorationGraph, { props: { graph } });
    const rows = wrapper.findAll('[style]');

    expect(rows[0].attributes('style')).toContain('margin-left: 0px');
    expect(rows[1].attributes('style')).toContain('margin-left: 28px');
  });
});
