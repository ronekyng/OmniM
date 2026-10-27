import type { LayerDefInput, ParamValueMap } from '../types';

export type Point3D = { x: number; y: number; z: number };

export type ObjectType =
  | 'wall' | 'slab' | 'roof' | 'floor'
  | 'road' | 'pipe' | 'beam' | 'column' | 'duct' | 'bridge' | 'tunnel'
  | 'door' | 'window'
  | 'tank' | 'pump' | 'valve' | 'gear' | 'equipment' | 'part' | 'assembly'
  | 'motor' | 'stair'
  | 'building' | 'storey' | 'space' | 'site'
  | 'point_cloud' | 'terrain_model' | 'feature_line' | 'contour_line';

export type GeometryType = 'box' | 'cylinder' | 'sphere' | 'cone';

export type CreationPhase = 'idle' | 'picking' | 'configuring' | 'creating' | 'error';

/** Interaction mode governing how many in-scene picks an object type needs,
 *  how they are collected and which gesture completes the picking session. */
export type PickMode =
  | { mode: 'polyline'; minClicks: 2 }  // wall, feature_line, contour_line, road, bridge, tunnel
  | { mode: 'polygon'; minClicks: 3 }   // slab, floor, roof (footprint)
  | { mode: 'segment'; minClicks: 2 }   // beam, pipe, duct, roof (extrusion)
  | { mode: 'place'; minClicks: 1 };    // column, tank, pump, valve, gear, equipment, part, assembly

/** Roof creation sub-mode (Phase 4.2). */
export type RoofMode = 'footprint' | 'extrusion';
/** Column creation sub-mode (Phase 4.1). */
export type ColumnMode = 'place' | 'segment';

export type CreationForm = {
  objectType: ObjectType;
  name: string;
  ifcClass: string;
  desc: string;
  parentId: string;
  seriesId: string;
  catalogType: string;
  /** Selected ComponentDef id (API-driven catalog) for CATALOG_TYPES. */
  catalogDefId: string;
  /** ParamDef-driven parameter values keyed by ParamDef name (mm units). */
  paramValues: ParamValueMap;
  /** Layer stack for surface types (wall/slab/roof/floor). */
  layers: LayerDefInput[];
  /** Serialized layer stack for undo/redo safety (JSON). */
  layersJson: string;
  posX: string; posY: string; posZ: string;
  startX: string; startY: string; startZ: string;
  endX: string; endY: string; endZ: string;
  /** All picked N points (polyline path / closed polygon boundary). */
  boundary_points: Point3D[];
  /** True when boundary_points form a closed polygon (slab/floor/roof footprint). */
  is_closed: boolean;
  /** Active interaction mode (derived from type + roofMode/columnMode). */
  pickMode: PickMode;
  /** Roof sub-mode: footprint polygon vs segment extrusion. */
  roofMode: RoofMode;
  /** Column sub-mode: single-point place (Revit) vs segment (Tekla). */
  columnMode: ColumnMode;
  /** Column height in metres (single-point mode). */
  columnHeight: string;
  geometryType: GeometryType;
  p1: string; p2: string; p3: string;
  length: string; height: string; width: string;
  core: string; finish: string;
  thickness: string; radius: string;
  targetWallId: string;
  openingWidth: string; openingHeight: string;
};

export type WallForm = CreationForm & { objectType: 'wall' };
export type SlabForm = CreationForm & { objectType: 'slab' };
export type BeamForm = CreationForm & { objectType: 'beam' };
export type ColumnForm = CreationForm & { objectType: 'column' };
export type RoadForm = CreationForm & { objectType: 'road' };
export type PipeForm = CreationForm & { objectType: 'pipe' };
export type DoorForm = CreationForm & { objectType: 'door' };
export type WindowForm = CreationForm & { objectType: 'window' };

export type CreationFormUnion =
  | WallForm | SlabForm | BeamForm | ColumnForm
  | RoadForm | PipeForm | DoorForm | WindowForm
  | CreationForm;

