export type SessionStatus = 'initializing' | 'awaiting_domain_confirm' | 'active' | 'completed';

export type SessionOrigin = { kind: 'new' } | { kind: 'imported'; source_path: string };

export type Verdict = 'correct' | 'partial' | 'wrong' | 'unknown';

export type NextAction = 'question' | 'hint' | 'advance' | 'complete';

export interface SourceRef {
  note_path: string;
  heading?: string | null;
  snippet: string;
}

/** Per-note extraction from the map stage of initialization. */
export interface NoteDigest {
  path: string;
  title: string;
  headings: string[];
  summary: string;
  key_terms: string[];
  domain_guess: string;
}

export interface Concept {
  id: string;
  title: string;
  summary: string;
  depth: number;
  prereqs: string[];
  source_refs: SourceRef[];
  covered: boolean;
}

export interface Domain {
  id: string;
  title: string;
  concept_ids: string[];
}

export interface KnowledgeGraph {
  concepts: Concept[];
  domains: Domain[];
  root_order: string[];
  domains_confirmed: boolean;
}

export interface Question {
  prompt_md: string;
  hint_used: boolean;
}

export interface TurnResult {
  verdict: Verdict;
  feedback_md: string;
  next_action: NextAction;
  hint?: string | null;
}

export interface WhiteboardStep {
  id: string;
  concept_id: string;
  title: string;
  content_md: string;
  group_index: number;
}

export interface TurnGroup {
  concept_id: string;
  question: Question;
  answer?: string | null;
  result?: TurnResult | null;
  whiteboard?: WhiteboardStep | null;
  covered_after: string[];
}

export interface ExplorationSession {
  id: string;
  title: string;
  format_version: number;
  created_at: string;
  updated_at: string;
  status: SessionStatus;
  origin: SessionOrigin;
  source_paths: string[];
  source_digests?: NoteDigest[];
  graph: KnowledgeGraph;
  groups: TurnGroup[];
  active_group: number;
  domain_warning?: string | null;
  completed_early: boolean;
}

export interface ExplorationSessionSummary {
  id: string;
  title: string;
  status: SessionStatus;
  created_at: string;
  updated_at: string;
  source_paths: string[];
  concepts_total: number;
  concepts_covered: number;
  group_count: number;
  completed_early: boolean;
}

export interface CreateExplorationSessionRequest {
  source_paths: string[];
  title?: string;
}

export interface ConfirmExplorationDomainsRequest {
  domains: Domain[];
}

export type ExplorationStreamEvent =
  | { event: 'phase'; data: { phase: string } }
  | { event: 'domain_warning'; data: { domains: Domain[]; warning: string } }
  | { event: 'graph_ready'; data: { graph: KnowledgeGraph } }
  | { event: 'error'; data: { message: string } }
  | { event: 'done'; data: { session_id: string } };
