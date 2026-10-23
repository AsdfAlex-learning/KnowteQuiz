import { describe, expect, it } from 'vitest';
import { parseExplorationCommand } from './explorationCommands';

describe('parseExplorationCommand', () => {
  it('treats plain text as an answer', () => {
    expect(parseExplorationCommand('  Because symbols map to tokens.  ')).toEqual({
      kind: 'answer',
      text: 'Because symbols map to tokens.',
    });
  });

  it('parses the control commands case-insensitively', () => {
    expect(parseExplorationCommand('/undo')).toEqual({ kind: 'undo' });
    expect(parseExplorationCommand('/UNDO')).toEqual({ kind: 'undo' });
    expect(parseExplorationCommand('/redo')).toEqual({ kind: 'redo' });
    expect(parseExplorationCommand('/skip')).toEqual({ kind: 'skip' });
    expect(parseExplorationCommand('/end')).toEqual({ kind: 'end' });
  });

  it('parses a side question', () => {
    expect(parseExplorationCommand('/btw why is linking separate?')).toEqual({
      kind: 'btw',
      question: 'why is linking separate?',
    });
  });

  it('falls back to an answer for unknown or incomplete commands', () => {
    expect(parseExplorationCommand('/btw')).toEqual({ kind: 'answer', text: '/btw' });
    expect(parseExplorationCommand('/nope')).toEqual({ kind: 'answer', text: '/nope' });
    expect(parseExplorationCommand('/')).toEqual({ kind: 'answer', text: '/' });
  });
});
