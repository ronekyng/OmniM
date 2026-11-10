export type PropertyValue = {
  string_value?: string;
  int_value?: number;
  double_value?: number;
  bool_value?: boolean;
};

export type Property = {
  name: string;
  value?: PropertyValue;
  unit?: string;
  description?: string;
};

export type PropertySet = {
  name: string;
  description?: string;
  properties: Property[];
};

export type PropertyKind = 'double' | 'string' | 'bool' | 'choice';

export type ObjectPropertyDef = {
  name: string;
  label: string;
  kind: PropertyKind;
  unit: string;
  min: number;
  max: number;
  step: number;
  choices: string[];
  description: string;
  group: string;
};

export type CatalogRef = {
  component_def_id: string;
  series_id: string;
  selector_path: string[];
  param_values: Record<string, number>;
};

export type OmniObject = {
  id: string;
  name: string;
  description: string;
  object_type: number;
  ifc_class: string;
  tags: string[];
  properties: Record<string, PropertyValue>;
  property_sets: PropertySet[];
  catalog_ref?: CatalogRef | null;
  /** Datum elevation in metres (P8-S1); present on storeys. */
  elevation?: number | null;
};

export type UpdatePropertiesRequest = {
  name?: string;
  description?: string;
  ifc_class?: string;
  tags?: string[];
  properties?: Record<string, string>;
};

/** Fields accepted by `PUT /api/objects/{id}` (datum-aware update). */
export type UpdateObjectRequest = {
  name?: string;
  description?: string;
  ifc_class?: string;
  tags?: string[];
  elevation?: number;
  level_id?: string;
  bottom_level_id?: string;
  top_level_id?: string;
};

/** Grid datum line (轴网), P8-S1/S3. `axis` is the direction the line runs. */
export type GridLine = {
  id: string;
  label: string;
  axis: 'AlongX' | 'AlongY';
  coordinate: number;
  extent_min: number;
  extent_max: number;
};

export type TreeNode = {
  id: string;
  name: string;
  object_type: string;
  ifc_class: string;
  has_geometry: boolean;
  children: TreeNode[];
};

export type MeshObject = {
  id: string;
  name: string;
  object_type: string;
  vertex_count: number;
  triangle_count: number;
  positions: number[];
  normals: number[];
  bbox_min: [number, number, number];
  bbox_max: [number, number, number];
  /** Storey association (P8-S4 storey view). */
  level_id?: string | null;
  bottom_level_id?: string | null;
  elevation?: number | null;
};

export type MeshResponse = {
  objects: MeshObject[];
};

export type StatsResponse = {
  name: string;
  object_count: number;
  material_count: number;
  lifecycle_record_count: number;
  connection_count: number;
  containment_count: number;
};

export type ImportIfcResponse = {
  success: boolean;
  imported: number;
  objects: number;
  message: string;
};

// GenRule request types
export type GenRuleWallRequest = {
  genrule: 'wall';
  name: string;
  length: number;
  height: number;
  core_thickness: number;
  finish_thickness: number;
  parent_id?: string;
  series_id?: string;
  start_x?: number;
  start_y?: number;
  start_z?: number;
  end_x?: number;
  end_y?: number;
  end_z?: number;
};

export type GenRuleSlabRequest = {
  genrule: 'slab';
  name: string;
  length: number;
  width: number;
  structural_thickness: number;
  finish_thickness: number;
  parent_id?: string;
  series_id?: string;
};

export type GenRuleRoadRequest = {
  genrule: 'road';
  name: string;
  length: number;
  width: number;
  thickness: number;
  /** Picked polyline path (start/end or full N-point alignment). */
  alignment?: [number, number, number][];
  parent_id?: string;
  series_id?: string;
};

export type GenRulePipeRequest = {
  genrule: 'pipe';
  name: string;
  length: number;
  radius: number;
  parent_id?: string;
  series_id?: string;
};

export type GenRuleDoorRequest = {
  genrule: 'door';
  name: string;
  wall_id: string;
  opening_width: number;
  opening_height: number;
  position_x: number;
  position_z: number;
  parent_id?: string;
};

export type GenRuleWindowRequest = {
  genrule: 'window';
  name: string;
  wall_id: string;
  opening_width: number;
  opening_height: number;
  position_x: number;
  position_z: number;
  parent_id?: string;
};

