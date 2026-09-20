import type { WorkspaceState } from '@/types/settings';

/**
 * Update the daily activity streak in workspace state.
 *
 * Rules:
 * - If last_active_date is today → no change.
 * - If last_active_date is yesterday → streak + 1.
 * - If last_active_date is older or null → streak = 1.
 * - Always sets last_active_date to today.
 */
export function updateStreak(workspace: WorkspaceState): void {
  const today = new Date().toISOString().slice(0, 10);
  const last = workspace.last_active_date;

  if (last === today) return;

  if (last) {
    const diff = daysBetween(last, today);
    workspace.streak_days = diff === 1 ? (workspace.streak_days ?? 0) + 1 : 1;
  } else {
    workspace.streak_days = 1;
  }

  workspace.last_active_date = today;
}

function daysBetween(a: string, b: string): number {
  const da = new Date(a + 'T00:00:00Z');
  const db = new Date(b + 'T00:00:00Z');
  return Math.round((db.getTime() - da.getTime()) / 86_400_000);
}
