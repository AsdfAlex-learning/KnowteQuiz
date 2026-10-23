import type { NoteTreeNode } from '../types/note';

/**
 * Flattens a note tree into the paths that can be selected as exploration
 * sources. Conversion candidates (`kind === 'document'`) are skipped: they are
 * not readable notes until they have been converted.
 */
export function collectMarkdownPaths(nodes: NoteTreeNode[]): string[] {
  const paths: string[] = [];
  for (const node of nodes) {
    if (node.is_dir) {
      paths.push(...collectMarkdownPaths(node.children));
    } else if (node.kind !== 'document') {
      paths.push(node.path);
    }
  }
  return paths;
}
