import { describe, it, expect } from 'vitest';
import * as THREE from 'three';
import { bendExtensions, addLayeredWallPreview } from '../app/components/viewer/three/Markers';
import { polygonPreviewError } from '../shared/creation/polyline';
import type { LayerDefInput } from '../shared/types';
import type { Point3D } from '../shared/types/creation';

const layers: LayerDefInput[] = [
  { name: 'core', thickness: 200, material_id: 'C30/37', function: 'structural', color: '#cccccc' }, // 200 mm → 0.2 m
];
const twoLayers: LayerDefInput[] = [
  { name: 'a', thickness: 100, material_id: 'C30/37', function: 'structural', color: '#aabbcc' },
  { name: 'b', thickness: 100, material_id: 'Mortar', function: 'finish', color: '#ccbbaa' },
];

const pt = (x: number, y: number, z = 0): Point3D => ({ x, y, z });

describe('bendExtensions (miter allowance)', () => {
  it('straight line → zero extension at every vertex', () => {
    const ext = bendExtensions([pt(0, 0), pt(1, 0), pt(2, 0), pt(3, 0)], 0.2);
    expect(ext.every((e) => e === 0)).toBe(true);
  });

  it('90° L-bend with t=0.2 → extension = t/2 at the bend, 0 at endpoints', () => {
    const ext = bendExtensions([pt(0, 0), pt(5, 0), pt(5, 4)], 0.2);
    expect(ext[0]).toBe(0);
    expect(ext[2]).toBe(0);
    expect(ext[1]).toBeCloseTo(0.1, 6); // (t/2)·tan(45°/2·2)= t/2·tan(45°)=0.1? tan(φ/2)=tan45=1 → t/2
  });

  it('sharp bend is capped (no exploding extents)', () => {
    const ext = bendExtensions([pt(0, 0), pt(1, 0), pt(1.001, 1)], 0.2);
    expect(ext[1]).toBeLessThanOrEqual(2 * 0.2);
  });
});

describe('addLayeredWallPreview (unit scale / geometry)', () => {
  it('layer thickness uses the server scale (mm→m), not the old 20× factor', () => {
    const group = new THREE.Group();
    addLayeredWallPreview(group, [pt(0, 0), pt(5, 4)], layers, 3);
    const inner = group.children[0] as THREE.Group;
    const box = inner.children[0] as THREE.Mesh;
    // BoxGeometry(run, thickness, height) — depth (y) must be 0.2 m, not ~4 m.
    // BoxGeometry(width, height, depth) = (run, thickness, wall height)
    const p = (box.geometry as THREE.BoxGeometry).parameters;
    expect(p.height).toBeCloseTo(0.2, 4); // thickness axis
    expect(p.depth).toBeCloseTo(3, 4);    // wall height axis
  });

  it('multi-segment path produces one box per segment per layer, boxes span the corners', () => {
    const group = new THREE.Group();
    addLayeredWallPreview(group, [pt(0, 0), pt(5, 0), pt(5, 4)], twoLayers, 3);
    expect(group.children.length).toBe(1); // one inner group for the path
    const inner = group.children[0] as THREE.Group;
    expect(inner.children.length).toBe(4); // 2 layers × 2 segments = 4 flat boxes
  });

  it('fewer than 2 points → no preview group', () => {
    const group = new THREE.Group();
    expect(addLayeredWallPreview(group, [pt(0, 0)], layers, 3)).toBeNull();
  });
});

describe('polygonPreviewError (front-side validation, mirrors server validate_boundary)', () => {
  it('accepts a proper quad', () => {
    expect(polygonPreviewError([pt(0, 0), pt(4, 0), pt(4, 3), pt(0, 3)])).toBeNull();
  });

  it('rejects a self-intersecting footprint (nonzero-area crossing quad)', () => {
    expect(polygonPreviewError([pt(0, 0), pt(5, 5), pt(5, 0), pt(2, 3)])).toMatch(/自相交/);
  });

  it('rejects the zero-area bowtie (area or crossing rule, both valid)', () => {
    expect(polygonPreviewError([pt(0, 0), pt(4, 3), pt(4, 0), pt(0, 3)])).not.toBeNull();
  });

  it('rejects collinear degenerate footprints', () => {
    expect(polygonPreviewError([pt(0, 0), pt(2, 0), pt(4, 0)])).toMatch(/共线|面积为零/);
  });

  it('rejects duplicate consecutive vertices', () => {
    expect(polygonPreviewError([pt(0, 0), pt(4, 0), pt(4, 3), pt(4, 3)])).toMatch(/重合/);
  });
});
