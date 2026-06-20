import type { MistakeEntry, MistakeFilter } from '../types/mistake'
import { invoke, isTauri } from './tauri'

export async function saveMistake(entry: MistakeEntry): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('save_mistake', { entry })
  }
  const res = await fetch('/api/mistakes', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(entry),
  })
  if (!res.ok) await throwHttpError(res)
  return res.json()
}

export async function loadMistakes(filter?: MistakeFilter): Promise<MistakeEntry[]> {
  if (isTauri()) {
    return invoke<MistakeEntry[]>('load_mistakes', { filter: filter ?? null })
  }
  const query = mistakeFilterQuery(filter)
  const res = await fetch(`/api/mistakes${query}`)
  if (!res.ok) await throwHttpError(res)
  return res.json()
}

async function throwHttpError(res: Response): Promise<never> {
  const message = await res.text().catch(() => '')
  throw new Error(message ? `HTTP ${res.status}: ${message}` : `HTTP ${res.status}`)
}

function mistakeFilterQuery(filter?: MistakeFilter): string {
  if (!filter) return ''
  const params = new URLSearchParams()
  if (filter.mode) params.set('mode', filter.mode)
  if (filter.note_path) params.set('note_path', filter.note_path)
  if (filter.offset !== undefined) params.set('offset', String(filter.offset))
  if (filter.limit !== undefined) params.set('limit', String(filter.limit))
  const query = params.toString()
  return query ? `?${query}` : ''
}

export async function listPromptTemplates(): Promise<Array<{ name: string; label: string; description: string }>> {
  if (isTauri()) {
    const result = await invoke<Array<[string, string, string]>>('list_prompt_templates')
    return result.map(([name, label, description]) => ({ name, label, description }))
  }
  const res = await fetch('/api/prompt-templates')
  if (!res.ok) await throwHttpError(res)
  const result = await res.json() as Array<[string, string, string]>
  return result.map(([name, label, description]) => ({ name, label, description }))
}
