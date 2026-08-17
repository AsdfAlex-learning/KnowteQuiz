// @vitest-environment jsdom

import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import QuizDefaultsForm from './QuizDefaultsForm.vue';
import { defaultSettings } from '@/utils/defaults';

vi.mock('@/services/mistake', () => ({
  listPromptTemplates: vi.fn(async () => []),
}));

describe('QuizDefaultsForm', () => {
  it('emits default mode changes', async () => {
    const quiz = defaultSettings().quiz;
    const wrapper = mount(QuizDefaultsForm, {
      props: {
        modelValue: quiz,
      },
    });

    const modeSelect = wrapper.find('select[aria-label="Default quiz mode"]');
    expect(modeSelect.exists()).toBe(true);

    await modeSelect.setValue('advanced');

    expect(wrapper.emitted('update:modelValue')?.[0]?.[0]).toEqual({
      ...quiz,
      default_mode: 'advanced',
    });
  });

  it('offers all supported quiz languages', () => {
    const wrapper = mount(QuizDefaultsForm, {
      props: {
        modelValue: defaultSettings().quiz,
      },
    });

    const languageOptions = wrapper
      .find('select[aria-label="Default quiz language"]')
      .findAll('option')
      .map((option) => option.attributes('value'));

    expect(languageOptions).toEqual(['zh', 'en', 'ja', 'ko']);
  });
});
