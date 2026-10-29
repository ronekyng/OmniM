import { describe, expect, it } from 'vitest';
import {
  beginSave,
  initialSaveState,
  markDirty,
  saveFailed,
  saveLabel,
  saveSucceeded,
  shouldWarnBeforeUnload,
} from '../shared/saveState';

describe('explicit save state machine (UI-DAT-02/03/06)', () => {
  it('dirty → saving → clean records the save time', () => {
    const dirty = markDirty(initialSaveState);
    expect(dirty.status).toBe('dirty');
    const saving = beginSave(dirty);
    expect(saving.status).toBe('saving');
    const clean = saveSucceeded(saving, Date.UTC(2026, 9, 29, 12, 34, 56));
    expect(clean.status).toBe('clean');
    expect(clean.lastSavedAt).not.toBeNull();
  });

  it('a failed save stays dirty for interception and retry', () => {
    const failed = saveFailed(markDirty(initialSaveState), 'disk full');
    expect(failed.status).toBe('failed');
    expect(failed.error).toBe('disk full');
    expect(shouldWarnBeforeUnload(failed)).toBe(true);
    expect(saveLabel(failed)).toBe('Save failed');
  });

  it('mutations do not downgrade an in-flight save', () => {
    const saving = beginSave(markDirty(initialSaveState));
    expect(markDirty(saving)).toBe(saving);
  });

  it('view-only sessions do not warn on unload', () => {
    expect(shouldWarnBeforeUnload(initialSaveState)).toBe(false);
    expect(saveLabel({ ...initialSaveState, lastSavedAt: Date.UTC(2026, 9, 29, 1, 2, 3) })).toMatch(/^Saved /);
  });
});
