import type { MistakeEntry } from '../types/mistake'
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
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json()
}

export async function loadMistakes(): Promise<MistakeEntry[]> {
  if (isTauri()) {
    return invoke<MistakeEntry[]>('load_mistakes')
  }
  const res = await fetch('/api/mistakes')
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json()
}

export async function listPromptTemplates(): Promise<Array<{ name: string; label: string; description: string }>> {
  if (isTauri()) {
    const result = await invoke<Array<[string, string, string]>>('list_prompt_templates')
    return result.map(([name, label, description]) => ({ name, label, description }))
  }
  const res = await fetch('/api/prompt-templates')
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  const result = await res.json() as Array<[string, string, string]>
  return result.map(([name, label, description]) => ({ name, label, description }))
}
