import { describe, expect, it } from 'vitest';
import { resolveInitialTheme } from '../shared/theme';

describe('theme resolution (UI-DS-02)', () => {
  it('uses an explicit stored choice', () => {
    expect(resolveInitialTheme('light', false)).toBe('light');
    expect(resolveInitialTheme('dark', true)).toBe('dark');
  });

  it('falls back to the OS preference', () => {
    expect(resolveInitialTheme(null, true)).toBe('light');
    expect(resolveInitialTheme(null, false)).toBe('dark');
    expect(resolveInitialTheme('auto', true)).toBe('light');
  });
});
