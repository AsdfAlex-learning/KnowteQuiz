import { afterEach, describe, expect, it, vi } from 'vitest';
import { getLlmRun, getRunContent, listLlmRuns } from './llmRuns';

vi.mock('./tauri', () => ({
  isTauri: () => false,
  invoke: vi.fn(),
  webStream: vi.fn(),
}));

const summary = {
  id: 'run-1',
  kind: 'quiz' as const,
  ts_start: '2026-10-12T12:00:00.000Z',
  ts_end: '2026-10-12T12:00:01.000Z',
  duration_ms: 1000,
  model: 'm',
  usage: null,
  usage_source: 'unavailable' as const,
  http_status: 200,
};

describe('llmRuns service', () => {
  afterEach(() => {
    vi.clearAllMocks();
    vi.unstubAllGlobals();
  });

  it('lists runs with query params from the web endpoint', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        text: async () => JSON.stringify([summary]),
        json: async () => [summary],
      })) as unknown as typeof fetch
    );

    const runs = await listLlmRuns({ limit: 5, kind: 'quiz' });

    expect(fetch).toHaveBeenCalledWith('/api/llm/runs?limit=5&kind=quiz');
    expect(runs).toHaveLength(1);
    expect(runs[0].id).toBe('run-1');
  });

  it('requests the detail endpoint for a single run', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        text: async () => JSON.stringify({ ...summary, endpoint_host: 'localhost:11434' }),
        json: async () => ({ ...summary, endpoint_host: 'localhost:11434' }),
      })) as unknown as typeof fetch
    );

    const run = await getLlmRun('run-1');

    expect(fetch).toHaveBeenCalledWith('/api/llm/runs/run-1');
    expect(run.endpoint_host).toBe('localhost:11434');
  });

  it('fetches run content as plain text', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        text: async () => 'PROMPT-TEXT',
      })) as unknown as typeof fetch
    );

    const content = await getRunContent('run-1', 'prompt');

    expect(fetch).toHaveBeenCalledWith('/api/llm/runs/run-1/prompt');
    expect(content).toBe('PROMPT-TEXT');
  });

  it('encodes the run id in the request URL', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        text: async () => JSON.stringify({ ...summary, endpoint_host: 'h' }),
        json: async () => ({ ...summary, endpoint_host: 'h' }),
      })) as unknown as typeof fetch
    );

    await getLlmRun('a/b');

    expect(fetch).toHaveBeenCalledWith('/api/llm/runs/a%2Fb');
  });

  it('throws the HTTP error body when the endpoint fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 404,
        text: async () => 'LLM run missing not found',
      })) as unknown as typeof fetch
    );

    await expect(getLlmRun('missing')).rejects.toThrow('HTTP 404: LLM run missing not found');
  });
});
