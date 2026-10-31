/**
 * Skeleton timing contract (FEEDBACK_ERRORS §3.2 / UI-FBK-02):
 * nothing appears for the first 400 ms, then a skeleton takes over.
 *
 * Kept as a pure predicate so the threshold is unit-testable without a DOM.
 */
export const SKELETON_DELAY_MS = 400;

export function shouldShowSkeleton(
  active: boolean,
  elapsedMs: number,
  delayMs: number = SKELETON_DELAY_MS,
): boolean {
  return active && elapsedMs >= delayMs;
}
