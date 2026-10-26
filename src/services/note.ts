import type { DirectoryListing, NoteTreeNode, NoteContent } from '../types/note';
import { invoke, isTauri } from './tauri';
import { throwHttpError, parseJsonResponse } from './http';
import { openFolderPicker } from '../composables/useFolderPicker';

export async function selectFolder(): Promise<string | null> {
  if (isTauri()) {
    return invoke<string | null>('select_folder');
  }
  // Browsers cannot hand the backend an absolute path, so the web runtime uses
  // an in-app directory browser backed by the local server.
  return openFolderPicker();
}

export async function listDirectories(path?: string): Promise<DirectoryListing> {
  if (isTauri()) {
    return invoke<DirectoryListing>('list_directories', { path: path ?? null });
  }
  const query = path ? `?path=${encodeURIComponent(path)}` : '';
  const res = await fetch(`/api/fs/dirs${query}`);
  if (!res.ok) await throwHttpError(res);
  return parseJsonResponse<DirectoryListing>(res);
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
