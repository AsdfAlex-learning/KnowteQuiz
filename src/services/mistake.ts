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
  if (filter.search_text) params.set('search_text', filter.search_text)
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

export async function markMistakeReviewed(mistakeId: string): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('mark_mistake_reviewed', { mistakeId })
  }
  const res = await fetch('/api/mistakes/review', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ mistake_id: mistakeId }),
  })
  if (!res.ok) await throwHttpError(res)
  return res.json()
}

function dateStamp(): string {
  const now = new Date()
  const year = now.getFullYear()
  const month = String(now.getMonth() + 1).padStart(2, '0')
  const day = String(now.getDate()).padStart(2, '0')
  return `${year}${month}${day}`
}

function formatMistakesAsMarkdown(mistakes: MistakeEntry[]): string {
  const date = new Date().toISOString().split('T')[0]
  const lines: string[] = [
    `# KnowteQuiz Mistake Export`,
    `Exported: ${date}`,
    `Total: ${mistakes.length} mistake${mistakes.length === 1 ? '' : 's'}`,
    '',
  ]

  for (let i = 0; i < mistakes.length; i++) {
    const m = mistakes[i]
    lines.push(`## ${i + 1}. ${m.question}`)
    lines.push('')
    lines.push(`- **Note**: \`${m.note_path}\``)
    lines.push(`- **Title**: ${m.note_title}`)
    lines.push(`- **Mode**: ${m.mode}`)
    lines.push(`- **Your Answer**: ${m.user_answer}`)
    lines.push(`- **Correct Answer**: ${m.correct_answer}`)
    lines.push(`- **Explanation**: ${m.explanation}`)
    if (m.user_reasoning) {
      lines.push(`- **Your Reasoning**: ${m.user_reasoning}`)
    }
    if (m.diagnosis && m.diagnosis.final_report) {
      lines.push(`- **Diagnosis Level**: ${m.diagnosis.final_report.overall_level}`)
    }
    lines.push(`- **Created**: ${m.created_at}`)
    lines.push(`- **Review Count**: ${m.review_count}`)
    lines.push('')
  }

  return lines.join('\n')
}

export async function exportMistakes(format: 'json' | 'markdown'): Promise<void> {
  const mistakes = await loadMistakes({})

  if (mistakes.length === 0) {
    throw new Error('No mistakes to export')
  }

  const defaultName = `knowtequiz-mistakes-${dateStamp()}`
  let content: string
  let extension: string
  let mimeType: string

  if (format === 'json') {
    content = JSON.stringify(mistakes, null, 2)
    extension = 'json'
    mimeType = 'application/json'
  } else {
    content = formatMistakesAsMarkdown(mistakes)
    extension = 'md'
    mimeType = 'text/markdown'
  }

  if (isTauri()) {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const { writeTextFile } = await import('@tauri-apps/plugin-fs')

    const filePath = await save({
      defaultPath: `${defaultName}.${extension}`,
      filters: [{
        name: format === 'json' ? 'JSON' : 'Markdown',
        extensions: [extension],
      }],
    })

    if (filePath) {
      await writeTextFile(filePath, content)
    }
  } else {
    const blob = new Blob([content], { type: mimeType })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${defaultName}.${extension}`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }
}
