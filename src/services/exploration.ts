import type {
  ConfirmExplorationDomainsRequest,
  CreateExplorationSessionRequest,
  Domain,
  ExplorationSession,
  ExplorationSessionSummary,
  ExplorationStreamEvent,
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
