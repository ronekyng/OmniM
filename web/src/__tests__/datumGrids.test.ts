import { describe, it, expect } from 'vitest';
import { gridLineEndpoints, DEFAULT_GRID_HALF_EXTENT } from '../app/components/viewer/three/DatumGrids';
import type { GridLine } from '../shared/types';

const line = (over: Partial<GridLine>): GridLine => ({
  id: 'g1',
  label: 'A',
  axis: 'AlongY',
  coordinate: 0,
  extent_min: 0,
  extent_max: 0,
  ...over,
});

describe('datum grid endpoints (P8-S3)', () => {
  it('AlongX lines run parallel to X at y = coordinate', () => {
    const { from, to } = gridLineEndpoints(line({ axis: 'AlongX', coordinate: 6, extent_min: 0, extent_max: 24 }));
    expect(from).toEqual({ x: 0, y: 6, z: 0 });
    expect(to).toEqual({ x: 24, y: 6, z: 0 });
  });

  it('AlongY lines run parallel to Y at x = coordinate', () => {
    const { from, to } = gridLineEndpoints(line({ axis: 'AlongY', coordinate: -3, extent_min: -5, extent_max: 5 }));
    expect(from).toEqual({ x: -3, y: -5, z: 0 });
    expect(to).toEqual({ x: -3, y: 5, z: 0 });
  });

  it('0/0 extents fall back to a bounded ±20 m span and reversed extents normalize', () => {
    const fallback = gridLineEndpoints(line({ extent_min: 0, extent_max: 0 }));
    expect(fallback.from.y).toBe(-DEFAULT_GRID_HALF_EXTENT);
    expect(fallback.to.y).toBe(DEFAULT_GRID_HALF_EXTENT);

    const reversed = gridLineEndpoints(line({ extent_min: 12, extent_max: 2 }));
    expect(reversed.from.y).toBe(2);
    expect(reversed.to.y).toBe(12);
  });
});
