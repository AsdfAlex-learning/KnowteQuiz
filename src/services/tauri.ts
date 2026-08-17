export { convertFileSrc, invoke } from '@tauri-apps/api/core';

export function isTauri(): boolean {
  return typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined;
}

export async function webStream<T>(path: string, body: unknown, onMessage: (msg: T) => void): Promise<void> {
  const response = await fetch(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const message = await response.text().catch(() => '');
    throw new Error(message ? `HTTP ${response.status}: ${message}` : `HTTP ${response.status}`);
  }

  const reader = response.body?.getReader();
  if (!reader) throw new Error('No response body');

  const decoder = new TextDecoder();
  let buffer = '';

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    buffer += decoder.decode(value, { stream: true });
    const chunks = buffer.split(/\r?\n\r?\n/);
    buffer = chunks.pop() || '';

    for (const chunk of chunks) {
      emitSseChunk(chunk, onMessage);
    }
  }

  if (buffer.trim()) {
    emitSseChunk(buffer, onMessage);
  }
}

function emitSseChunk<T>(chunk: string, onMessage: (msg: T) => void): void {
  const lines = chunk.split(/\r?\n/);
  let data = '';
  for (const line of lines) {
    if (line.startsWith('data:')) {
      data = line.slice(5).replace(/^ /, '');
      break;
    }
  }
  if (data) {
    try {
      const msg = JSON.parse(data) as T;
      onMessage(msg);
    } catch (e) {
      throw new Error(`SSE parse error: ${String(e)}`);
    }
  }
}
