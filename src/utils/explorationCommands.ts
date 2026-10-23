export type ExplorationCommand =
  | { kind: 'answer'; text: string }
  | { kind: 'undo' }
  | { kind: 'redo' }
  | { kind: 'skip' }
  | { kind: 'end' }
  | { kind: 'btw'; question: string };

/**
 * Parses one composer submission. Unknown `/` input is treated as a normal
 * answer so a stray slash never blocks the learner.
 */
export function parseExplorationCommand(input: string): ExplorationCommand {
  const text = input.trim();
  if (!text.startsWith('/')) return { kind: 'answer', text };

  const head = text.split(/\s+/, 1)[0].toLowerCase();
  switch (head) {
    case '/undo':
      return { kind: 'undo' };
    case '/redo':
      return { kind: 'redo' };
    case '/skip':
      return { kind: 'skip' };
    case '/end':
      return { kind: 'end' };
    case '/btw': {
      const question = text.slice(head.length).trim();
      return question ? { kind: 'btw', question } : { kind: 'answer', text };
    }
    default:
      return { kind: 'answer', text };
  }
}
