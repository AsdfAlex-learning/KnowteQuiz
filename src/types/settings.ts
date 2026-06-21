import type { QuestionType, QuizDifficulty } from './quiz'

export interface LlmConfig {
  provider: string
  base_url: string
  api_key: string
  model: string
  max_tokens: number
  temperature: number
}

export type SettingsLLM = LlmConfig

export interface UiLayout {
  left_visible: boolean
  right_visible: boolean
  left_width: number
  right_width: number
}

export interface AdvancedConfig {
  max_diagnosis_rounds: number
  require_reasoning: boolean
  show_diagnosis_report: boolean
}

export interface QuizDefaults {
  default_types: QuestionType[]
  default_language: string
  default_count: number
  default_mode: string
  default_difficulty: QuizDifficulty
  prompt_template: string
  advanced: AdvancedConfig
}

export type SettingsQuiz = QuizDefaults

export interface WorkspaceState {
  root_path?: string | null
  expanded_dirs: string[]
  selected_path?: string | null
  scroll_positions: Record<string, number>
}

export interface ConnectionTestResult {
  ok: boolean
  kind: 'ok' | 'auth' | 'model' | 'server' | 'network' | 'unknown' | string
  message: string
  status?: number | null
}

export interface DataBackupResult {
  backup_dir: string
  files: string[]
}

export interface DataRestoreResult {
  backup_dir: string
  pre_restore_backup_dir: string
  files: string[]
}

export interface DataFileStatus {
  name: string
  exists: boolean
  size_bytes: number
  modified_at?: string | null
}

export interface DataStatus {
  data_dir: string
  files: DataFileStatus[]
}

export interface Settings {
  version: string
  theme: string
  llm: LlmConfig
  ui: { layout: UiLayout }
  quiz: QuizDefaults
  workspace: WorkspaceState
}
