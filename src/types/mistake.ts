import type { DiagnosisRound, DiagnosisReport } from './diagnosis'

export type MistakeMode = 'basic' | 'advanced'

export interface DiagnosisContext {
  rounds: number
  conversation: DiagnosisRound[]
  final_report: DiagnosisReport
}

export interface MistakeEntry {
  id: string
  note_path: string
  note_title: string
  question: string
  user_answer: string
  correct_answer: string
  explanation: string
  mode: MistakeMode
  user_reasoning?: string
  diagnosis?: DiagnosisContext
  created_at: string
  review_count: number
  last_reviewed_at?: string
}

export interface MistakeFilter {
  mode?: MistakeMode
  note_path?: string
  search_text?: string
  offset?: number
  limit?: number
}