/** Standalone opening volume (IfcOpeningElement), host-relative (P8). */
export type GenRuleOpeningRequest = {
  genrule: 'opening';
  name: string;
  wall_id: string;
  opening_width: number;
  opening_height: number;
  position_x: number;
  position_z: number;
  parent_id?: string;
};

export type AssemblyComponentDef = {
  name: string;
  shape: string;
  p1: number;
  p2: number;
  p3: number;
  tx: number;
  ty: number;
  tz: number;
};

/** One part of the new assembly container (docs/specification/ASSEMBLY.md). */
export type AssemblyPartInput = {
  name: string;
  /** Catalog part; when absent the part is a baked primitive. */
  component_def_id?: string;
  params?: Record<string, number>;
  shape?: string;
  p1?: number;
  p2?: number;
  p3?: number;
  /** Metres. */
  position?: { x: number; y: number; z: number };
  /** Radians about the position point. */
  rotation_z?: number;
};

export type CreateAssemblyRequest = {
  name: string;
  parent_id?: string;
  skip_series_gate?: boolean;
  parts: AssemblyPartInput[];
};

export type AssemblyPartInfo = {
  id: string;
  name: string;
  kind: 'component' | 'part';
};

export type CreateAssemblyResponse = {
  assembly_id: string;
  name: string;
  parts: AssemblyPartInfo[];
  part_count: number;
};

export type GenRuleAssemblyRequest = {
  genrule: 'assembly';
  name: string;
  components: AssemblyComponentDef[];
};

export type LayerRequestDef = {
  name: string;
  thickness: number;
  material_id?: string;
  function?: string;
};

export type GenRuleLayeredRequest = {
  genrule: 'layered';
  name: string;
  mode: 'wall' | 'slab' | 'roof' | 'floor';
  length?: number;
  width?: number;
  height?: number;
  layers: LayerRequestDef[];
  /** Continuous center-line path (wall): [x, y, z] triples in metres. */
  path?: [number, number, number][];
  /** Closed footprint boundary (slab/roof/floor): [x, y, z] triples. */
  boundary?: [number, number, number][];
  parent_id?: string;
  series_id?: string;
  start_x?: number;
  start_y?: number;
  start_z?: number;
  end_x?: number;
  end_y?: number;
  end_z?: number;
};

export type GenRuleLineRequest = {
  genrule: 'line';
  name: string;
  alignment?: [number, number, number][];
  diameter?: number;
  parent_id?: string;
};

export type GenRuleRequest =
  | GenRuleWallRequest
  | GenRuleSlabRequest
  | GenRuleRoadRequest
  | GenRulePipeRequest
  | GenRuleDoorRequest
  | GenRuleWindowRequest
  | GenRuleOpeningRequest
  | GenRuleAssemblyRequest
  | GenRuleLayeredRequest
  | GenRuleLineRequest;

// Undo/Redo
export type UndoRedoResponse = {
  success: boolean;
  message: string;
};

export type UndoStatusResponse = {
  can_undo: boolean;
  can_redo: boolean;
  undo_depth: number;
};

export type ExecutionResult = {
  success: boolean;
  message: string;
  created_ids: string[];
  /** Non-fatal caveats: approximate-kernel notices, etc. */
  warnings?: string[];
};

// AMIR AI types
export type AmirTranslateRequest = {
  text: string;
  selected_ids?: string[] | null;
};

export type AmirTranslateResponse = {
  success: boolean;
  commands: unknown[];
  summary: string;
  errors: string[];
};

export type AmirChatRequest = {
  session_id?: string | null;
  text: string;
  selected_ids?: string[] | null;
};

export type AmirChatResponse = {
  session_id: string;
  reply: string;
  commands: unknown[];
  summary: string;
};

export type AmirDescribeRequest = {
  object_ids?: string[] | null;
  format?: string | null;
};

export type AmirDescribeResponse = {
  description: string;
};

export type AmirExplainRequest = {
  errors: string[];
  context?: string | null;
};

export type AmirExplainResponse = {
  explanations: string[];
};

// ── Sketch types ──
export type SketchEntityType = 'Point' | 'Line' | 'Circle' | 'Arc' | 'Rectangle' | 'Polygon' | 'Ellipse' | 'Spline' | 'Polyline';

