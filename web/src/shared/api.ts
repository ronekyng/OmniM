import type { ComponentDef, ComponentResponse, ExecutionResult, FeatureItem, FeatureResponse, GenRuleRequest, ImportIfcResponse, MeshResponse, OmniObject, LoadSeriesResponse, SelectResult, Series, Sketch, SketchResponse, StatsResponse, TreeNode, UndoRedoResponse, UndoStatusResponse, UpdatePropertiesRequest, AmirTranslateRequest, AmirTranslateResponse, AmirChatRequest, AmirChatResponse, AmirDescribeRequest, AmirDescribeResponse, AmirExplainRequest, AmirExplainResponse, InstantiateResponse, PreviewRequest, CatalogPreviewMesh, ExportCatalogResponse, ImportCatalogResponse, SaveSeriesResponse, Material, ObjectPropertyDef, Measurement, MeasurementPoint } from './types';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, init);
  if (!response.ok) {
    let msg = `${response.status} ${response.statusText}`;
    try {
      const body = await response.text();
      if (body) msg = `${msg}: ${body}`;
    } catch {}
    throw new Error(msg);
  }
  return response.json() as Promise<T>;
}

export const api = {
  stats: () => request<StatsResponse>('/api/stats'),
  tree: () => request<TreeNode[]>('/api/tree'),
  object: (id: string) => request<OmniObject>(`/api/objects/${id}`),
  objectTypeProperties: (typeName: string) => request<ObjectPropertyDef[]>(`/api/object-types/${encodeURIComponent(typeName)}/properties`),
  updateProperties: (id: string, body: UpdatePropertiesRequest) => fetch(`/api/objects/${id}/properties`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  createObject: (body: unknown) => request<OmniObject>('/api/objects', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  transformObject: (id: string, body: unknown) => fetch(`/api/objects/${id}/transform`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  setProperty: (id: string, body: { name: string; value: string }) => fetch(`/api/objects/${id}/prop`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  mesh: () => request<MeshResponse>('/api/mesh', { cache: 'no-cache' }),
  deleteObject: (id: string) => fetch(`/api/objects/${id}`, { method: 'DELETE' }),
  updateObject: (id: string, body: { name?: string; description?: string }) => fetch(`/api/objects/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  importIfc: (content: string, options: { normalize: boolean; mode: 'merge' | 'replace' }) => request<ImportIfcResponse>(`/api/import/ifc?normalize=${options.normalize}&mode=${options.mode}`, {
    method: 'POST',
    headers: { 'Content-Type': 'text/plain' },
    body: content,
  }),
  exportIfc: async () => {
    const response = await fetch('/api/export/ifc');
    if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
    return response.blob();
  },
  listObjects: () => request<OmniObject[]>('/api/objects'),

  genruleCreate: (body: GenRuleRequest) => request<OmniObject>('/api/genrule', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  undo: () => request<UndoRedoResponse>('/api/undo', { method: 'POST' }),
  redo: () => request<UndoRedoResponse>('/api/redo', { method: 'POST' }),
  undoStatus: () => request<UndoStatusResponse>('/api/undo-status'),
  executeOmcom: (body: unknown) => request<ExecutionResult>('/api/omcom', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),

  // ── AMIR AI endpoints ──
  amirTranslate: (body: AmirTranslateRequest) => request<AmirTranslateResponse>('/api/amir/translate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  amirChat: (body: AmirChatRequest) => request<AmirChatResponse>('/api/amir/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  amirDescribe: (body: AmirDescribeRequest) => request<AmirDescribeResponse>('/api/amir/describe', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  amirExplain: (body: AmirExplainRequest) => request<AmirExplainResponse>('/api/amir/explain', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),

  setParent: (childId: string, newParentId: string | null) => {
    const command = {
      version: '1.0',
      id: `set-parent-${Date.now()}`,
      action: 30,
      target: { object: { object_id: childId } },
      operations: [{
        add_containment: { parent_id: newParentId ?? '' },
      }],
    };
    return api.executeOmcom(command);
  },

  duplicateObject: (sourceId: string, newName: string, parentId: string) => {
    const command = {
      version: '1.0',
      id: `copy-${Date.now()}`,
      action: 4,
      target: { object: { object_id: sourceId } },
      operations: [{
        copy_object: { source_object_id: sourceId, new_name: newName, parent_id: parentId },
      }],
    };
    return api.executeOmcom(command);
  },

  // ── Sketch endpoints ──
  listSketches: (componentId?: string) => {
    const query = componentId ? `?component_id=${encodeURIComponent(componentId)}` : '';
    return request<SketchResponse[]>(`/api/sketches${query}`);
  },
  getSketch: (id: string) => request<Sketch>(`/api/sketches/${id}`),
  createSketch: (body: { name: string; origin?: { x: number; y: number; z: number }; normal?: { x: number; y: number; z: number }; component_id?: string }) =>
    request<SketchResponse>('/api/sketches', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  deleteSketch: (id: string) => fetch(`/api/sketches/${id}`, { method: 'DELETE' }),
  updateSketch: (id: string, body: { origin?: { x: number; y: number; z: number }; name?: string }) =>
    request<Sketch>(`/api/sketches/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  addSketchEntity: (id: string, body: { entity_type: string; points: { x: number; y: number; z: number }[]; radius?: number; sides?: number; construction?: boolean }) =>
    request<{ index: number }>(`/api/sketches/${id}/entities`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  updateSketchEntity: (id: string, idx: number, body: { points?: { x: number; y: number; z: number }[]; radius?: number; sides?: number }) =>
    fetch(`/api/sketches/${id}/entities/${idx}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  addSketchConstraint: (id: string, body: { constraint_type: string; entity_refs: [number, number][]; value?: number }) =>
    request<{ index: number }>(`/api/sketches/${id}/constraints`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  solveSketch: (id: string) => request<{ success: boolean; dof: number; message: string }>(`/api/sketches/${id}/solve`, { method: 'POST' }),
  deleteSketchEntity: (id: string, idx: number) => fetch(`/api/sketches/${id}/entities/${idx}`, { method: 'DELETE' }),
  removeSketchConstraint: (id: string, idx: number) => fetch(`/api/sketches/${id}/constraints/${idx}`, { method: 'DELETE' }),
  extrudeSketch: (id: string, body: { depth: number; direction_x?: number; direction_y?: number; direction_z?: number }) =>
    request<FeatureResponse>(`/api/sketches/${id}/extrude`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  revolveSketch: (id: string, body: { angle: number; axis_origin_x?: number; axis_origin_y?: number; axis_origin_z?: number; axis_direction_x?: number; axis_direction_y?: number; axis_direction_z?: number }) =>
    request<FeatureResponse>(`/api/sketches/${id}/revolve`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),

  // ── Component endpoints ──
  listComponents: () => request<ComponentResponse[]>('/api/components'),
  createComponent: (body: { name: string; parent_id?: string }) =>
    request<ComponentResponse>('/api/components', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  getComponent: (id: string) => request<ComponentResponse>(`/api/components/${id}`),
  deleteComponent: (id: string) => fetch(`/api/components/${id}`, { method: 'DELETE' }),
  listFeatures: (componentId: string) => request<FeatureItem[]>(`/api/components/${componentId}/features`),
  addFeature: (componentId: string, body: {
    feature_type: string;
    sketch_id: string;
    input_sketch_ids?: string[];
    axis_sketch_id?: string;
    depth?: number;
    direction_x?: number;
    direction_y?: number;
    direction_z?: number;
    angle?: number;
    is_cut?: boolean;
  }) => request<FeatureItem>(`/api/components/${componentId}/features`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),
  removeFeature: (componentId: string, featureId: string) =>
    fetch(`/api/components/${componentId}/features/${featureId}`, { method: 'DELETE' }),
  updateFeature: (componentId: string, featureId: string, body: { sketch_id?: string; axis_sketch_id?: string; depth?: number; angle?: number; is_cut?: boolean; suppressed?: boolean }) =>
    fetch(`/api/components/${componentId}/features/${featureId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  rebuildComponent: (componentId: string) =>
    request<{ success: boolean }>(`/api/components/${componentId}/rebuild`, { method: 'POST' }),
  getSectionLines: (componentId: string, planeZ?: number) =>
    request<{ x1: number; y1: number; x2: number; y2: number }[]>(`/api/components/${componentId}/section?plane_z=${planeZ ?? 0}`),

  // ── Material endpoints ──
  listMaterials: () => request<Material[]>('/api/materials'),
  createMaterial: (body: { id: string; name: string; category?: number; density?: number; color?: { r: number; g: number; b: number; a: number } }) => fetch('/api/materials', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  }),

  // ── Measurement endpoints ──
  listMeasurements: () => request<Measurement[]>('/api/measurements'),
  createMeasurement: (body: { type: string; points: MeasurementPoint[]; value: number; label?: string; color?: string }) =>
    request<Measurement>('/api/measurements', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  deleteMeasurement: (id: string) => fetch(`/api/measurements/${id}`, { method: 'DELETE' }),

  // ── Catalog endpoints ──
  listCatalog: () => request<ComponentDef[]>('/api/catalog'),
  createComponentDef: (body: { name: string; category: string; description?: string; ifc_class?: string; standard_reference?: string; tags?: string[]; domain?: string }) =>
    request<ComponentDef>('/api/catalog', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  getComponentDef: (id: string) => request<ComponentDef>(`/api/catalog/${id}`),
  deleteComponentDef: (id: string) => fetch(`/api/catalog/${id}`, { method: 'DELETE' }),
  addComponentParam: (id: string, body: { name: string; param_type?: number; default_value?: number; min_value?: number; max_value?: number; step?: number; unit?: string }) =>
    fetch(`/api/catalog/${id}/params`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  addConnector: (id: string, body: { connector_id: string; connector_name: string; position_x_expr?: string; position_y_expr?: string; position_z_expr?: string; direction_x_expr?: string; direction_y_expr?: string; direction_z_expr?: string; bore_expr?: string; connection_type_code?: string; role?: number }) =>
    fetch(`/api/catalog/${id}/connectors`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  publishComponent: (id: string) => fetch(`/api/catalog/${id}/publish`, { method: 'POST' }),
  updateComponentDef: (id: string, body: { name?: string; category?: string; description?: string; ifc_class?: string; standard_reference?: string; tags?: string[]; domain?: string; allowed_materials?: string[]; default_material?: string }) =>
    fetch(`/api/catalog/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  updateComponentParam: (id: string, idx: number, body: { name?: string; default_value?: number; min_value?: number; max_value?: number; step?: number; unit?: string }) =>
    fetch(`/api/catalog/${id}/params/${idx}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  deleteComponentParam: (id: string, idx: number) =>
    fetch(`/api/catalog/${id}/params/${idx}`, { method: 'DELETE' }),
  assignFeatureTree: (id: string, featureTreeId: string) =>
    fetch(`/api/catalog/${id}/feature-tree`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ feature_tree_id: featureTreeId }),
    }),
  exportCatalog: () => request<ExportCatalogResponse>('/api/catalog/export', { method: 'POST' }),
  importCatalog: (path: string) => request<ImportCatalogResponse>('/api/catalog/import', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path }),
  }),
  instantiateFromCatalog: (body: { component_def_id: string; instance_name: string; params: Record<string, number>; position?: { x: number; y: number; z: number }; rotation_z?: number; skip_series_gate?: boolean }) =>
    request<InstantiateResponse>('/api/catalog/instantiate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  previewCatalogComponent: (body: PreviewRequest) =>
    request<CatalogPreviewMesh>('/api/catalog/preview', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),

  // ── Series endpoints ──
  listSeries: () => request<Series[]>('/api/series'),
  createSeries: (body: { name: string; description?: string; domain?: string; standard_reference?: string }) =>
    request<Series>('/api/series', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  getSeries: (id: string) => request<Series>(`/api/series/${id}`),
  updateSeries: (id: string, body: { name?: string; description?: string; domain?: string; standard_reference?: string; applicable_phases?: string[] }) =>
    fetch(`/api/series/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  saveSeries: () => request<SaveSeriesResponse>('/api/series/save', { method: 'POST' }),
  loadSeries: (path: string, source?: string) => request<LoadSeriesResponse>('/api/series/load', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, source: source || 'standalone' }),
  }),
  deleteSeries: (id: string) => fetch(`/api/series/${id}`, { method: 'DELETE' }),
  addSelector: (seriesId: string, body: { selector_id: string; question: string }) =>
    fetch(`/api/series/${seriesId}/selectors`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  addAnswer: (seriesId: string, body: { selector_id: string; answer_id: string; answer_value: string; is_numeric_range?: boolean; range_min?: number; range_max?: number }) =>
    fetch(`/api/series/${seriesId}/answers`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  addSpecComponent: (seriesId: string, body: { selector_id: string; answer_id: string; component_def_id: string; material?: string; connection_type_code?: string }) =>
    fetch(`/api/series/${seriesId}/spec`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  addParamConstraint: (seriesId: string, body: { component_def_id: string; param_name: string; default_value?: number; min_value?: number; max_value?: number; step?: number; fixed_value?: string }) =>
    fetch(`/api/series/${seriesId}/constraints`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  selectFromSeries: (seriesId: string, answers: string[]) =>
    request<SelectResult>(`/api/series/${seriesId}/select`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ answers }),
    }),
  updateParamConstraint: (seriesId: string, idx: number, body: { component_def_id: string; param_name?: string; default_value?: number; min_value?: number; max_value?: number; step?: number }) =>
    fetch(`/api/series/${seriesId}/constraints/${idx}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    }),
  deleteParamConstraint: (seriesId: string, idx: number, componentDefId: string) =>
    fetch(`/api/series/${seriesId}/constraints/${idx}`, {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ component_def_id: componentDefId }),
    }),
};
