// @vitest-environment jsdom

import { mount } from '@vue/test-utils';
import { nextTick } from 'vue';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, describe, expect, it, vi } from 'vitest';
import QuizGenerator from './QuizGenerator.vue';
import { useExplorerStore } from '@/stores/explorer';
import { useQuizStore } from '@/stores/quiz';
import { useMistakeStore } from '@/stores/mistakes';
import { useSettingsStore } from '@/stores/settings';
import { generateQuiz } from '@/services/quiz';

vi.mock('@/services/quiz', () => ({
  generateQuiz: vi.fn(),
  submitAnswerAdvanced: vi.fn(),
  diagnoseFollowUp: vi.fn(),
  generateDiagnosisReport: vi.fn(),
}));

describe('QuizGenerator', () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  function mockGeneratedQuiz() {
    vi.mocked(generateQuiz).mockImplementation(async (_params, _onPhase, onChunk, onDone) => {
      onChunk({
        id: 'q1',
        question_type: 'single',
        question: 'Which claim is true?',
        options: ['A. Alpha', 'B. Beta'],
        answer: 'A',
        explanation: 'Alpha is true.',
      });
      onDone(1);
    });
  }

  it('starts an advanced quiz when advanced mode is selected', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const explorerStore = useExplorerStore();
    explorerStore.selectedPath = '/notes/rust.md';
    mockGeneratedQuiz();

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    });
    await nextTick();

    await wrapper
      .findAll('button')
      .find((button) => button.text() === 'Advanced')
      ?.trigger('click');
    await wrapper
      .findAll('button')
      .find((button) => button.text() === 'Start Quiz')
      ?.trigger('click');

    expect(useQuizStore().mode).toBe('advanced');
  });

  it('clears stale mistake save state before starting a new quiz', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const explorerStore = useExplorerStore();
    explorerStore.selectedPath = '/notes/rust.md';
    const mistakeStore = useMistakeStore();
    mistakeStore.savedIds.add('q1');
    mistakeStore.errors.set('q1', 'old error');
    mockGeneratedQuiz();

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    });
    await nextTick();

    const startButton = wrapper.findAll('button').find((button) => button.text() === 'Start Quiz');
    expect(startButton?.exists()).toBe(true);
    expect(startButton?.attributes('disabled')).toBeUndefined();

    await startButton?.trigger('click');

    expect(mistakeStore.isSaved('q1')).toBe(false);
    expect(mistakeStore.errorFor('q1')).toBeNull();
  });

  it('uses quiz defaults that load after the generator is mounted', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const explorerStore = useExplorerStore();
    explorerStore.selectedPath = '/notes/rust.md';
    const settingsStore = useSettingsStore();
    mockGeneratedQuiz();

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    });
    await nextTick();

    settingsStore.settings.quiz = {
      ...settingsStore.settings.quiz,
      default_mode: 'basic',
      default_types: ['multiple'],
      default_count: 7,
      default_language: 'en',
      default_difficulty: 'hard',
    };
    await nextTick();

    await wrapper
      .findAll('button')
      .find((button) => button.text() === 'Start Quiz')
      ?.trigger('click');

    expect(generateQuiz).toHaveBeenCalledWith(
      expect.objectContaining({
        types: ['multiple'],
        count: 7,
        difficulty: 'hard',
        lang: 'en',
      }),
      expect.any(Function),
      expect.any(Function),
      expect.any(Function),
      expect.any(Function)
    );
  });

  it('falls back to supported quiz defaults when saved defaults are invalid', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const explorerStore = useExplorerStore();
    explorerStore.selectedPath = '/notes/rust.md';
    const settingsStore = useSettingsStore();
    mockGeneratedQuiz();

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    });
    await nextTick();

    settingsStore.settings.quiz = {
      ...settingsStore.settings.quiz,
      default_types: [],
      default_language: 'fr',
      default_difficulty: 'wild' as never,
    };
    await nextTick();

    await wrapper
      .findAll('button')
      .find((button) => button.text() === 'Start Quiz')
      ?.trigger('click');

    expect(generateQuiz).toHaveBeenCalledWith(
      expect.objectContaining({
        types: ['single', 'short'],
        difficulty: 'medium',
        lang: 'zh',
      }),
      expect.any(Function),
      expect.any(Function),
      expect.any(Function),
      expect.any(Function)
    );
  });

  it('falls back to a supported quiz count when the saved default count is out of range', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const explorerStore = useExplorerStore();
    explorerStore.selectedPath = '/notes/rust.md';
    const settingsStore = useSettingsStore();
    mockGeneratedQuiz();

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    });
    await nextTick();

    settingsStore.settings.quiz = {
      ...settingsStore.settings.quiz,
      default_count: 0,
    };
    await nextTick();

    await wrapper
      .findAll('button')
      .find((button) => button.text() === 'Start Quiz')
      ?.trigger('click');

    expect(generateQuiz).toHaveBeenCalledWith(
      expect.objectContaining({
        count: 5,
      }),
      expect.any(Function),
      expect.any(Function),
      expect.any(Function),
      expect.any(Function)
    );
  });

  it('uses the loaded default quiz mode when starting a quiz', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const explorerStore = useExplorerStore();
    explorerStore.selectedPath = '/notes/rust.md';
    const settingsStore = useSettingsStore();
    mockGeneratedQuiz();

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    });
    await nextTick();

    settingsStore.settings.quiz = {
      ...settingsStore.settings.quiz,
      default_mode: 'advanced',
    };
    await nextTick();

    await wrapper
      .findAll('button')
      .find((button) => button.text() === 'Start Quiz')
      ?.trigger('click');

    expect(useQuizStore().mode).toBe('advanced');
  });

  it('does not mutate saved quiz default types when toggling generator types', async () => {
    const pinia = createPinia();
    setActivePinia(pinia);
    const settingsStore = useSettingsStore();
    settingsStore.settings.quiz.default_types = ['single', 'short'];

    const wrapper = mount(QuizGenerator, {
      global: {
        plugins: [pinia],
      },
    });
    await nextTick();

    await wrapper
      .findAll('button')
      .find((button) => button.text() === 'Multiple Choice')
      ?.trigger('click');

    expect(settingsStore.settings.quiz.default_types).toEqual(['single', 'short']);
  });
});
