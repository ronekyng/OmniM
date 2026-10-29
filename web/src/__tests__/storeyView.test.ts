import { describe, it, expect } from 'vitest';
import { meshVisibleOnStorey } from '../shared/storeyView';

describe('storey view filter (P8-S4)', () => {
  it('shows everything when no storey is active', () => {
    expect(meshVisibleOnStorey({ level_id: 'f1', bottom_level_id: null }, null)).toBe(true);
  });

  it('filters by level_id, falling back to bottom_level_id', () => {
    expect(meshVisibleOnStorey({ level_id: 'f1' }, 'f1')).toBe(true);
    expect(meshVisibleOnStorey({ level_id: 'f1' }, 'f2')).toBe(false);
    expect(meshVisibleOnStorey({ level_id: null, bottom_level_id: 'f2' }, 'f2')).toBe(true);
    expect(meshVisibleOnStorey({ level_id: null, bottom_level_id: 'f1' }, 'f2')).toBe(false);
    // level_id wins over bottom_level_id when both are present.
    expect(meshVisibleOnStorey({ level_id: 'f2', bottom_level_id: 'f1' }, 'f1')).toBe(false);
  });

  it('keeps unassigned geometry visible on every storey', () => {
    expect(meshVisibleOnStorey({}, 'f1')).toBe(true);
  });
});
