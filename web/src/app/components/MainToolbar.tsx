import { useEffect, useRef, useState } from 'react';
import { api } from '../../shared/api';
import type { OmniObject, TreeNode } from '../../shared/types';
import { useNavigate } from 'react-router-dom';
import { IcoRefresh, IcoAdd, IcoAssy, IcoDelete, IcoUndo, IcoRedo, IcoImport, IcoExport, IcoSketch, IcoCatalog, IcoSeries, IcoGrid } from './Icons';
import { useToastApi } from '../../shared/toast';
import { useConfirm } from './ConfirmDialog';
import { nextAutoName } from '../../shared/naming';
import { useAppStore } from '../store/appStore';
import { CreatePanel } from './create/CreatePanel';
import { AssemblyPanel } from './create/AssemblyPanel';
import type { ObjectType, GeometryType, CreationForm } from '../../shared/types/creation';
import { createInitialForm, validateForm, totalPolylineLength, type ValidationResult } from '../../shared/types/creation';
import { CATALOG_TYPES, GENRULE_TYPES, LAYERED_TYPES, LINE_GENRULE_TYPES, LINEAR_TYPES, SURFACE_TYPES, PICKING_TYPES, DATUM_TYPES, DOMAIN_GROUPS, defaultParamsForDef, ensureCatalogLoaded, getCatalogSnapshot, resolveDef } from '../../shared/catalog';

