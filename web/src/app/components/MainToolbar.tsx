import { useEffect, useRef, useState } from 'react';
import { api } from '../../shared/api';
import type { OmniObject, TreeNode } from '../../shared/types';
import { useNavigate } from 'react-router-dom';
import { IcoRefresh, IcoAdd, IcoAssy, IcoDelete, IcoUndo, IcoRedo, IcoImport, IcoExport, IcoSketch, IcoCatalog, IcoSeries, IcoGrid } from './Icons';
import { useToastApi } from '../../shared/toast';
import { useAppStore } from '../store/appStore';
import { CreatePanel } from './create/CreatePanel';
import { AssemblyPanel } from './create/AssemblyPanel';
import type { ObjectType, GeometryType, CreationForm } from '../../shared/types/creation';
import { createInitialForm, validateForm, type ValidationResult } from '../../shared/types/creation';
import { CATALOG_TYPES, GENRULE_TYPES, LAYERED_TYPES, LINE_GENRULE_TYPES, LINEAR_TYPES, SURFACE_TYPES, PICKING_TYPES, DOMAIN_GROUPS, defaultParamsForDef, ensureCatalogLoaded, getCatalogSnapshot, resolveDef } from '../../shared/catalog';

export type TransformMode = 'translate' | 'rotate' | 'scale' | null;
export type CreateFormState = CreationForm;

type MainToolbarProps = {
  selectedIds: readonly string[];
  canUndo: boolean;
  canRedo: boolean;
  onRefresh: () => void;
  onDelete: () => void;
  onCreated: () => void;
  onImported: () => void;
  onEditComponent?: (componentId?: string) => void;
  selectedParentId?: string;
  tree?: TreeNode[];
};

