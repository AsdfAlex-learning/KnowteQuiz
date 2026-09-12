/**
 * Shared composable for preventing race conditions in async operations.
 * Each call to `next()` returns an incremented ID. Call `isLatest(id)`
 * to check if the operation is still the most recent one.
 */
export function useRequestId() {
  let id = 0;

  return {
    /** Returns the next request ID (increments the counter). */
    next(): number {
      return ++id;
    },
    /** Returns true if `checkId` matches the current (latest) request ID. */
    isLatest(checkId: number): boolean {
      return checkId === id;
    },
  };
}
