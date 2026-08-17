export interface NoteTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  children: NoteTreeNode[];
}

export interface NoteContent {
  path: string;
  title: string;
  content: string;
  metadata: Record<string, string>;
}
