import { describe, expect, it } from 'vitest';
import { nextAutoName, uniqueName } from '../shared/naming';

describe('auto-naming (UI-INT-09)', () => {
  it('starts at 001 and fills the lowest free number', () => {
    expect(nextAutoName('wall', [])).toBe('wall-001');
    expect(nextAutoName('wall', ['wall-001', 'wall-003'])).toBe('wall-002');
    expect(nextAutoName('wall', ['wall-001', 'wall-002'])).toBe('wall-003');
  });

  it('is per-prefix and ignores unrelated names', () => {
    expect(nextAutoName('beam', ['wall-001', 'beam-001'])).toBe('beam-002');
    expect(nextAutoName('wall', ['Wall-001'])).toBe('wall-001');
  });

  it('keeps the pattern beyond 999 instead of colliding', () => {
    expect(nextAutoName('pipe', Array.from({ length: 999 }, (_, i) => `pipe-${String(i + 1).padStart(3, '0')}`))).toBe('pipe-1000');
  });

  it('uniqueName appends a suffix only when needed', () => {
    expect(uniqueName('wall-001', ['wall-001'])).toBe('wall-001-2');
    expect(uniqueName('wall-001', ['wall-001', 'wall-001-2'])).toBe('wall-001-3');
    expect(uniqueName('fresh', ['other'])).toBe('fresh');
  });
});
