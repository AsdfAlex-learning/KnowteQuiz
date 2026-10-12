import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import LlmRunsPanel from './LlmRunsPanel.vue';
import * as llmRuns from '@/services/llmRuns';
import type { LlmRunSummary } from '@/types/llmRun';

vi.mock('@/services/llmRuns', () => ({
  listLlmRuns: vi.fn(),
  getLlmRun: vi.fn(),
  getRunContent: vi.fn(),
}));

function summary(overrides: Partial<LlmRunSummary> = {}): LlmRunSummary {
  return {
    id: 'run-1',
    kind: 'quiz',
    ts_start: '2026-10-12T12:00:00.000Z',
    ts_end: '2026-10-12T12:00:01.000Z',
    duration_ms: 1250,
    model: 'test-model',
    usage: null,
    usage_source: 'unavailable',
    http_status: 200,
    ...overrides,
  };
}

describe('LlmRunsPanel', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.mocked(llmRuns.listLlmRuns).mockResolvedValue([]);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  function mountPanel() {
    return mount(LlmRunsPanel, { global: { plugins: [createPinia()] } });
  }

  it('shows the empty state when no runs are recorded', async () => {
    const wrapper = mountPanel();
    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('No LLM runs recorded yet');
    });
  });

  it('lists runs with kind badge and formatted duration', async () => {
    vi.mocked(llmRuns.listLlmRuns).mockResolvedValue([
      summary(),
      summary({ id: 'run-2', kind: 'diagnosis_report', duration_ms: 2500, usage: { total_tokens: 320 } }),
    ]);

    const wrapper = mountPanel();
    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Quiz');
    });

    expect(wrapper.text()).toContain('Report');
    expect(wrapper.text()).toContain('1.3s');
    expect(wrapper.text()).toContain('2.5s');
    // usage present on run-2, absent on run-1
    expect(wrapper.text()).toContain('320t');
  });

  it('expands a row and loads the prompt and raw response', async () => {
    vi.mocked(llmRuns.listLlmRuns).mockResolvedValue([summary()]);
    vi.mocked(llmRuns.getLlmRun).mockResolvedValue({
      ...summary(),
      endpoint_host: 'localhost:11434',
      request_params: { temperature: 0.3 },
    });
    vi.mocked(llmRuns.getRunContent).mockImplementation(async (_id, field) =>
      field === 'prompt' ? 'PROMPT-TEXT' : 'RAW-RESPONSE'
    );

    const wrapper = mountPanel();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Quiz'));

    await wrapper.get('.border-b.cursor-pointer').trigger('click');

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('PROMPT-TEXT');
    });

    expect(llmRuns.getLlmRun).toHaveBeenCalledWith('run-1');
    expect(llmRuns.getRunContent).toHaveBeenCalledWith('run-1', 'prompt');
    expect(llmRuns.getRunContent).toHaveBeenCalledWith('run-1', 'response');
    expect(wrapper.text()).toContain('RAW-RESPONSE');
    expect(wrapper.text()).toContain('localhost:11434');
  });

  it('shows a not-recorded placeholder when the content was not stored', async () => {
    vi.mocked(llmRuns.listLlmRuns).mockResolvedValue([summary()]);
    vi.mocked(llmRuns.getLlmRun).mockResolvedValue({
      ...summary(),
      endpoint_host: 'h',
      request_params: null,
      prompt_file: null,
      response_file: null,
    });
    vi.mocked(llmRuns.getRunContent).mockResolvedValue('');

    const wrapper = mountPanel();
    await vi.waitFor(() => expect(wrapper.text()).toContain('Quiz'));
    await wrapper.get('.border-b.cursor-pointer').trigger('click');

    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('Not recorded');
    });
  });

  it('surfaces a load failure in the error banner', async () => {
    vi.mocked(llmRuns.listLlmRuns).mockRejectedValue(new Error('boom'));

    const wrapper = mountPanel();
    await vi.waitFor(() => {
      expect(wrapper.text()).toContain('boom');
    });
  });
});
