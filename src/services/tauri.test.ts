import { afterEach, describe, expect, it, vi } from 'vitest'
import { webStream } from './tauri'

function streamFromText(text: string): ReadableStream<Uint8Array> {
  return new ReadableStream({
    start(controller) {
      controller.enqueue(new TextEncoder().encode(text))
      controller.close()
    },
  })
}

describe('webStream', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('emits final SSE data even when the stream ends without a blank delimiter', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      body: streamFromText('data: {"event":"done","data":{"total":1}}'),
    })))
    const messages: unknown[] = []

    await webStream('/api/quiz/generate', {}, (msg) => messages.push(msg))

    expect(messages).toEqual([{ event: 'done', data: { total: 1 } }])
  })

  it('accepts SSE data lines without a space after the colon', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      body: streamFromText('data:{"event":"done","data":{"total":1}}'),
    })))
    const messages: unknown[] = []

    await webStream('/api/quiz/generate', {}, (msg) => messages.push(msg))

    expect(messages).toEqual([{ event: 'done', data: { total: 1 } }])
  })

  it('rejects malformed SSE data instead of silently ignoring it', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: true,
      body: streamFromText('data: {"event":"done"'),
    })))

    await expect(webStream('/api/quiz/generate', {}, vi.fn()))
      .rejects.toThrow('SSE parse error')
  })

  it('includes response text when an HTTP request fails', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ({
      ok: false,
      status: 500,
      text: async () => 'LLM API error: model not found',
    })))

    await expect(webStream('/api/quiz/generate', {}, vi.fn()))
      .rejects.toThrow('LLM API error: model not found')
  })
})
