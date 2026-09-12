import type { NoteTreeNode, NoteContent } from '../types/note';
import { invoke, isTauri } from './tauri';
import { throwHttpError, parseJsonResponse } from './http';

export async function selectFolder(): Promise<string | null> {
  if (isTauri()) {
    return invoke<string | null>('select_folder');
  }
  const path = window.prompt('Enter the full path to your notes folder:');
  return path || null;
}

export async function scanNotes(rootPath: string): Promise<NoteTreeNode[]> {
  if (isTauri()) {
    return invoke<NoteTreeNode[]>('scan_notes', { rootPath });
  }
  const params = new URLSearchParams({ root_path: rootPath });
  const res = await fetch(`/api/notes/scan?${params}`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<NoteTreeNode[]>(res);
}

export async function readNote(path: string): Promise<NoteContent> {
  if (isTauri()) {
    return invoke<NoteContent>('read_note', { path });
  }
  const params = new URLSearchParams({ path });
  const res = await fetch(`/api/notes/read?${params}`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<NoteContent>(res);
}
