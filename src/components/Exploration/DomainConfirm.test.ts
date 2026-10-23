import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import DomainConfirm from './DomainConfirm.vue';
import type { Domain } from '@/types/exploration';

const domains: Domain[] = [
  { id: 'd1', title: 'Compilers', concept_ids: ['c1', 'c2'] },
  { id: 'd2', title: 'Cooking', concept_ids: ['c3', 'c2'] },
];

describe('DomainConfirm', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('emits the edited domains', async () => {
    const wrapper = mount(DomainConfirm, { props: { domains } });

    await wrapper.find('input').setValue('Compilers & Linkers');
    await wrapper.findAll('button')[0].trigger('click');

    const emitted = wrapper.emitted('confirm');
    expect(emitted).toBeTruthy();
    expect((emitted?.[0][0] as Domain[])[0].title).toBe('Compilers & Linkers');
  });

  it('merges every domain into one with deduplicated concepts', async () => {
    const wrapper = mount(DomainConfirm, { props: { domains } });

    const mergeButton = wrapper.findAll('button').find((button) => button.text().includes('Merge into one domain'));
    await mergeButton?.trigger('click');

    const emitted = wrapper.emitted('confirm');
    expect(emitted?.[0][0]).toEqual([{ id: 'd1', title: 'All topics', concept_ids: ['c1', 'c2', 'c3'] }]);
  });

  it('disables confirmation while a request is in flight', () => {
    const wrapper = mount(DomainConfirm, { props: { domains, confirming: true } });

    expect(wrapper.findAll('button')[0].attributes('disabled')).toBeDefined();
  });
});
