export { convertFileSrc, invoke } from '@tauri-apps/api/core';

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

export function isTauri(): boolean {
  return typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;
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
  // SSE spec: multiple `data:` lines in one event join with \n; `:` lines are
  // comments; `event:`/`id:`/`retry:` fields are not used by this backend.
  const dataLines: string[] = [];
  for (const line of chunk.split(/\r?\n/)) {
    if (line.startsWith(':')) continue;
    if (line.startsWith('data:')) {
      dataLines.push(line.slice(5).replace(/^ /, ''));
    }
  }
  if (dataLines.length === 0) return;

  try {
    const msg = JSON.parse(dataLines.join('\n')) as T;
    onMessage(msg);
  } catch {
    // A malformed event must not kill the whole stream: skip it and keep
    // reading. Malformed events are logged on the backend debug log instead.
  }
}
