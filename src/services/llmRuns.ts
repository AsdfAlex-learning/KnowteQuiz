import type { LlmRunKind, LlmRunRecord, LlmRunSummary } from '../types/llmRun';
import { invoke, isTauri } from './tauri';
import { throwHttpError, parseJsonResponse } from './http';

export interface LlmRunListFilter {
  limit?: number;
  kind?: LlmRunKind;
}

/**
 * List recorded LLM runs, newest first. Metadata only — the prompt and raw
 * response are fetched per-run via `getRunContent`.
 */
export async function listLlmRuns(filter?: LlmRunListFilter): Promise<LlmRunSummary[]> {
  if (isTauri()) {
    return invoke<LlmRunSummary[]>('llm_list_runs', {
      limit: filter?.limit ?? null,
      kind: filter?.kind ?? null,
    });
  }
  const query = runListQuery(filter);
  const res = await fetch(`/api/llm/runs${query}`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<LlmRunSummary[]>(res);
}

function runListQuery(filter?: LlmRunListFilter): string {
  const params = new URLSearchParams();
  if (filter?.limit !== undefined) params.set('limit', String(filter.limit));
  if (filter?.kind) params.set('kind', filter.kind);
  const query = params.toString();
  return query ? `?${query}` : '';
}

/** Full record for a single run (still no prompt/response bodies). */
export async function getLlmRun(id: string): Promise<LlmRunRecord> {
  if (isTauri()) {
    return invoke<LlmRunRecord>('llm_get_run', { id });
  }
  const res = await fetch(`/api/llm/runs/${encodeURIComponent(id)}`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<LlmRunRecord>(res);
}

/**
 * Read the recorded prompt or raw response of a run as plain text. Returns an
 * empty string when that content was not recorded (e.g. `record_llm_io` off).
 */
export async function getRunContent(id: string, field: 'prompt' | 'response'): Promise<string> {
  if (isTauri()) {
    return invoke<string>('llm_get_run_content', { id, field });
  }
  const res = await fetch(`/api/llm/runs/${encodeURIComponent(id)}/${field}`);
  if (!res.ok) await throwHttpError(res);
  return res.text();
}