export const DEFAULT_LAYERS_WALL: LayerDefInput[] = [
  { name: '饰面层', thickness: 20, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
  { name: '结构层', thickness: 200, material_id: 'C30/37', material_name: 'C30/37 混凝土', function: 'structural', color: '#b0aaa0' },
  { name: '饰面层', thickness: 20, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
];

export const DEFAULT_LAYERS_SLAB: LayerDefInput[] = [
  { name: '结构层', thickness: 150, material_id: 'C30/37', material_name: 'C30/37 混凝土', function: 'structural', color: '#b0aaa0' },
  { name: '饰面层', thickness: 50, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
];

export const DEFAULT_LAYERS_ROOF: LayerDefInput[] = [
  { name: '结构层', thickness: 120, material_id: 'C30/37', material_name: 'C30/37 混凝土', function: 'structural', color: '#b0aaa0' },
  { name: '保温层', thickness: 60, material_id: 'Insulation', material_name: '保温材料', function: 'insulation', color: '#d8c8a0' },
  { name: '防水层', thickness: 8, material_id: 'Waterproof', material_name: '防水卷材', function: 'finish', color: '#9aa6b2' },
];

export const DEFAULT_LAYERS_FLOOR: LayerDefInput[] = [
  { name: '结构层', thickness: 150, material_id: 'C30/37', material_name: 'C30/37 混凝土', function: 'structural', color: '#b0aaa0' },
  { name: '饰面层', thickness: 50, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
];

export const DEFAULT_FORM: CreationForm = {
  objectType: 'wall',
  name: '',
  ifcClass: 'IfcWall',
  desc: '',
  parentId: '',
  seriesId: '',
  posX: '0', posY: '0', posZ: '0',
  startX: '0', startY: '0', startZ: '0',
  endX: '5', endY: '0', endZ: '0',
  geometryType: 'box',
  p1: '5', p2: '1', p3: '3',
  length: '5', height: '3', width: '3.5',
  core: '0.2', finish: '0.02',
  thickness: '0.3', radius: '0.3',
  targetWallId: '',
  openingWidth: '1.0', openingHeight: '2.1',
  catalogType: 'concrete-rect',
  catalogDefId: '',
  paramValues: {},
  layers: DEFAULT_LAYERS_WALL,
  layersJson: JSON.stringify(DEFAULT_LAYERS_WALL),
  boundary_points: [],
  is_closed: false,
  pickMode: { mode: 'segment', minClicks: 2 },
  roofMode: 'footprint',
  columnMode: 'place',
  columnHeight: '4',
};

export type ValidationResult = {
  valid: boolean;
  errors: Record<string, string>;
};

export function validateForm(form: CreationForm): ValidationResult {
  const errors: Record<string, string> = {};
  // Name is optional — the backend auto-generates one from the object type.
  if (form.objectType === 'door' || form.objectType === 'window') {
    if (!form.targetWallId) errors.targetWallId = '请选择目标墙体';
    const w = parseFloat(form.openingWidth);
    if (isNaN(w) || w <= 0) errors.openingWidth = '开口宽度必须大于 0';
    const h = parseFloat(form.openingHeight);
    if (isNaN(h) || h <= 0) errors.openingHeight = '开口高度必须大于 0';
  }
  return { valid: Object.keys(errors).length === 0, errors };
}

export function defaultPickModeForType(type: ObjectType): PickMode {
  switch (type) {
    case 'wall': case 'road': case 'bridge': case 'tunnel':
    case 'feature_line': case 'contour_line':
      return { mode: 'polyline', minClicks: 2 };
    case 'slab': case 'floor':
      return { mode: 'polygon', minClicks: 3 };
    case 'roof':
      return { mode: 'polygon', minClicks: 3 }; // footprint; switched to segment in extrusion mode
    case 'beam': case 'pipe': case 'duct':
      return { mode: 'segment', minClicks: 2 };
    default:
      return { mode: 'place', minClicks: 1 };
  }
}

export function needsPoints(mode: PickMode): boolean {
  return mode.mode === 'polyline' || mode.mode === 'polygon' || mode.mode === 'segment';
}

/** Number of in-scene clicks the type consumes (derived from its pick mode, in all workflows). */
export function neededClicksForType(type: ObjectType): number {
  const mode = defaultPickModeForType(type);
  if (!needsPoints(mode)) return 1;
  return mode.mode === 'polygon' ? 3 : 2;
}

/** Effective pick mode for the given form, honouring roof/column sub-modes. */
export function pickModeForForm(form: CreationForm): PickMode {
  if (form.objectType === 'roof') {
    return form.roofMode === 'extrusion'
      ? { mode: 'segment', minClicks: 2 }
      : { mode: 'polygon', minClicks: 3 };
  }
  if (form.objectType === 'column') {
    return form.columnMode === 'segment'
      ? { mode: 'segment', minClicks: 2 }
      : { mode: 'place', minClicks: 1 };
  }
  return defaultPickModeForType(form.objectType);
}

export function totalPolylineLength(points: Point3D[]): number {
  let total = 0;
  for (let i = 1; i < points.length; i++) {
    const dx = points[i].x - points[i - 1].x;
    const dy = points[i].y - points[i - 1].y;
    const dz = points[i].z - points[i - 1].z;
    total += Math.sqrt(dx * dx + dy * dy + dz * dz);
  }
  return total;
}

/** Update only the point-derived fields (pos / start / end / boundary / length)
 *  of a form from collected points, without touching user-edited fields. */
export function applyPointsToForm(form: CreationForm, points: Point3D[]): CreationForm {
  const next = { ...form };
  next.boundary_points = [...points];
  const mode = pickModeForForm(next);
  if (points.length >= 1) {
    next.posX = points[0].x.toFixed(3);
    next.posY = points[0].y.toFixed(3);
    next.posZ = points[0].z.toFixed(3);
  }
  next.is_closed = mode.mode === 'polygon' && points.length >= 3;
  if (mode.mode === 'polygon') {
    // Footprint is described by its boundary; length/start/end stay descriptive.
    if (points.length >= 2) {
      next.startX = points[0].x.toFixed(3);
      next.startY = points[0].y.toFixed(3);
      next.startZ = points[0].z.toFixed(3);
    }
  } else if (mode.mode === 'segment') {
    // Segment: P0/P1 define a single span (Tekla-style beam/pipe); N points
    // (polyline) define a continuous path, so bridged forms filter to span.
    const span = points.length >= 2 ? [points[0], points[points.length - 1]] : [] as Point3D[];
    const p0 = span.length === 2 ? span[0] : points[0];
    const p1 = span.length === 2 ? span[1] : points[0];
    if (span.length === 2) {
      next.startX = p0.x.toFixed(3); next.startY = p0.y.toFixed(3); next.startZ = p0.z.toFixed(3);
      next.endX = p1.x.toFixed(3); next.endY = p1.y.toFixed(3); next.endZ = p1.z.toFixed(3);
    }
    const lengthM = span.length === 2 ? totalPolylineLength(span) : 0;
    next.length = lengthM.toFixed(3);
    if (next.paramValues) {
      next.paramValues = { ...next.paramValues, length: Math.round(lengthM * 1000) };
    }
  } else {
    // polyline (wall/road/bridge/tunnel/feature/contour) — full N-point path.
    if (points.length >= 2) {
      next.startX = points[0].x.toFixed(3); next.startY = points[0].y.toFixed(3); next.startZ = points[0].z.toFixed(3);
      const last = points[points.length - 1];
      next.endX = last.x.toFixed(3); next.endY = last.y.toFixed(3); next.endZ = last.z.toFixed(3);
    }
    next.length = totalPolylineLength(points).toFixed(3);
    if (next.paramValues) {
      const lengthM = totalPolylineLength(points);
      next.paramValues = { ...next.paramValues, length: Math.round(lengthM * 1000) };
    }
  }
  return next;
}

export function createInitialForm(type: ObjectType, points: Point3D[] = []): CreationForm {
  const form: CreationForm = { ...DEFAULT_FORM, objectType: type };
  if (type === 'door') form.ifcClass = 'IfcDoor';
  else if (type === 'window') form.ifcClass = 'IfcWindow';
  else if (type === 'wall') form.ifcClass = 'IfcWall';
  else if (type === 'slab') form.ifcClass = 'IfcSlab';
  else if (type === 'beam') form.ifcClass = 'IfcBeam';
  else if (type === 'column') form.ifcClass = 'IfcColumn';
  else if (type === 'road') form.ifcClass = 'IfcRoad';
  else if (type === 'pipe') form.ifcClass = 'IfcPipe';
  else if (type === 'tank') form.ifcClass = 'IfcTank';
  else if (type === 'pump') form.ifcClass = 'IfcPump';
  else if (type === 'valve') form.ifcClass = 'IfcValve';
  else if (type === 'roof') form.ifcClass = 'IfcRoof';
  else if (type === 'floor') form.ifcClass = 'IfcFloor';
  else if (type === 'feature_line') form.ifcClass = 'IfcGeometricCurveSet';
  else if (type === 'contour_line') form.ifcClass = 'IfcGeometricCurveSet';
  if (type === 'wall') {
    form.layers = DEFAULT_LAYERS_WALL;
    form.layersJson = JSON.stringify(DEFAULT_LAYERS_WALL);
  } else if (type === 'slab') {
    form.layers = DEFAULT_LAYERS_SLAB;
    form.layersJson = JSON.stringify(DEFAULT_LAYERS_SLAB);
  } else if (type === 'roof') {
    form.layers = DEFAULT_LAYERS_ROOF;
    form.layersJson = JSON.stringify(DEFAULT_LAYERS_ROOF);
    form.roofMode = 'footprint';
  } else if (type === 'floor') {
    form.layers = DEFAULT_LAYERS_FLOOR;
    form.layersJson = JSON.stringify(DEFAULT_LAYERS_FLOOR);
  }
  form.pickMode = defaultPickModeForType(type);
  return applyPointsToForm(form, points);
}

export function assertState(condition: unknown, phase: CreationPhase, context: string): asserts condition {
  if (!condition) {
    throw new Error(`[${context}] Illegal state transition for phase: ${phase}`);
  }
}