export type SketchEntity = {
  id: string;
  entity_type: SketchEntityType;
  points: { x: number; y: number; z: number }[];
  radius: number;
  sides: number;
  construction?: boolean;
};

export type SketchConstraintType =
  | 'Horizontal' | 'Vertical' | 'Coincident' | 'Parallel'
  | 'Perpendicular' | 'Length' | 'Distance' | 'Angle'
  | 'Radius' | 'Diameter' | 'Tangent' | 'Symmetric'
  | 'Fixed' | 'AtMidpoint' | 'EqualLength' | 'EqualRadius'
  | 'PtOnLine' | 'PtOnCircle' | 'SameOrientation';

export type SketchConstraint = {
  id: string;
  constraint_type: SketchConstraintType;
  entity_refs: { entity_index: number; point_index: number }[];
  value: number;
};

export type Sketch = {
  id: string;
  name: string;
  origin: { x: number; y: number; z: number };
  normal: { x: number; y: number; z: number };
  is_3d: boolean;
  solved: boolean;
  entities: SketchEntity[];
  constraints: SketchConstraint[];
};

export type SketchResponse = {
  id: string;
  name: string;
  solved: boolean;
  entity_count: number;
  constraint_count: number;
  component_id: string | null;
};

export type FeatureResponse = {
  success: boolean;
  message: string;
  vertices: number;
  faces: number;
  object_id: string;
};

export type ComponentResponse = {
  id: string;
  name: string;
  feature_count: number;
  sketch_ids: string[];
  parent_id: string | null;
};

export type FeatureItem = {
  id: string;
  name: string;
  feature_type: string;
  sketch_id: string;
  input_sketch_ids: string[];
  axis_sketch_id: string;
  depth: number;
  angle: number;
  is_cut: boolean;
  order: number;
  suppressed: boolean;
  dirty: boolean;
  dependency_ids: string[];
};

// ── Catalog types ──
export type ParamDef = {
  name: string;
  param_type: number;
  default_value: number;
  min_value: number;
  max_value: number;
  step: number;
  unit: string;
  formula: string;
  choices: string[];
  description: string;
};

export type ConnectorDef = {
  id: string;
  name: string;
  position_x_expr: string;
  position_y_expr: string;
  position_z_expr: string;
  direction_x_expr: string;
  direction_y_expr: string;
  direction_z_expr: string;
  bore_expr: string;
  connection_type_code: string;
  role: number;
};

export type ComponentDef = {
  id: string;
  name: string;
  category: string;
  description: string;
  params: ParamDef[];
  feature_tree_id: string;
  connectors: ConnectorDef[];
  allowed_materials: string[];
  default_material: string;
  ifc_class: string;
  standard_reference: string;
  tags: string[];
  domain: string;
  properties: Record<string, string>;
  published: boolean;
};

// ── Series types ──
export type ParamConstraint = {
  param_name: string;
  default_value: number;
  min_value: number;
  max_value: number;
  step: number;
  fixed_value: string;
};

export type SpecComponent = {
  id: string;
  component_def_id: string;
  param_constraints: ParamConstraint[];
  material: string;
  connection_type_code: string;
  design_params: Record<string, string>;
};

export type AnswerOutcome = {
  next_selector?: Selector;
  component?: SpecComponent;
};

export type Answer = {
  id: string;
  value: string;
  is_numeric_range: boolean;
  range_min: number;
  range_max: number;
  outcome?: AnswerOutcome;
};

export type Selector = {
  id: string;
  question: string;
  answers: Answer[];
};

export type Series = {
  id: string;
  name: string;
  description: string;
  domain: string;
  standard_reference: string;
  root_selector: Selector | null;
  applicable_phases: string[];
  metadata: Record<string, string>;
};

export type SelectResult = {
  component_def_id: string;
  selector_path: string[];
  spec_component: SpecComponent;
};

export type InstantiateResponse = {
  component_id: string;
  object_type: string;
  name: string;
};

export type PreviewRequest = {
  component_def_id: string;
  params: Record<string, number>;
};

export type CatalogPreviewMesh = {
  vertices: number[];
  normals: number[];
  uvs: number[];
  faces: number[];
};

export type ExportCatalogResponse = {
  path: string;
  component_count: number;
};

