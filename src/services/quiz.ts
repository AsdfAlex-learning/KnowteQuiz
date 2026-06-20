import type { QuizStreamParams, QuizQuestion, QuizEvent, DiagnosisEvent, DiagnosisReport, BlindSpot, DiagnosisRound } from '../types'
import { invoke, isTauri, webStream } from './tauri'

export async function generateQuiz(
  params: QuizStreamParams,
  onChunk: (q: QuizQuestion) => void,
  onDone: (total: number) => void,
  onError: (msg: string) => void,
): Promise<void> {
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core')
    const channel = new Channel<QuizEvent>()
    channel.onmessage = (msg) => {
      if (msg.event === 'chunk') onChunk(msg.data)
      else if (msg.event === 'done') onDone(msg.data.total)
      else if (msg.event === 'error') onError(msg.data.message)
    }
    await invoke('generate_quiz', { params, onEvent: channel })
  } else {
    await webStream<QuizEvent>('/api/quiz/generate', params, (msg) => {
      if (msg.event === 'chunk') onChunk(msg.data)
      else if (msg.event === 'done') onDone(msg.data.total)
      else if (msg.event === 'error') onError(msg.data.message)
    })
  }
}

export async function submitAnswerAdvanced(
  question: string,
  user_answer: string,
  user_reasoning: string,
  note_path: string,
  onInitial: (data: DiagnosisRound) => void,
  onFollowUp: (data: { question: string; blind_spots: BlindSpot[] }) => void,
  onReport: (data: DiagnosisReport) => void,
  onError: (msg: string) => void,
): Promise<string> {
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core')
    const channel = new Channel<DiagnosisEvent>()
    channel.onmessage = (msg) => {
      if (msg.event === 'initial') onInitial(msg.data)
      else if (msg.event === 'follow_up') onFollowUp(msg.data)
      else if (msg.event === 'report') onReport(msg.data)
      else if (msg.event === 'error') onError(msg.data.message)
    }
    return invoke<string>('submit_answer_advanced', {
      question, userAnswer: user_answer, userReasoning: user_reasoning, notePath: note_path, onEvent: channel,
    })
  } else {
    const sessionId = crypto.randomUUID ? crypto.randomUUID() : `web-${Date.now()}-${Math.random().toString(36).slice(2)}`
    await webStream<DiagnosisEvent>('/api/quiz/diagnose', {
      session_id: sessionId,
      question,
      user_answer,
      user_reasoning,
      note_path,
    }, (msg) => {
      if (msg.event === 'initial') onInitial(msg.data)
      else if (msg.event === 'follow_up') onFollowUp(msg.data)
      else if (msg.event === 'report') onReport(msg.data)
      else if (msg.event === 'error') onError(msg.data.message)
    })
    return sessionId
  }
}

export async function diagnoseFollowUp(
  sessionId: string,
  userReply: string,
  onFollowUp: (data: { question: string; blind_spots: BlindSpot[] }) => void,
  onReport: (data: DiagnosisReport) => void,
  onError: (msg: string) => void,
): Promise<void> {
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core')
    const channel = new Channel<DiagnosisEvent>()
    channel.onmessage = (msg) => {
      if (msg.event === 'follow_up') onFollowUp(msg.data)
      else if (msg.event === 'report') onReport(msg.data)
      else if (msg.event === 'error') onError(msg.data.message)
    }
    await invoke('diagnose_follow_up', { sessionId, userReply, onEvent: channel })
  } else {
    await webStream<DiagnosisEvent>(`/api/quiz/diagnose/${sessionId}/follow_up`, {
      user_reply: userReply,
    }, (msg) => {
      if (msg.event === 'follow_up') onFollowUp(msg.data)
      else if (msg.event === 'report') onReport(msg.data)
      else if (msg.event === 'error') onError(msg.data.message)
    })
  }
}

export async function generateDiagnosisReport(sessionId: string): Promise<DiagnosisReport> {
  if (isTauri()) {
    return invoke<DiagnosisReport>('generate_diagnosis_report', { sessionId })
  }
  const res = await fetch(`/api/quiz/diagnose/${sessionId}/report`)
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json()
}
