/**
 * Shared HTTP helpers for the web (non-Tauri) runtime.
 * Used by quiz.ts, note.ts, settings.ts, mistake.ts.
 */

export async function throwHttpError(res: Response): Promise<never> {
  const body = await res.text();
  throw new Error(body ? `HTTP ${res.status}: ${body}` : `HTTP ${res.status}`);
}

export async function parseJsonResponse<T>(res: Response): Promise<T> {
  const text = await res.text();
  try {
    return JSON.parse(text) as T;
  } catch {
    throw new Error(text || `HTTP ${res.status}: Response is not valid JSON`);
  }
}
