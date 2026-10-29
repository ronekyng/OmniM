import { describe, it, expect } from 'vitest';
import * as THREE from 'three';
import {
  createInitialForm,
  defaultFormForType,
  pickModeForForm,
  previewFormFor,
  storeyOptions,
  validateForm,
  withGenesisForm,
  type CreationForm,
} from '../shared/types/creation';
import { addPathPreview } from '../app/components/viewer/three/Markers';
import { DEFAULT_LAYERS_WALL } from '../shared/types/creation';

describe('genesis form defaults (DATA_MODEL §1 matrix defaults)', () => {
  it('wall defaults to panel, slab/floor/roof to region', () => {
    expect(defaultFormForType('wall')).toBe('panel');
    expect(defaultFormForType('slab')).toBe('region');
    expect(defaultFormForType('floor')).toBe('region');
    expect(defaultFormForType('roof')).toBe('region');
  });

  it('createInitialForm derives the pick mode from the form', () => {
    const wall = createInitialForm('wall');
    expect(wall.form).toBe('panel');
    expect(pickModeForForm(wall)).toEqual({ mode: 'polyline', minClicks: 2 });

    const slab = createInitialForm('slab');
    expect(slab.form).toBe('region');
    expect(pickModeForForm(slab)).toEqual({ mode: 'polygon', minClicks: 3 });
  });
});

describe('wall form switch → pickMode (single form → pickMode lookup)', () => {
  it('wall panel picks a polyline, wall region picks a polygon', () => {
    const wall = createInitialForm('wall');
    const region = withGenesisForm(wall, 'region');
    expect(pickModeForForm(region)).toEqual({ mode: 'polygon', minClicks: 3 });
    expect(pickModeForForm(withGenesisForm(region, 'panel'))).toEqual({ mode: 'polyline', minClicks: 2 });
  });

  it('roof sub-modes keep driving the form kind', () => {
    const roof = createInitialForm('roof');
    expect(roof.form).toBe('region');
    const extrusion: CreationForm = { ...roof, roofMode: 'extrusion', form: 'panel' };
    expect(pickModeForForm(extrusion)).toEqual({ mode: 'segment', minClicks: 2 });
  });
});

describe('previewFormFor (form → preview language)', () => {
  it('maps surface/linear/point families', () => {
    expect(previewFormFor(createInitialForm('wall'))).toBe('panel');
    expect(previewFormFor(createInitialForm('slab'))).toBe('region');
    expect(previewFormFor(createInitialForm('road'))).toBe('linear');
    expect(previewFormFor(createInitialForm('feature_line'))).toBe('point');
    expect(previewFormFor(createInitialForm('column'))).toBe('linear');
  });
});

describe('addPathPreview dispatch (unified preview)', () => {
  const pts = [
    { x: 0, y: 0, z: 0 },
    { x: 4, y: 0, z: 0 },
    { x: 4, y: 3, z: 0 },
  ];

  it('region draws outline + closing edge + fill', () => {
    const g = new THREE.Group();
    addPathPreview(g, { points: pts, closed: true, form: 'region' });
    // 2 outline lines (open chain + closing) + 1 fill mesh
    expect(g.children.filter((c) => c instanceof THREE.Line).length).toBe(2);
    expect(g.children.filter((c) => c instanceof THREE.Mesh).length).toBe(1);
  });

  it('panel with layers builds the layered ghost as a single sub-group', () => {
    const g = new THREE.Group();
    addPathPreview(g, { points: pts, closed: false, form: 'panel', layers: DEFAULT_LAYERS_WALL, height: 3 });
    expect(g.children.length).toBe(1);
    expect(g.children[0]).toBeInstanceOf(THREE.Group);
  });

  it('panel without layers falls back to an outline line', () => {
    const g = new THREE.Group();
    addPathPreview(g, { points: pts, closed: false, form: 'panel' });
    expect(g.children.length).toBe(1);
    expect(g.children[0]).toBeInstanceOf(THREE.Line);
  });

  it('closed panel loops repeat the first point for the closing bend', () => {
    const g = new THREE.Group();
    addPathPreview(g, { points: pts, closed: true, form: 'panel', layers: DEFAULT_LAYERS_WALL, height: 3 });
    const sub = g.children[0] as THREE.Group;
    // closed triangle loop: 3 segments × 3 layers (the appended first point
    // makes the last window the closing edge)
    expect(sub.children.length).toBe(9);
  });

  it('point form adds markers for every picked point', () => {
    const g = new THREE.Group();
    addPathPreview(g, { points: pts, closed: false, form: 'point' });
    expect(g.children.filter((c) => c instanceof THREE.Mesh).length).toBe(3);
  });
});

describe('level-driven extent (P8-S3)', () => {
  it('storeyOptions keeps only storeys, defaults missing elevations to 0, sorts by elevation', () => {
    const options = storeyOptions([
      { id: 'f2', name: 'F2', object_type: 3, elevation: 3.9 },
      { id: 'w1', name: 'W1', object_type: 100 },
      { id: 'f1', name: 'F1', object_type: 3, elevation: 0 },
      { id: 'b1', name: 'B1', object_type: 3, elevation: -3.5 },
    ]);
    expect(options.map((o) => o.id)).toEqual(['b1', 'f1', 'f2']);
    expect(storeyOptions([{ id: 'x', name: 'X', object_type: 3 }])[0].elevation).toBe(0);
  });

  it('extent refs are both-or-neither, distinct, and only offered on extent types', () => {
    const wall = createInitialForm('wall');
    expect(validateForm({ ...wall, bottomLevelId: 'f1' }).errors.bottomLevelId).toBeTruthy();
    expect(
      validateForm({ ...wall, bottomLevelId: 'f1', topLevelId: 'f1' }).errors.bottomLevelId,
    ).toBeTruthy();
    expect(validateForm({ ...wall, bottomLevelId: 'f1', topLevelId: 'f2' }).valid).toBe(true);

    // Columns are not extent-capable in the UI (catalog Component path).
    const column = createInitialForm('column');
    expect(validateForm({ ...column, bottomLevelId: 'f1' }).valid).toBe(true);
  });
});
