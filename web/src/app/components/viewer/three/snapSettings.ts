// snapSettings: pure module (no `three` import) so the app store can carry
// snapping defaults without pulling the viewer graph into the initial chunk
// (PRF-01/06).
export type SnapMode = 'vertex' | 'edge' | 'midpoint' | 'endpoint' | 'grid';

export type SnapResult = {
  point: { x: number; y: number; z: number };
  snapped: boolean;
  snapType: SnapMode | null;
  distance: number;
};

export type SnapSettings = {
  enabled: boolean;
  toVertex: boolean;
  toEdge: boolean;
  toMidpoint: boolean;
  toEndpoint: boolean;
  toGrid: boolean;
  threshold: number; // world units
  gridSize: number;
};

export const DEFAULT_SNAP_SETTINGS: SnapSettings = {
  enabled: true,
  toVertex: true,
  toEdge: false,
  toMidpoint: false,
  toEndpoint: true,
  toGrid: true,
  threshold: 0.5,
  gridSize: 1.0,
};