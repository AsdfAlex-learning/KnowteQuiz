export { invoke } from '@tauri-apps/api/core'

export function isTauri(): boolean {
  return typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined
}

export async function webStream<T>(
  path: string,
  body: Record<string, unknown>,
  onMessage: (msg: T) => void,
): Promise<void> {
  const response = await fetch(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })

  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`)
  }

  const reader = response.body?.getReader()
  if (!reader) throw new Error('No response body')

  const decoder = new TextDecoder()
  let buffer = ''

  while (true) {
    const { done, value } = await reader.read()
    if (done) break

    buffer += decoder.decode(value, { stream: true })
    const chunks = buffer.split('\n\n')
    buffer = chunks.pop() || ''

    for (const chunk of chunks) {
      const lines = chunk.split('\n')
      let data = ''
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          data = line.slice(6)
          break
        }
      }
      if (data) {
        try {
          const msg = JSON.parse(data) as T
          onMessage(msg)
        } catch (e) {
          console.error('SSE parse error:', e)
        }
      }
    }
  }
}
