import { reactive } from 'vue';

export interface FolderPickerState {
  open: boolean;
  startPath: string | null;
}

const state = reactive<FolderPickerState>({ open: false, startPath: null });
let resolver: ((path: string | null) => void) | null = null;

export function folderPickerState(): FolderPickerState {
  return state;
}

/**
 * Opens the in-app directory browser and resolves with the chosen path (or
 * `null` when cancelled). Used by the web runtime, where the browser cannot
 * hand the backend an absolute folder path.
 */
export function openFolderPicker(startPath?: string): Promise<string | null> {
  state.startPath = startPath ?? null;
  state.open = true;
  return new Promise((resolve) => {
    // A second request replaces the first; never leave an old promise pending.
    resolver?.(null);
    resolver = resolve;
  });
}

export function closeFolderPicker(path: string | null) {
  state.open = false;
  state.startPath = null;
  const pending = resolver;
  resolver = null;
  pending?.(path);
}
