import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import AppearanceSettings from './AppearanceSettings.vue';
import { defaultSettings } from '@/utils/defaults';
import type { ThemeConfig } from '@/types/settings';

vi.mock('@/services/settings', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/services/settings')>();
  return { ...actual, saveMediaFile: vi.fn() };
});

function theme(): ThemeConfig {
  return defaultSettings().theme_config;
}

describe('AppearanceSettings', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('applies a VS Code preset palette', async () => {
    const wrapper = mount(AppearanceSettings, { props: { modelValue: theme() } });

    const monokai = wrapper.findAll('button').find((button) => button.text().includes('Monokai'));
    await monokai?.trigger('click');

    const value = wrapper.emitted('update:modelValue')?.[0][0] as ThemeConfig;
    expect(value.preset).toBe('vscode-monokai');
    expect(value.background_color).toBe('#272822');
    expect(value.accent_color).toBe('#a6e22e');
  });

  it('marks the theme custom when a colour is edited', async () => {
    const wrapper = mount(AppearanceSettings, { props: { modelValue: theme() } });

    await wrapper.find('#bg-color-input').setValue('#123456');

    const value = wrapper.emitted('update:modelValue')?.[0][0] as ThemeConfig;
    expect(value.preset).toBe('custom');
    expect(value.background_color).toBe('#123456');
  });

  it('resets to VS Code Dark+', async () => {
    const wrapper = mount(AppearanceSettings, { props: { modelValue: theme() } });

    const reset = wrapper.findAll('button').find((button) => button.text().includes('Reset'));
    await reset?.trigger('click');

    const value = wrapper.emitted('update:modelValue')?.[0][0] as ThemeConfig;
    expect(value.preset).toBe('vscode-dark-plus');
    expect(value.background_color).toBe('#1e1e1e');
    expect(value.accent_color).toBe('#007acc');
  });
});
