import type {
  QuizStreamParams,
  QuizQuestion,
  QuizEvent,
  DiagnosisEvent,
  DiagnosisReport,
  BlindSpot,
  DiagnosisRound,
} from '../types';
import { invoke, isTauri, webStream } from './tauri';
import { createLocalId } from '../utils/id';
import { throwHttpError, parseJsonResponse } from './http';

export interface StreamOptions {
  signal?: AbortSignal;
}

interface QuizEventHandlers {
  onPhase: (phase: string) => void;
  onChunk: (q: QuizQuestion) => void;
  onDone: (total: number) => void;
  onError: (msg: string) => void;
}

function dispatchQuizEvent(msg: QuizEvent, h: QuizEventHandlers): void {
  if (msg.event === 'phase') h.onPhase(msg.data.phase);
  else if (msg.event === 'chunk') h.onChunk(msg.data);
  else if (msg.event === 'done') h.onDone(msg.data.total);
  else if (msg.event === 'error') h.onError(msg.data.message);
}

interface DiagnosisEventHandlers {
  onInitial: (data: DiagnosisRound) => void;
  onFollowUp: (data: { question: string; blind_spots: BlindSpot[] }) => void;
  onReport: (data: DiagnosisReport) => void;
  onError: (msg: string) => void;
}

function dispatchDiagnosisEvent(msg: DiagnosisEvent, h: DiagnosisEventHandlers): void {
  if (msg.event === 'initial') h.onInitial(msg.data);
  else if (msg.event === 'follow_up') h.onFollowUp(msg.data);
  else if (msg.event === 'report') h.onReport(msg.data);
  else if (msg.event === 'error') h.onError(msg.data.message);
}

export async function generateQuiz(
  params: QuizStreamParams,
  onPhase: (phase: string) => void,
  onChunk: (q: QuizQuestion) => void,
  onDone: (total: number) => void,
  onError: (msg: string) => void,
  options: StreamOptions = {}
): Promise<void> {
  const handlers: QuizEventHandlers = { onPhase, onChunk, onDone, onError };
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core');
    const channel = new Channel<QuizEvent>();
    channel.onmessage = (msg) => dispatchQuizEvent(msg, handlers);
    await invoke('generate_quiz', { params, onEvent: channel });
  } else {
    await webStream<QuizEvent>('/api/quiz/generate', params, (msg) => dispatchQuizEvent(msg, handlers), options.signal);
  }
}

export async function submitAnswerAdvanced(
  question: string,
  correct_answer: string,
  user_answer: string,
  user_reasoning: string,
  note_path: string,
  onInitial: (data: DiagnosisRound) => void,
  onFollowUp: (data: { question: string; blind_spots: BlindSpot[] }) => void,
  onReport: (data: DiagnosisReport) => void,
  onError: (msg: string) => void,
  options: StreamOptions = {}
): Promise<string> {
  const handlers: DiagnosisEventHandlers = { onInitial, onFollowUp, onReport, onError };
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core');
    const channel = new Channel<DiagnosisEvent>();
    channel.onmessage = (msg) => dispatchDiagnosisEvent(msg, handlers);
    return invoke<string>('submit_answer_advanced', {
      question,
      correctAnswer: correct_answer,
      userAnswer: user_answer,
      userReasoning: user_reasoning,
      notePath: note_path,
      onEvent: channel,
    });
  }
  const sessionId = createLocalId('web');
  await webStream<DiagnosisEvent>(
    '/api/quiz/diagnose',
    {
      session_id: sessionId,
      question,
      correct_answer,
      user_answer,
      user_reasoning,
      note_path,
    },
    (msg) => dispatchDiagnosisEvent(msg, handlers),
    options.signal
  );
  return sessionId;
}

export async function diagnoseFollowUp(
  sessionId: string,
  userReply: string,
  onFollowUp: (data: { question: string; blind_spots: BlindSpot[] }) => void,
  onReport: (data: DiagnosisReport) => void,
  onError: (msg: string) => void,
  options: StreamOptions = {}
): Promise<void> {
  const handlers: DiagnosisEventHandlers = { onInitial: () => {}, onFollowUp, onReport, onError };
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core');
    const channel = new Channel<DiagnosisEvent>();
    channel.onmessage = (msg) => dispatchDiagnosisEvent(msg, handlers);
    await invoke('diagnose_follow_up', { sessionId, userReply, onEvent: channel });
  } else {
    await webStream<DiagnosisEvent>(
      `/api/quiz/diagnose/${sessionId}/follow_up`,
      {
        user_reply: userReply,
      },
      (msg) => dispatchDiagnosisEvent(msg, handlers),
      options.signal
    );
  }
}

export async function generateDiagnosisReport(sessionId: string): Promise<DiagnosisReport> {
  if (isTauri()) {
    return invoke<DiagnosisReport>('generate_diagnosis_report', { sessionId });
  }
  const res = await fetch(`/api/quiz/diagnose/${sessionId}/report`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<DiagnosisReport>(res);
}

export interface SessionCleanupResult {
  deleted_count: number;
  remaining_count: number;
}

export async function cleanupSessions(): Promise<SessionCleanupResult> {
  if (isTauri()) {
    return invoke<SessionCleanupResult>('cleanup_sessions');
  }
  const res = await fetch('/api/sessions/cleanup', { method: 'POST' });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<SessionCleanupResult>(res);
}
