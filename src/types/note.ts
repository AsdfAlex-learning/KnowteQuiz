export type NoteNodeKind = 'markdown' | 'document';

export interface NoteTreeNode {
  name: string;
  path: string;
  /** `markdown` = readable/quizzable note; `document` = conversion candidate. */
  kind?: NoteNodeKind;
  is_dir: boolean;
  children: NoteTreeNode[];
}

export interface NoteContent {
  path: string;
  title: string;
  content: string;
  metadata: Record<string, string>;
}