export type ImportCatalogResponse = {
  component_count: number;
  names: string[];
};

export type SaveSeriesResponse = {
  path: string;
  saved_count: number;
};

export type LoadSeriesResponse = {
  loaded_count: number;
  series_names: string[];
};

// ── Material types ──
export type MaterialCategory = 'METAL' | 'CONCRETE' | 'WOOD' | 'MASONRY' | 'PLASTIC' | 'COMPOSITE' | 'GLASS' | 'INSULATION' | 'SOIL' | 'WATER' | 'OTHER';

export type Material = {
  id: string;
  name: string;
  category: number;
  density: number;
  youngs_modulus: number;
  poissons_ratio: number;
  yield_strength: number;
  ultimate_strength: number;
  thermal_conductivity: number;
  specific_heat: number;
  color?: { r: number; g: number; b: number; a: number };
  roughness: number;
  metallic: number;
  transparency: number;
  ifc_material: string;
  standards: string[];
};

export const MATERIAL_CATEGORY_KEYS: Record<number, string> = {
  0: 'mat.cat.0',
  1: 'mat.cat.1',
  2: 'mat.cat.2',
  3: 'mat.cat.3',
  4: 'mat.cat.4',
  5: 'mat.cat.5',
  6: 'mat.cat.6',
  7: 'mat.cat.7',
  8: 'mat.cat.8',
  9: 'mat.cat.9',
  10: 'mat.cat.10',
  99: 'mat.cat.99',
};

// ── Layer structure (LayerEditor) types ──
export type LayerDefInput = {
  name: string;
  thickness: number;      // in mm
  material_id: string;
  material_name?: string;
  function: string;       // structural | insulation | finish | ...
  color?: string;         // hex preview color
};

export const LAYER_FUNCTIONS = ['structural', 'insulation', 'finish', 'membrane', 'vapor', 'vapor_barrier', 'finish_floor', 'finish_ceiling', 'cladding', 'waterproof'];

// ── Creation form extensions ──
export type CatalogParamValues = Record<string, number>;

/** A ComponentParamPanel value set: raw param values keyed by ParamDef name. */
export type ParamValueMap = CatalogParamValues;

export type CreateRequest = {
  object_type: string;
  name: string;
  description?: string;
  parent_id?: string;
  series_id?: string;
  geometry?: {
    operations: { operation: unknown }[];
  };
};

// ── Measurement types ──
export type MeasurementPoint = { x: number; y: number; z: number };

export type Measurement = {
  id: string;
  type: 'distance' | 'angle' | 'area';
  points: MeasurementPoint[];
  value: number;
  label: string;
  color: string;
  created_at: number;
};

export type ProjectSummary = {
  name: string;
  path: string;
  size: number;
  modified: number | null;
};

/** Motion joint list item (ASSEMBLY §9). */
export type JointItem = {
  id: string;
  name: string;
  kind: string;
  from_object_id: string;
  to_object_id: string;
  origin?: [number, number, number] | null;
  axis?: [number, number, number] | null;
  limit_min?: number | null;
  limit_max?: number | null;
};

/** Grouped door/window schedule row (DATA_MODEL opening schedule). */
export type OpeningsScheduleRow = {
  object_type: 'door' | 'window';
  width_mm: number;
  height_mm: number;
  leaf_count?: number | null;
  with_sill?: boolean | null;
  with_lintel?: boolean | null;
  frame_type?: string | null;
  count: number;
};

/** Editable project metadata (DATA_INTEGRITY §2.3, UI-DAT-07). */
export type ProjectMeta = {
  name: string;
  description: string;
  author: string;
  /** Display units only; stored geometry stays metric. */
  unit_system: string;
  mode_version: number;
  created_at?: number | null;
  modified_at?: number | null;
};

export type BomRow = {
  object_id: string;
  name: string;
  object_type: string;
  kind: 'component' | 'part' | 'proxy';
  component_def_id?: string | null;
  series_id?: string | null;
  bbox_volume_m3?: number | null;
};

export type BomResponse = {
  root_id: string;
  root_object_type: string;
  rows: BomRow[];
  totals: { row_count: number; total_volume_m3: number };
};

export type CurrentProject = {
  open: boolean;
  name: string;
  path: string | null;
};
