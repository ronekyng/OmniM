import { describe, expect, it } from 'vitest';
import { SKELETON_DELAY_MS, shouldShowSkeleton } from '../shared/delayedVisibility';

describe('skeleton timing (UI-FBK-02)', () => {
  it('shows nothing before 400 ms', () => {
    expect(shouldShowSkeleton(true, 0)).toBe(false);
    expect(shouldShowSkeleton(true, SKELETON_DELAY_MS - 1)).toBe(false);
  });

  it('shows the skeleton at and after the threshold', () => {
    expect(shouldShowSkeleton(true, SKELETON_DELAY_MS)).toBe(true);
    expect(shouldShowSkeleton(true, 3_000)).toBe(true);
  });

  it('never shows a skeleton when not loading', () => {
    expect(shouldShowSkeleton(false, 10_000)).toBe(false);
  });

  it('supports a custom delay for slow panels', () => {
    expect(shouldShowSkeleton(true, 199, 200)).toBe(false);
    expect(shouldShowSkeleton(true, 200, 200)).toBe(true);
  });
});
