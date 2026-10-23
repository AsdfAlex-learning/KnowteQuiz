import type {
  ConfirmExplorationDomainsRequest,
  CreateExplorationSessionRequest,
  Domain,
  ExplorationExportResult,
  ExplorationSession,
  ExplorationSessionSummary,
  ExplorationStreamEvent,
  ImportExplorationRequest,
  KnowledgeGraph,
} from '../types/exploration';
import { invoke, isTauri, webStream } from './tauri';
import { parseJsonResponse, throwHttpError } from './http';

export interface StreamOptions {
  signal?: AbortSignal;
}

export interface ExplorationStreamHandlers {
  onPhase?: (phase: string) => void;
  onDomainWarning?: (domains: Domain[], warning: string) => void;
  onGraphReady?: (graph: KnowledgeGraph) => void;
  onError?: (message: string) => void;
  onDone?: (sessionId: string) => void;
}

function dispatchExplorationEvent(msg: ExplorationStreamEvent, handlers: ExplorationStreamHandlers): void {
  switch (msg.event) {
    case 'phase':
      handlers.onPhase?.(msg.data.phase);
      break;
    case 'domain_warning':
      handlers.onDomainWarning?.(msg.data.domains, msg.data.warning);
      break;
    case 'graph_ready':
      handlers.onGraphReady?.(msg.data.graph);
      break;
    case 'error':
      handlers.onError?.(msg.data.message);
      break;
    case 'done':
      handlers.onDone?.(msg.data.session_id);
      break;
  }
}

export async function createExplorationSession(request: CreateExplorationSessionRequest): Promise<ExplorationSession> {
  if (isTauri()) {
    return invoke<ExplorationSession>('exploration_create', { request });
  }
  const res = await fetch('/api/exploration/sessions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request),
  });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSession>(res);
}

export async function listExplorationSessions(): Promise<ExplorationSessionSummary[]> {
  if (isTauri()) {
    return invoke<ExplorationSessionSummary[]>('exploration_list');
  }
  const res = await fetch('/api/exploration/sessions');
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSessionSummary[]>(res);
}

export async function getExplorationSession(sessionId: string): Promise<ExplorationSession> {
  if (isTauri()) {
    return invoke<ExplorationSession>('exploration_get', { sessionId });
  }
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSession>(res);
}

export async function deleteExplorationSession(sessionId: string): Promise<boolean> {
  if (isTauri()) {
    return invoke<boolean>('exploration_delete', { sessionId });
  }
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}`, {
    method: 'DELETE',
  });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<boolean>(res);
}

export async function startExplorationStream(
  sessionId: string,
  handlers: ExplorationStreamHandlers,
  options: StreamOptions = {}
): Promise<void> {
  const path = `/api/exploration/sessions/${encodeURIComponent(sessionId)}/start`;
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core');
    const channel = new Channel<ExplorationStreamEvent>();
    channel.onmessage = (msg) => dispatchExplorationEvent(msg, handlers);
    await invoke('exploration_start_stream', { sessionId, onEvent: channel });
    return;
  }
  await webStream<ExplorationStreamEvent>(path, {}, (msg) => dispatchExplorationEvent(msg, handlers), options.signal);
}

export async function confirmExplorationDomains(
  sessionId: string,
  request: ConfirmExplorationDomainsRequest,
  handlers: ExplorationStreamHandlers,
  options: StreamOptions = {}
): Promise<void> {
  const path = `/api/exploration/sessions/${encodeURIComponent(sessionId)}/domains`;
  if (isTauri()) {
    const { Channel } = await import('@tauri-apps/api/core');
    const channel = new Channel<ExplorationStreamEvent>();
    channel.onmessage = (msg) => dispatchExplorationEvent(msg, handlers);
    await invoke('exploration_confirm_domains', {
      sessionId,
      request,
      onEvent: channel,
    });
    return;
  }
  await webStream<ExplorationStreamEvent>(
    path,
    request,
    (msg) => dispatchExplorationEvent(msg, handlers),
    options.signal
  );
}

export async function nextExplorationQuestion(sessionId: string): Promise<ExplorationSession> {
  if (isTauri()) {
    return invoke<ExplorationSession>('exploration_next_question', { sessionId });
  }
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}/question`, { method: 'POST' });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSession>(res);
}

export async function askExploration(sessionId: string, answer: string): Promise<ExplorationSession> {
  if (isTauri()) {
    return invoke<ExplorationSession>('exploration_ask', {
      sessionId,
      request: { answer },
    });
  }
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}/ask`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ answer }),
  });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSession>(res);
}

async function postSessionAction(sessionId: string, action: string, body?: unknown): Promise<ExplorationSession> {
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}/${action}`, {
    method: 'POST',
    ...(body === undefined
      ? {}
      : {
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(body),
        }),
  });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSession>(res);
}

export async function undoExploration(sessionId: string): Promise<ExplorationSession> {
  if (isTauri()) return invoke<ExplorationSession>('exploration_undo', { sessionId });
  return postSessionAction(sessionId, 'undo');
}

export async function redoExploration(sessionId: string): Promise<ExplorationSession> {
  if (isTauri()) return invoke<ExplorationSession>('exploration_redo', { sessionId });
  return postSessionAction(sessionId, 'redo');
}

export async function finishExploration(sessionId: string): Promise<ExplorationSession> {
  if (isTauri()) return invoke<ExplorationSession>('exploration_finish', { sessionId });
  return postSessionAction(sessionId, 'finish');
}

export async function skipExploration(sessionId: string): Promise<ExplorationSession> {
  if (isTauri()) return invoke<ExplorationSession>('exploration_skip', { sessionId });
  return postSessionAction(sessionId, 'skip');
}

export async function askExplorationAside(sessionId: string, question: string): Promise<ExplorationSession> {
  if (isTauri()) {
    return invoke<ExplorationSession>('exploration_aside', {
      sessionId,
      request: { question },
    });
  }
  return postSessionAction(sessionId, 'btw', { question });
}

export async function exportExplorationSession(sessionId: string): Promise<ExplorationExportResult> {
  if (isTauri()) {
    return invoke<ExplorationExportResult>('exploration_export', { sessionId });
  }
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}/export`, { method: 'POST' });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationExportResult>(res);
}

export async function saveExplorationToVault(sessionId: string, fileName: string): Promise<string> {
  if (isTauri()) {
    return invoke<string>('exploration_save_to_vault', { sessionId, fileName });
  }
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}/save`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ file_name: fileName }),
  });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<string>(res);
}

export async function importExplorationHistory(request: ImportExplorationRequest): Promise<ExplorationSession> {
  if (isTauri()) {
    return invoke<ExplorationSession>('exploration_import', { request });
  }
  const res = await fetch('/api/exploration/import', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(request),
  });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSession>(res);
}

export async function continueExploration(sessionId: string): Promise<ExplorationSession> {
  if (isTauri()) {
    return invoke<ExplorationSession>('exploration_continue', { sessionId });
  }
  const res = await fetch(`/api/exploration/sessions/${encodeURIComponent(sessionId)}/continue`, { method: 'POST' });
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<ExplorationSession>(res);
}
