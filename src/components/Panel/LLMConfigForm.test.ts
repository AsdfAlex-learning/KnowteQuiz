import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import LLMConfigForm from './LLMConfigForm.vue';
import { defaultSettings } from '@/utils/defaults';
import { listLlmModels } from '@/services/settings';
import type { SettingsLLM } from '@/types/settings';

vi.mock('@/services/settings', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/services/settings')>();
  return { ...actual, listLlmModels: vi.fn() };
});

function llm(overrides: Partial<SettingsLLM> = {}): SettingsLLM {
  return { ...defaultSettings().llm, ...overrides };
}

describe('LLMConfigForm', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('fills the URL and default model when a provider preset is picked', async () => {
    const wrapper = mount(LLMConfigForm, { props: { llm: llm() } });

    await wrapper.find('select').setValue('deepseek');

    const emitted = wrapper.emitted('update:llm');
    expect(emitted).toBeTruthy();
    const value = emitted?.[0][0] as SettingsLLM;
    expect(value.provider).toBe('deepseek');
    expect(value.base_url).toBe('https://api.deepseek.com/v1');
    expect(value.model).toBe('deepseek-chat');
  });

  it('returns to custom without touching the URL', async () => {
    const wrapper = mount(LLMConfigForm, {
      props: { llm: llm({ provider: 'deepseek', base_url: 'https://api.deepseek.com/v1' }) },
    });

    await wrapper.find('select').setValue('custom');

    const value = wrapper.emitted('update:llm')?.[0][0] as SettingsLLM;
    expect(value.provider).toBe('custom');
    expect(value.base_url).toBe('https://api.deepseek.com/v1');
  });

  it('fetches models and exposes them as datalist options', async () => {
    vi.mocked(listLlmModels).mockResolvedValue(['gpt-4o', 'gpt-4o-mini']);
    const wrapper = mount(LLMConfigForm, {
      props: { llm: llm({ base_url: 'https://api.openai.com/v1', model: 'gpt-4o' }) },
    });

    const fetchButton = wrapper.findAll('button').find((button) => button.text().includes('Fetch models'));
    await fetchButton?.trigger('click');
    await wrapper.vm.$nextTick();

    expect(listLlmModels).toHaveBeenCalled();
    expect(wrapper.findAll('datalist option')).toHaveLength(2);
    expect(wrapper.text()).toContain('2 models found');
  });

  it('reports an empty model list', async () => {
    vi.mocked(listLlmModels).mockResolvedValue([]);
    const wrapper = mount(LLMConfigForm, {
      props: { llm: llm({ base_url: 'https://api.openai.com/v1' }) },
    });

    const fetchButton = wrapper.findAll('button').find((button) => button.text().includes('Fetch models'));
    await fetchButton?.trigger('click');
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain('The provider returned no models');
  });
});
