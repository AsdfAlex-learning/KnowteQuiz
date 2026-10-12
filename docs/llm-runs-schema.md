# LLM Run Schema & Query API

> **Audience**: tool authors, plugin authors, and anyone who wants to inspect or
> analyse KnowteQuiz's LLM traffic outside the built-in panel.

KnowteQuiz records every LLM call it makes. This document defines the on-disk
format and the query interface so external tools, custom pages, or your own
scripts can consume it.

---

## 1. Storage layout

Everything lives under the app data directory (see README → *Where your data
lives*):

```
<app data>/knowtequiz/
  llm_runs/
    llm_runs.jsonl                     # one JSON object per line (append-only)
  debug/
    {ts}_{uuid}_{kind}_raw.txt         # raw LLM response body
    {ts}_{uuid}_{kind}_prompt.txt      # full prompt text sent to the model
```

- `llm_runs.jsonl` is **append-only** and rolled to the newest **500** records.
- `debug/` files are rolled to the newest **100** (oldest deleted).
- `prompt_file` / `response_file` in a record are **bare filenames** inside
  `debug/` (not full paths), so a consumer can build its own path or URL.

### Privacy switch

`settings.llm.record_llm_io` (boolean, default `true`) controls whether the
prompt and raw response bodies are written to `debug/`. When `false`:

- metadata (timing, token usage, status, error) **is still recorded**,
- `prompt_file` and `response_file` are `null`,
- the content routes return an empty body.

The API key is **never** recorded. `endpoint_host` is host:port only.

---

## 2. JSONL record schema

Each line is a JSON object, `snake_case`:

| Field | Type | Notes |
|---|---|---|
| `id` | `string` | UUID (dashes stripped) |
| `kind` | `string` enum | see below |
| `ts_start` | `string` (RFC 3339) | local time with offset |
| `ts_end` | `string` (RFC 3339) | local time with offset |
| `duration_ms` | `number` | wall-clock duration |
| `model` | `string` | configured model name |
| `endpoint_host` | `string` | e.g. `localhost:11434` — never credentials |
| `request_params` | `object` | `temperature`, `max_tokens`, `stream`, `response_format`, `stream_options` |
| `usage` | `object \| null` | `prompt_tokens` / `completion_tokens` / `total_tokens` (each optional) |
| `usage_source` | `string` enum | `body` \| `stream_options` \| `unavailable` |
| `http_status` | `number \| null` | HTTP status of the LLM response |
| `error` | `string \| null` | error message when the call failed |
| `prompt_file` | `string \| null` | filename in `debug/` (see privacy switch) |
| `response_file` | `string \| null` | filename in `debug/` (see privacy switch) |

### `kind` values

| Value | Meaning |
|---|---|
| `quiz` | streaming quiz generation |
| `quiz_fixup` | the one-shot JSON auto-repair retry |
| `diagnosis_initial` | advanced-mode initial blind-spot diagnosis |
| `diagnosis_followup` | advanced-mode follow-up round |
| `diagnosis_report` | advanced-mode final report |
| `connection_test` | Settings → Test Connection |
| `probe` | Settings → Probe LLM Capabilities |

### `usage_source` semantics

| Value | Meaning |
|---|---|
| `body` | non-streaming response carried a `usage` object |
| `stream_options` | the endpoint honoured `stream_options.include_usage` and sent a usage chunk |
| `unavailable` | the endpoint reported no usage (common with local models) — `usage` is `null` |

---

## 3. Query API

### 3.1 REST (web mode)

The Axum web server (`node start.cjs --web`) exposes read-only endpoints. All
are `GET`.

| Endpoint | Returns | Notes |
|---|---|---|
| `/api/llm/runs?limit=&kind=` | `LlmRunSummary[]` | newest first; `limit` defaults to 50 |
| `/api/llm/runs/:id` | `LlmRunRecord` | full record incl. file references |
| `/api/llm/runs/:id/prompt` | `text/plain` | empty body when not recorded |
| `/api/llm/runs/:id/response` | `text/plain` | empty body when not recorded |

Content routes send `X-Content-Type-Options: nosniff`.

#### Examples

```bash
# List the 20 most recent quiz-generation runs
curl -s 'http://127.0.0.1:14200/api/llm/runs?limit=20&kind=quiz' | jq

# Full metadata for one run
curl -s 'http://127.0.0.1:14200/api/llm/runs/abc123' | jq

# The prompt that was sent
curl -s 'http://127.0.0.1:14200/api/llm/runs/abc123/prompt'

# The raw model response
curl -s 'http://127.0.0.1:14200/api/llm/runs/abc123/response'
```

Python:

```python
import json, urllib.request

BASE = "http://127.0.0.1:14200"

def get(path):
    with urllib.request.urlopen(f"{BASE}{path}") as r:
        return r.read().decode()

runs = json.loads(get("/api/llm/runs?limit=10"))
for run in runs:
    print(run["kind"], run["duration_ms"], "ms", run["usage"])

prompt = get(f"/api/llm/runs/{runs[0]['id']}/prompt")
response = get(f"/api/llm/runs/{runs[0]['id']}/response")
```

Node / any SSE-capable tool:

```js
const runs = await (await fetch('http://127.0.0.1:14200/api/llm/runs?limit=10')).json();
const detail = await (await fetch(`http://127.0.0.1:14200/api/llm/runs/${runs[0].id}`)).json();
const prompt = await (await fetch(`http://127.0.0.1:14200/api/llm/runs/${runs[0].id}/prompt`)).text();
```

### 3.2 Tauri commands (desktop mode)

| Command | Arguments | Returns |
|---|---|---|
| `llm_list_runs` | `limit?: number`, `kind?: string` | `LlmRunSummary[]` |
| `llm_get_run` | `id: string` | `LlmRunRecord` |
| `llm_get_run_content` | `id: string`, `field: "prompt" \| "response"` | `string` |

### 3.3 File access (works in every mode)

`llm_runs.jsonl` is plain JSONL — `tail`, `grep`, `jq`, or a spreadsheet import
all work without any API. Example:

```bash
# Every failed call with its error
jq -c 'select(.error != null) | {id, kind, ts_end, error}' <app-data>/llm_runs/llm_runs.jsonl

# Token totals by kind
jq -s 'group_by(.kind) | map({kind: .[0].kind, total: (map(.usage.total_tokens // 0) | add)})' <app-data>/knowtequiz/llm_runs/llm_runs.jsonl
```

---

## 4. Browser access / CORS

Browser-origin tools (extensions, custom pages served from another origin) are
blocked by default. To allow them:

1. Add the origin to `settings.cors.allow_origins`, e.g.
   ```json
   { "cors": { "allow_origins": ["http://localhost:3000"] } }
   ```
2. **Restart the web server** — the CORS layer is built once at startup.

Non-browser tools (curl, Python, Node scripts) are unaffected by CORS and can
call the API directly.

---

## 5. Built-in panel

The app itself ships a **LLM** tab in the right-hand panel that consumes the
same API: it lists runs and expands to show the request params, prompt, and raw
response with copy buttons. Use it for a quick look; use the API/files above for
automation.

---

## 6. Notes & limitations

- `llm_runs.jsonl` is **not** included in `backup_data` (it is observability
  data, not user data) — it will not grow your backups.
- Timing is wall-clock around the HTTP request only; it excludes prompt
  construction and response parsing.
- Streaming runs record `stream_options: { include_usage: true }` in
  `request_params`; if the endpoint ignores it, `usage_source` is `unavailable`.
- A run record whose `prompt_file`/`response_file` points at a file that has been
  pruned will return an empty body from the content routes.
