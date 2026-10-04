import { afterEach, describe, expect, it, vi } from 'vitest';
import { webStream } from './tauri';

function streamFromText(text: string): ReadableStream<Uint8Array> {
  return new ReadableStream({
    start(controller) {
      controller.enqueue(new TextEncoder().encode(text));
      controller.close();
    },
  });
}

function streamFromChunks(chunks: string[]): ReadableStream<Uint8Array> {
  return new ReadableStream({
    start(controller) {
      for (const chunk of chunks) {
        controller.enqueue(new TextEncoder().encode(chunk));
      }
      controller.close();
    },
  });
}

describe('webStream', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('emits final SSE data even when the stream ends without a blank delimiter', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        body: streamFromText('data: {"event":"done","data":{"total":1}}'),
      }))
    );
    const messages: unknown[] = [];

    await webStream('/api/quiz/generate', {}, (msg) => messages.push(msg));

    expect(messages).toEqual([{ event: 'done', data: { total: 1 } }]);
  });

  it('accepts SSE data lines without a space after the colon', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        body: streamFromText('data:{"event":"done","data":{"total":1}}'),
      }))
    );
    const messages: unknown[] = [];

    await webStream('/api/quiz/generate', {}, (msg) => messages.push(msg));

    expect(messages).toEqual([{ event: 'done', data: { total: 1 } }]);
  });

  it('emits SSE events separated by CRLF delimiters', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        body: streamFromChunks([
          'data: {"event":"phase","data":{"phase":"requesting_model"}}\r\n\r\n',
          'data: {"event":"done","data":{"total":1}}\r\n\r\n',
        ]),
      }))
    );
    const messages: unknown[] = [];

    await webStream('/api/quiz/generate', {}, (msg) => messages.push(msg));

    expect(messages).toEqual([
      { event: 'phase', data: { phase: 'requesting_model' } },
      { event: 'done', data: { total: 1 } },
    ]);
  });

  it('skips malformed SSE events and keeps the stream alive', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        body: streamFromChunks(['data: {"event":"chunk"\n\n', 'data: {"event":"done","data":{"total":1}}\n\n']),
      }))
    );
    const messages: unknown[] = [];

    await webStream('/api/quiz/generate', {}, (msg) => messages.push(msg));

    expect(messages).toEqual([{ event: 'done', data: { total: 1 } }]);
  });

  it('joins multi-line data fields into a single event', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        body: streamFromText('data: {"event":"done",\ndata: "payload":{"total":1}}\n\n'),
      }))
    );
    const messages: unknown[] = [];

    await webStream('/api/quiz/generate', {}, (msg) => messages.push(msg));

    expect(messages).toEqual([{ event: 'done', payload: { total: 1 } }]);
  });

  it('passes an abort signal through to fetch and rejects when aborted', async () => {
    const controller = new AbortController();
    const fetchMock = vi.fn(async (_url: string, init: RequestInit) => {
      return new Promise((_resolve, reject) => {
        init.signal?.addEventListener('abort', () =>
          reject(new DOMException('The operation was aborted.', 'AbortError'))
        );
      });
    });
    vi.stubGlobal('fetch', fetchMock);

    const pending = webStream('/api/quiz/generate', {}, vi.fn(), controller.signal);
    controller.abort();

    await expect(pending).rejects.toThrow();
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/quiz/generate',
      expect.objectContaining({ signal: controller.signal })
    );
  });

  it('includes response text when an HTTP request fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 500,
        text: async () => 'LLM API error: model not found',
      }))
    );

    await expect(webStream('/api/quiz/generate', {}, vi.fn())).rejects.toThrow('LLM API error: model not found');
  });
});
