export type QuestionType = 'single' | 'multiple' | 'short'
export type QuizDifficulty = 'easy' | 'medium' | 'hard'
export type QuizLanguage = 'zh' | 'en' | 'ja' | 'ko'

export interface QuizQuestion {
  id: string
  question_type: QuestionType
  question: string
  options: string[]
  answer: string
  explanation: string
}

export interface QuizStreamParams {
  path: string
  types: QuestionType[]
  count: number
  difficulty: QuizDifficulty
  lang: QuizLanguage
}

export type QuizEvent =
  | { event: 'chunk'; data: QuizQuestion }
  | { event: 'done'; data: { total: number } }
  | { event: 'error'; data: { message: string } }
