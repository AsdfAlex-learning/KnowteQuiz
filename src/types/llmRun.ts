/** Token usage reported by the LLM endpoint (null when unavailable). */
export interface LlmUsage {
  prompt_tokens?: number;
  completion_tokens?: number;
  total_tokens?: number;
}

/**
 * How token usage was obtained:
 * - `body` — parsed from a non-streaming response body
 * - `stream_options` — parsed from an SSE chunk when the endpoint honoured
 *   `stream_options.include_usage`
 * - `unavailable` — the endpoint did not report usage
 */
export type LlmUsageSource = 'body' | 'stream_options' | 'unavailable';

export type LlmRunKind =
  'quiz' | 'quiz_fixup' | 'diagnosis_initial' | 'diagnosis_followup' | 'diagnosis_report' | 'connection_test' | 'probe';

/** List-view projection: metadata only, no prompt/response payloads. */
export interface LlmRunSummary {
  id: string;
  kind: LlmRunKind;
  ts_start: string;
  ts_end: string;
  duration_ms: number;
  model: string;
  usage: LlmUsage | null;
  usage_source: LlmUsageSource;
  http_status: number | null;
  error?: string;
}

/**
 * Full record for one LLM call. `prompt_file` / `response_file` are bare
 * filenames under the app data `debug/` directory — fetch their contents via
 * `getRunContent(id, 'prompt' | 'response')` rather than reading the fields.
 */
export interface LlmRunRecord extends LlmRunSummary {
  endpoint_host: string;
  request_params: unknown;
  prompt_file?: string | null;
  response_file?: string | null;
}
