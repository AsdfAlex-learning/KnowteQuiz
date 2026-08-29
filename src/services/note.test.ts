// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { readNote, scanNotes, selectFolder } from './note';

describe('note service', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('returns parsed note tree when web scanning succeeds', async () => {
    const mockTree = [
      { name: 'test.md', path: '/notes/test.md', is_dir: false, children: [] },
      { name: 'subdir', path: '/notes/subdir', is_dir: true, children: [] },
    ];
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        text: async () => JSON.stringify(mockTree),
      }))
    );

    const result = await scanNotes('/notes');

    expect(result).toEqual(mockTree);
  });

  it('returns parsed note content when web reading succeeds', async () => {
    const mockNote = { path: '/notes/test.md', title: 'Test', content: '# Hello', metadata: {} };
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        text: async () => JSON.stringify(mockNote),
      }))
    );

    const result = await readNote('/notes/test.md');

    expect(result).toEqual(mockNote);
  });

  it('prompts for folder path in web mode when selectFolder is called', async () => {
    vi.stubGlobal(
      'prompt',
      vi.fn(() => '/Users/test/notes')
    );

    const result = await selectFolder();

    expect(result).toBe('/Users/test/notes');
    expect(prompt).toHaveBeenCalledWith('Enter the full path to your notes folder:');
  });

  it('returns null when user cancels folder selection in web mode', async () => {
    vi.stubGlobal(
      'prompt',
      vi.fn(() => null)
    );

    const result = await selectFolder();

    expect(result).toBeNull();
  });

  it('includes the response body when web note scanning fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 404,
        text: async () => 'Directory does not exist: D:/missing',
      }))
    );

    await expect(scanNotes('D:/missing')).rejects.toThrow('HTTP 404: Directory does not exist: D:/missing');
  });

  it('includes the response body when web note reading fails', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: false,
        status: 500,
        text: async () => 'Failed to read file D:/notes/a.md',
      }))
    );

    await expect(readNote('D:/notes/a.md')).rejects.toThrow('HTTP 500: Failed to read file D:/notes/a.md');
  });
});
