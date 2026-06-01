export interface BlindSpot {
  tag: string
  severity: string
  description: string
  note_reference: string
  suggestion: string
}

export interface DiagnosisRound {
  role: 'ai' | 'user'
  content: string
  blind_spots: BlindSpot[]
  follow_up?: string
}

export interface DiagnosisReport {
  summary: string
  blind_spots: BlindSpot[]
  overall_level: string
  next_steps: string[]
}

export interface DiagnosisSession {
  session_id: string
  question: string
  user_answer: string
  user_reasoning: string
  note_path: string
  conversation: DiagnosisRound[]
  current_round: number
  max_rounds: number
  final_report?: DiagnosisReport
}

export type DiagnosisEvent =
  | { event: 'initial'; data: DiagnosisRound }
  | { event: 'follow_up'; data: { question: string; blind_spots: BlindSpot[] } }
  | { event: 'report'; data: DiagnosisReport }
  | { event: 'error'; data: { message: string } }
