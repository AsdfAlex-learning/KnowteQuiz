import type {
  CreateExplorationSessionRequest,
  ExplorationSession,
  ExplorationSessionSummary,
} from '../types/exploration';
import { invoke, isTauri } from './tauri';
import { parseJsonResponse, throwHttpError } from './http';

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
