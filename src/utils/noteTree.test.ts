import { describe, expect, it } from 'vitest';
import { collectMarkdownPaths } from './noteTree';
import type { NoteTreeNode } from '../types/note';

describe('collectMarkdownPaths', () => {
  it('flattens nested notes and skips conversion candidates', () => {
    const tree: NoteTreeNode[] = [
      { name: 'a.md', path: 'a.md', kind: 'markdown', is_dir: false, children: [] },
      {
        name: 'dir',
        path: 'dir',
        is_dir: true,
        children: [
          { name: 'b.md', path: 'dir/b.md', kind: 'markdown', is_dir: false, children: [] },
          {
            name: 'c.pdf',
            path: 'dir/c.pdf',
            kind: 'document',
            is_dir: false,
            children: [],
          },
        ],
      },
    ];

    expect(collectMarkdownPaths(tree)).toEqual(['a.md', 'dir/b.md']);
  });

  it('treats nodes without a kind as markdown', () => {
    const tree: NoteTreeNode[] = [{ name: 'legacy.md', path: 'legacy.md', is_dir: false, children: [] }];

    expect(collectMarkdownPaths(tree)).toEqual(['legacy.md']);
  });
});