function flattenTreeNames(nodes: TreeNode[]): string[] {
  const names: string[] = [];
  const walk = (list: TreeNode[]) => {
    for (const node of list) {
      names.push(node.name);
      walk(node.children);
    }
  };
  walk(nodes);
  return names;
}

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
  const confirm = useConfirm();
  const fileRef = useRef<HTMLInputElement | null>(null);
  const [assemblyOpen, setAssemblyOpen] = useState(false);
  const [activeDomain, setActiveDomain] = useState<string | null>(null);
  // Ribbon contract (IA_SHELL §3.3): start scoped to the building discipline
  // so the contextual command row is real from the first second.
  useEffect(() => { setActiveDomain((d) => d ?? 'building'); }, []);
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
    const name = form.name || nextAutoName(objectType, flattenTreeNames(tree ?? []));

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

    // Helper for the post-create field writes below: they used to be wrapped
    // in `.catch(() => {})`, which ate HTTP 500s and read as "saved
    // successfully" — surface them.
    const warnField = (err: unknown) =>
      toast.error(`附加字段保存失败: ${err instanceof Error ? err.message : String(err)}`);

    // Ensure the catalog is loaded so we can resolve the selected ComponentDef.
    await ensureCatalogLoaded().catch(warnField);
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
        // P8-S3: level-driven vertical extent (both-or-neither; server validates)
        ...(form.bottomLevelId && form.topLevelId
          ? { bottom_level_id: form.bottomLevelId, top_level_id: form.topLevelId }
          : {}),
        start_x: sx, start_y: sy, start_z: sz,
        end_x: ex, end_y: ey, end_z: ez,
        ...(objectType === 'wall' && useBoundary
          ? { path: boundaryPoints }
          : (objectType !== 'wall' && useBoundary ? { boundary: boundaryPoints } : {})),
      });
    } else if (objectType === 'road' && boundaryPoints.length >= 2) {
      // Road with picked alignment: server GenRuleRequest::Road accepts an
      // N-point path. With >= 2 picks we always send alignment (kernel bridge
      // task provides server support); straight 2-point paths degrade naturally.
      const alignmentLength = totalPolylineLength(form.boundary_points);
      obj = await api.genruleCreate({
        genrule: 'road',
        name,
        alignment: boundaryPoints,
        length: alignmentLength || length,
        width: parseFloat(form.width) || 7,
        thickness: parseFloat(form.thickness) || 0.3,
        parent_id: parentId,
        series_id: seriesId,
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
        case 'opening':
          obj = await api.genruleCreate({
            genrule: 'opening',
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
      const hasSpan =
        LINEAR_TYPES.has(objectType) &&
        length > 1e-9 &&
        (ex !== sx || ey !== sy || ez !== sz);
      if (hasSpan) {
        // Linear catalog components extrude along local +Z (their template
        // convention). Lay that axis along the picked segment and anchor the
        // part at the start point — a Z-yaw alone leaves beams vertical.
        if ('length' in params) params.length = Math.round(length * 1000);
        const rotation = quaternionFromZTo(ex - sx, ey - sy, ez - sz);
        result = await api.instantiateFromCatalog({
          component_def_id: def.id,
          instance_name: name,
          params,
          position: { x: sx, y: sy, z: sz },
          ...(rotation ? { rotation } : {}),
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
        // Datum/container types own no geometry (a storey is an elevation,
        // not a 5×0.3×3 box). The server also strips it defensively.
        ...(DATUM_TYPES.has(objectType)
          ? {}
          : { geometry: { operations: [{ operation: createOperation(form) }] } }),
        parent_id: parentId,
        // P8-S1: datum elevation on storey creation (m; may be negative).
        ...(objectType === 'storey' ? { elevation: parseFloat(form.elevation) || 0 } : {}),
        // P8-S3: level-driven vertical extent (both-or-neither).
        ...(form.bottomLevelId && form.topLevelId
          ? { bottom_level_id: form.bottomLevelId, top_level_id: form.topLevelId }
          : {}),
      });
    }

    if (form.ifcClass || form.desc) {
      await api.updateProperties(obj.id, {
        name: obj.name,
        description: form.desc || '',
        ifc_class: form.ifcClass || '',
      }).catch(warnField);
    }

    if (LAYERED_TYPES.has(objectType) && form.layers) {
      await api.setProperty(obj.id, { name: 'layers', value: JSON.stringify(form.layers) }).catch(warnField);
    }
    // Catalog instances were already positioned by the instantiate call —
    // transforming them again by the midpoint would double the offset.
    if (LINEAR_TYPES.has(objectType) && !GENRULE_TYPES.has(objectType) && !CATALOG_TYPES.has(objectType)) {
      const mx = (sx + ex) / 2, my = (sy + ey) / 2, mz = (sz + ez) / 2;
      if (mx !== 0 || my !== 0 || mz !== 0) {
        await api.transformObject(obj.id, { origin: null, translation: { x: mx, y: my, z: mz }, rotation: null, scale: { x: 1, y: 1, z: 1 } }).catch(warnField);
      }
    }
    if (LINEAR_TYPES.has(objectType)) {
      await api.setProperty(obj.id, { name: 'start_point', value: `${sx},${sy},${sz}` }).catch(warnField);
      await api.setProperty(obj.id, { name: 'end_point', value: `${ex},${ey},${ez}` }).catch(warnField);
    } else if (SURFACE_TYPES.has(objectType)) {
      if (objectType === 'wall' || objectType === 'slab') {
        await api.setProperty(obj.id, { name: 'height', value: String(parseFloat(form.height) || 3) }).catch(warnField);
        await api.setProperty(obj.id, { name: 'core_thickness', value: String(parseFloat(form.core) || 0.2) }).catch(warnField);
        await api.setProperty(obj.id, { name: 'finish_thickness', value: String(parseFloat(form.finish) || 0.02) }).catch(warnField);
      }
    } else if (!DATUM_TYPES.has(objectType)) {
      const px = parseFloat(form.posX) || 0, py = parseFloat(form.posY) || 0, pz = parseFloat(form.posZ) || 0;
      if (px !== 0 || py !== 0 || pz !== 0) {
        await api.transformObject(obj.id, { origin: null, translation: { x: px, y: py, z: pz }, rotation: null, scale: { x: 1, y: 1, z: 1 } }).catch(warnField);
      }
    }

    toast.success(`Created ${obj.name || 'object'}`);
  }

  /** Quaternion rotating local +Z onto `(dx, dy, dz)` (no scaling).
   *  Used to lay +Z-extruded catalog beams/pipes along a picked segment. */
  function quaternionFromZTo(dx: number, dy: number, dz: number) {
    const len = Math.sqrt(dx * dx + dy * dy + dz * dz);
    if (len < 1e-9) return null;
    const nx = dx / len, ny = dy / len, nz = dz / len;
    if (nz > 1 - 1e-9) return null;
    if (nz < -1 + 1e-9) return { x: 1, y: 0, z: 0, w: 0 };
    const sx = -ny, sy = nx;
    const sinTheta = Math.hypot(sx, sy);
    const half = Math.acos(Math.max(-1, Math.min(1, nz))) / 2;
    const k = Math.sin(half) / sinTheta;
    return { x: sx * k, y: sy * k, z: 0, w: Math.cos(half) };
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
    const normalize = await confirm({
      title: 'Move imported model to origin?',
      message: 'Cancel keeps the original IFC coordinates.',
      confirmLabel: 'Move to origin',
      cancelLabel: 'Keep coordinates',
    });
    const replace = await confirm({
      title: 'Replace the current project?',
      message: 'Cancel merges the imported objects into the open project.',
      confirmLabel: 'Replace',
      cancelLabel: 'Merge',
      danger: true,
    });
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
        <button onClick={() => { setAssemblyOpen((v) => !v); }} title="Assembly (multi-part)"><IcoAssy /><span className="btn-label">Assembly</span></button>
        <button className="danger" disabled={selectedIds.length === 0} onClick={onDelete} title="Delete selected"><IcoDelete /><span className="btn-label">Delete</span></button>
        <span className="separator" />
        <button disabled={!canUndo} onClick={() => void doUndo()} title="Undo (Ctrl+Z)"><IcoUndo /></button>
        <button disabled={!canRedo} onClick={() => void doRedo()} title="Undo"><IcoRedo /></button>
        <span className="separator" />
        <button onClick={() => fileRef.current?.click()} title="Import IFC"><IcoImport /><span className="btn-label">Import</span></button>
        <button onClick={() => void exportIfc()} title="Export IFC"><IcoExport /><span className="btn-label">Export</span></button>
        <span className="separator" />
        <button className="accent component-action" onClick={async () => {
          try {
            const names = (await api.listObjects()).map((o) => o.name);
            const comp = await api.createComponent({ name: nextAutoName('Component', names) });
            onEditComponent?.(comp.id);
          } catch (e) {
            toast.error(`创建 Component 失败: ${e instanceof Error ? e.message : String(e)}`);
          }
        }} title="创建 Component 并进入编辑器"><IcoSketch /><span className="btn-label">Component</span></button>
        <span className="separator" />
        <button onClick={() => setCatalogOpen((v) => !v)} title="Catalog browser"><IcoCatalog /><span className="btn-label">Catalog</span></button>
        <button onClick={() => navigate('/series-manager')} title="Series Manager"><IcoSeries /><span className="btn-label">Series</span></button>
        <button onClick={() => navigate('/library-manager')} title="Library Manager"><IcoCatalog /><span className="btn-label">Library</span></button>
        <span className="separator" />
        <button className={`domain-switch ${activeDomain === 'building' ? 'active' : ''}`} onClick={() => setActiveDomain('building')} title="建筑" aria-pressed={activeDomain === 'building'}><IcoGrid /><span className="btn-label">建筑</span></button>
        <button className={`domain-switch ${activeDomain === 'civil' ? 'active' : ''}`} onClick={() => setActiveDomain('civil')} title="土木" aria-pressed={activeDomain === 'civil'}><IcoGrid /><span className="btn-label">土木</span></button>
        <button className={`domain-switch ${activeDomain === 'plant' ? 'active' : ''}`} onClick={() => setActiveDomain('plant')} title="工厂" aria-pressed={activeDomain === 'plant'}><IcoGrid /><span className="btn-label">工厂</span></button>
        <button className={`domain-switch ${activeDomain === 'mechanical' ? 'active' : ''}`} onClick={() => setActiveDomain('mechanical')} title="机械" aria-pressed={activeDomain === 'mechanical'}><IcoGrid /><span className="btn-label">机械</span></button>
      </div>

      {activeDomain && (
        <div className="domain-toolbar" role="toolbar" aria-label={`${DOMAIN_GROUPS[activeDomain].label} 命令组`}>
          <span className="ribbon-group-label">创建</span>
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
          {creationObjectType && (
            <span className="domain-tool-indicator">当前：{DOMAIN_GROUPS[activeDomain]?.labels[creationObjectType] || creationObjectType} · Esc 取消</span>
          )}
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
        <AssemblyPanel onCreated={onCreated} tree={tree ?? []} />
      )}
    </>
  );
}