export function MainToolbar({
  selectedIds, canUndo, canRedo,
  onRefresh, onDelete,
  onCreated, onImported, onEditComponent,
  selectedParentId, tree,
}: MainToolbarProps) {
  const navigate = useNavigate();
  const toast = useToastApi();
  const fileRef = useRef<HTMLInputElement | null>(null);
  const [assemblyOpen, setAssemblyOpen] = useState(false);
  const [activeDomain, setActiveDomain] = useState<string | null>(null);
  const [formErrors, setFormErrors] = useState<Record<string, string>>({});

  const catalogOpen = useAppStore((s) => s.catalogPanelOpen);
  const setCatalogOpen = useAppStore((s) => s.setCatalogPanelOpen);
  const creationPhase = useAppStore((s) => s.creationPhase);
  const creationObjectType = useAppStore((s) => s.creationObjectType);
  const creationForm = useAppStore((s) => s.creationForm);
  const creationError = useAppStore((s) => s.creationError);
  const creationDispatch = useAppStore((s) => s.creationDispatch);
  const creatingExtra = useAppStore((s) => s.creatingExtra);
  const setCreatingExtra = useAppStore((s) => s.setCreatingExtra);

  const showCreatePanel = (creationPhase === 'configuring' || creationPhase === 'creating' || creationPhase === 'error');

  const lastPickingModeRef = useRef<string | null>(null);
  const lastPickingPointsRef = useRef<{ x: number; y: number; z: number }[]>([]);

  // Merge external extra props (IFC/desc from Properties panel) into form
  const extraRef = useRef(creatingExtra);
  useEffect(() => {
    if (creatingExtra && showCreatePanel &&
        (creatingExtra.ifcClass !== extraRef.current?.ifcClass ||
         creatingExtra.desc !== extraRef.current?.desc) && creationForm) {
      creationDispatch({ type: 'updateForm', partial: { ifcClass: creatingExtra.ifcClass, desc: creatingExtra.desc } });
    }
    extraRef.current = creatingExtra;
  }, [creatingExtra, showCreatePanel, creationForm, creationDispatch]);

  // Watch for 'creating' phase and execute the API call
  useEffect(() => {
    if (creationPhase !== 'creating' || !creationForm) return;
    const form = creationForm;
    const doCreate = async () => {
      try {
        await createObjectWith(form);
        creationDispatch({ type: 'finishCreation' });
        onCreated();
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        creationDispatch({ type: 'failCreation', error: msg });
        toast.error(`创建失败: ${msg}`);
      }
    };
    void doCreate();
  }, [creationPhase]);

  async function createObjectWith(form: CreationForm) {
    const objectType = form.objectType;
    const name = form.name || `${objectType}-${Date.now()}`;

    let sx = 0, sy = 0, sz = 0, ex = 0, ey = 0, ez = 0;
    let length = 5;
    if (LINEAR_TYPES.has(objectType) || SURFACE_TYPES.has(objectType)) {
      sx = parseFloat(form.startX) || 0;
      sy = parseFloat(form.startY) || 0;
      sz = parseFloat(form.startZ) || 0;
      ex = parseFloat(form.endX) || 0;
      ey = parseFloat(form.endY) || 0;
      ez = parseFloat(form.endZ) || 0;
      length = Math.sqrt((ex-sx)**2 + (ey-sy)**2 + (ez-sz)**2) || 5;
    }

    const boundaryPoints = (form.boundary_points || []).map((p) => [p.x, p.y, p.z] as [number, number, number]);
    const useBoundary = boundaryPoints.length >= 1;

    let obj: OmniObject;

    const parentId = form.parentId || undefined;
    const seriesId = form.seriesId || undefined;

    // Ensure the catalog is loaded so we can resolve the selected ComponentDef.
    await ensureCatalogLoaded().catch(() => {});
    const catalogSnapshot = getCatalogSnapshot();

    // ── Layered surface components (wall/slab/roof/floor) → layered genrule ──
    // Units: the backend layer_extrusion + other genrules work in METERS, and the
    // pick points / form length-height-width are in meters. Layer thicknesses from
    // the LayerEditor are in mm (e.g. 200 = 200mm), so convert mm → m here.
    if (LAYERED_TYPES.has(objectType) && form.layers && form.layers.length > 0) {
      const layers = form.layers.map((l) => ({
        name: l.name || 'layer',
        thickness: ((Number(l.thickness) || 0) / 1000),
        material_id: l.material_id || undefined,
        function: l.function || 'structural',
      }));
      obj = await api.genruleCreate({
        genrule: 'layered',
        name,
        mode: objectType as 'wall' | 'slab' | 'roof' | 'floor',
        length: length,
        width: parseFloat(form.width) || 1,
        height: parseFloat(form.height) || 3,
        layers,
        parent_id: parentId,
        series_id: seriesId || undefined,
        start_x: sx, start_y: sy, start_z: sz,
        end_x: ex, end_y: ey, end_z: ez,
        ...(objectType === 'wall' && useBoundary
          ? { path: boundaryPoints }
          : (objectType !== 'wall' && useBoundary ? { boundary: boundaryPoints } : {})),
      });
    } else if (LINE_GENRULE_TYPES.has(objectType)) {
      obj = await api.genruleCreate({
        genrule: 'line',
        name,
        alignment: useBoundary ? boundaryPoints : (sx !== 0 || sy !== 0 || sz !== 0 || ex !== sx || ey !== sy ? [[sx, sy, sz], [ex, ey, ez]] : []),
        diameter: 0.05,
        parent_id: parentId,
      });
    } else if (GENRULE_TYPES.has(objectType)) {
      switch (objectType) {
        case 'door':
          obj = await api.genruleCreate({
            genrule: 'door',
            name,
            wall_id: form.targetWallId,
            opening_width: parseFloat(form.openingWidth) || 1.0,
            opening_height: parseFloat(form.openingHeight) || 2.1,
            position_x: parseFloat(form.posX) || 0,
            position_z: parseFloat(form.posZ) || 0,
            parent_id: parentId,
          });
          break;
        case 'window':
          obj = await api.genruleCreate({
            genrule: 'window',
            name,
            wall_id: form.targetWallId,
            opening_width: parseFloat(form.openingWidth) || 1.0,
            opening_height: parseFloat(form.openingHeight) || 2.1,
            position_x: parseFloat(form.posX) || 0,
            position_z: parseFloat(form.posZ) || 0,
            parent_id: parentId,
          });
          break;
        default:
          throw new Error(`Unknown genrule type: ${objectType}`);
      }
    } else if (CATALOG_TYPES.has(objectType)) {
      // Product path: resolve the selected ComponentDef and build params from
      // its ParamDefs (defaults + user overrides). Falls back to the first
      // published def for the type if none was explicitly selected.
      const def = resolveDef(form.catalogDefId, catalogSnapshot.catalog)
        ?? resolveDef(form.catalogType, catalogSnapshot.catalog)
        ?? catalogSnapshot.catalog.find((d) => d.ifc_class === form.ifcClass && d.published)
        ?? catalogSnapshot.catalog.find((d) => d.ifc_class === form.ifcClass);
      if (!def) {
        throw new Error(`No catalog component for type ${objectType}. Load a catalog first.`);
      }

      const params: Record<string, number> = { ...defaultParamsForDef(def), ...form.paramValues };

      // Derive position / length from picking for linear & surface types.
      const px = parseFloat(form.posX) || 0;
      const py = parseFloat(form.posY) || 0;
      const pz = parseFloat(form.posZ) || 0;

      let result: { component_id: string; object_type: string; name: string };
      if (LINEAR_TYPES.has(objectType)) {
        const mx = (sx + ex) / 2, my = (sy + ey) / 2, mz = (sz + ez) / 2;
        if ('length' in params) params.length = Math.round(length * 1000);
        result = await api.instantiateFromCatalog({
          component_def_id: def.id,
          instance_name: name,
          params,
          position: { x: mx, y: my, z: mz },
          rotation_z: Math.atan2(ey - sy, ex - sx),
          skip_series_gate: !seriesId,
        });
      } else {
        result = await api.instantiateFromCatalog({
          component_def_id: def.id,
          instance_name: name,
          params,
          position: { x: px, y: py, z: pz },
          skip_series_gate: !seriesId,
        });
      }
      obj = await api.object(result.component_id);
    } else {
      obj = await api.createObject({
        object_type: objectType,
        name,
        description: form.desc || '',
        geometry: { operations: [{ operation: createOperation(form) }] },
        parent_id: parentId,
      });
    }

    if (form.ifcClass || form.desc) {
      await api.updateProperties(obj.id, {
        name: obj.name,
        description: form.desc || '',
        ifc_class: form.ifcClass || '',
      }).catch(() => {});
    }

    if (LAYERED_TYPES.has(objectType) && form.layers) {
      await api.setProperty(obj.id, { name: 'layers', value: JSON.stringify(form.layers) }).catch(() => {});
    }
    if (LINEAR_TYPES.has(objectType) && !GENRULE_TYPES.has(objectType)) {
      const mx = (sx + ex) / 2, my = (sy + ey) / 2, mz = (sz + ez) / 2;
      if (mx !== 0 || my !== 0 || mz !== 0) {
        await api.transformObject(obj.id, { origin: null, translation: { x: mx, y: my, z: mz }, rotation: null, scale: { x: 1, y: 1, z: 1 } }).catch(() => {});
      }
    }
    if (LINEAR_TYPES.has(objectType)) {
      await api.setProperty(obj.id, { name: 'start_point', value: `${sx},${sy},${sz}` }).catch(() => {});
      await api.setProperty(obj.id, { name: 'end_point', value: `${ex},${ey},${ez}` }).catch(() => {});
    } else if (SURFACE_TYPES.has(objectType)) {
      if (objectType === 'wall' || objectType === 'slab') {
        await api.setProperty(obj.id, { name: 'height', value: String(parseFloat(form.height) || 3) }).catch(() => {});
        await api.setProperty(obj.id, { name: 'core_thickness', value: String(parseFloat(form.core) || 0.2) }).catch(() => {});
        await api.setProperty(obj.id, { name: 'finish_thickness', value: String(parseFloat(form.finish) || 0.02) }).catch(() => {});
      }
    } else {
      const px = parseFloat(form.posX) || 0, py = parseFloat(form.posY) || 0, pz = parseFloat(form.posZ) || 0;
      if (px !== 0 || py !== 0 || pz !== 0) {
        await api.transformObject(obj.id, { origin: null, translation: { x: px, y: py, z: pz }, rotation: null, scale: { x: 1, y: 1, z: 1 } }).catch(() => {});
      }
    }

    toast.success(`Created ${obj.name || 'object'}`);
  }

  function createOperation(f: CreationForm) {
    const p1 = parseFloat(f.p1) || 1;
    const p2 = parseFloat(f.p2) || 1;
    const p3 = parseFloat(f.p3) || 1;
    if (f.geometryType === 'box') return { Box: { length: p1, width: p2, height: p3 } };
    if (f.geometryType === 'cylinder') return { Cylinder: { radius: p1, height: p2, segments: parseInt(f.p3, 10) || 16 } };
    if (f.geometryType === 'sphere') return { Sphere: { radius: p1, segments: parseInt(f.p2, 10) || 16 } };
    return { Cone: { radius_bottom: p1, radius_top: p2, height: p3, segments: 16 } };
  }

  function handleDomainObjectClick(t: string) {
    if (PICKING_TYPES.has(t as ObjectType)) {
      creationDispatch({ type: 'startPicking', objectType: t as ObjectType });
    } else {
      const form = createInitialForm(t as ObjectType, []);
      creationDispatch({ type: 'completePicking', form });
    }
  }

  function handleSubmit() {
    if (!creationForm) return;
    const result = validateForm(creationForm);
    setFormErrors(result.errors);
    if (!result.valid) {
      toast.error('请修正表单错误');
      return;
    }
    creationDispatch({ type: 'submitCreation' });
  }

  function handleCancel() {
    creationDispatch({ type: 'cancelCreation' });
    setFormErrors({});
  }

  function handleResetToPicking() {
    creationDispatch({ type: 'resetToPicking' });
    setFormErrors({});
  }

  async function exportIfc() {
    const blob = await api.exportIfc();
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'project.ifc';
    anchor.click();
    URL.revokeObjectURL(url);
  }

  async function importIfc(file: File) {
    const normalize = confirm('Import IFC and move model to origin?\n\nOK = move to origin\nCancel = keep original coordinates');
    const replace = confirm('Replace current project with imported IFC?\n\nOK = replace\nCancel = merge into current project');
    const result = await api.importIfc(await file.text(), { normalize, mode: replace ? 'replace' : 'merge' });
    if (!result.success) {
      toast.error(`Import failed: ${result.message}`);
      return;
    }
    toast.success(`Imported ${result.imported} objects`);
    onImported();
  }

  async function doUndo() {
    try {
      await api.undo();
      toast.success('Undo successful');
      onCreated();
    } catch (err) {
      toast.error(`Undo failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function doRedo() {
    try {
      await api.redo();
      toast.success('Redo successful');
      onCreated();
    } catch (err) {
      toast.error(`Redo failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  return (
    <>
      <div className="main-toolbar">
        <button onClick={onRefresh} title="Refresh"><IcoRefresh /></button>
        <button className="accent" onClick={() => { setAssemblyOpen(false); handleDomainObjectClick('wall'); }} title="Create object"><IcoAdd /><span className="btn-label">Create</span></button>
        <button onClick={() => { setAssemblyOpen((v) => !v); }} title="Assembly (multi-part)"><IcoAssy /><span className="btn-label">Assy</span></button>
        <button className="danger" disabled={selectedIds.length === 0} onClick={onDelete} title="Delete selected"><IcoDelete /><span className="btn-label">Del</span></button>
        <span className="separator" />
        <button disabled={!canUndo} onClick={() => void doUndo()} title="Undo (Ctrl+Z)"><IcoUndo /></button>
        <button disabled={!canRedo} onClick={() => void doRedo()} title="Undo"><IcoRedo /></button>
        <span className="separator" />
        <button onClick={() => fileRef.current?.click()} title="Import IFC"><IcoImport /><span className="btn-label">Import</span></button>
        <button onClick={() => void exportIfc()} title="Export IFC"><IcoExport /><span className="btn-label">Export</span></button>
        <span className="separator" />
        <button className="accent" onClick={async () => {
          try {
            const comp = await api.createComponent({ name: `comp-${Date.now()}` });
            onEditComponent?.(comp.id);
          } catch (e) {
            toast.error(`创建 Component 失败: ${e instanceof Error ? e.message : String(e)}`);
          }
        }} title="创建 Component 并进入编辑器"><IcoSketch /><span className="btn-label">+ Component</span></button>
        <span className="separator" />
        <button onClick={() => setCatalogOpen((v) => !v)} title="Catalog browser"><IcoCatalog /><span className="btn-label">Catalog</span></button>
        <button onClick={() => navigate('/series-manager')} title="Series Manager"><IcoSeries /><span className="btn-label">Series</span></button>
        <button onClick={() => navigate('/library-manager')} title="Library Manager"><IcoCatalog /><span className="btn-label">Library</span></button>
        <span className="separator" />
        <button className={activeDomain === 'building' ? 'active' : ''} onClick={() => setActiveDomain(activeDomain === 'building' ? null : 'building')} title="建筑"><IcoGrid /><span className="btn-label">建筑</span></button>
        <button className={activeDomain === 'civil' ? 'active' : ''} onClick={() => setActiveDomain(activeDomain === 'civil' ? null : 'civil')} title="土木"><IcoGrid /><span className="btn-label">土木</span></button>
        <button className={activeDomain === 'plant' ? 'active' : ''} onClick={() => setActiveDomain(activeDomain === 'plant' ? null : 'plant')} title="工厂"><IcoGrid /><span className="btn-label">工厂</span></button>
        <button className={activeDomain === 'mechanical' ? 'active' : ''} onClick={() => setActiveDomain(activeDomain === 'mechanical' ? null : 'mechanical')} title="机械"><IcoGrid /><span className="btn-label">机械</span></button>
      </div>

      {activeDomain && (
        <div className="domain-toolbar">
          {DOMAIN_GROUPS[activeDomain]?.types.map((t) => (
            <button
              key={t}
              className={creationObjectType === t ? 'active' : ''}
              onClick={() => handleDomainObjectClick(t)}
              title={t}
            >
              {DOMAIN_GROUPS[activeDomain]?.labels[t] || t}
            </button>
          ))}
        </div>
      )}

      <input ref={fileRef} type="file" accept=".ifc" hidden
        onChange={(event) => {
          const file = event.currentTarget.files?.[0];
          if (file) void importIfc(file).catch((err) => toast.error(`Import failed: ${err instanceof Error ? err.message : String(err)}`));
          event.currentTarget.value = '';
        }}
      />

      {showCreatePanel && creationForm && (
        <CreatePanel
          form={creationForm}
          phase={creationPhase}
          errors={formErrors}
          error={creationError}
          tree={tree}
          selectedParentId={selectedParentId}
          onChange={(partial) => creationDispatch({ type: 'updateForm', partial })}
          onSubmit={handleSubmit}
          onCancel={handleCancel}
          onResetToPicking={handleResetToPicking}
        />
      )}

      {assemblyOpen && (
        <AssemblyPanel onCreated={onCreated} />
      )}
    </>
  );
}