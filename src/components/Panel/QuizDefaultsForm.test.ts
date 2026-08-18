// @vitest-environment jsdom

import { mount } from '@vue/test-utils';
import { describe, expect, it, vi } from 'vitest';
import QuizDefaultsForm from './QuizDefaultsForm.vue';
import { defaultSettings } from '@/utils/defaults';

vi.mock('@/composables/useI18n', () => ({
  useI18n: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        'settings_page.quiz_defaults': 'Quiz Defaults',
        'settings_page.language_hint': 'Language',
        'settings_page.question_types': 'Question Types',
        'settings_page.single_choice': 'Single Choice',
        'settings_page.multiple_choice': 'Multiple Choice',
        'settings_page.short_answer': 'Short Answer',
        'settings_page.default_mode': 'Default Mode',
        'settings_page.basic': 'Basic',
        'settings_page.advanced': 'Advanced',
        'settings_page.question_count': 'Count',
        'settings_page.difficulty': 'Difficulty',
        'settings_page.easy': 'Easy',
        'settings_page.medium': 'Medium',
        'settings_page.hard': 'Hard',
      };
      return translations[key] || key;
    },
    locale: { value: 'en' },
    availableLocales: [],
    setLocale: () => {},
  }),
}));

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
});
