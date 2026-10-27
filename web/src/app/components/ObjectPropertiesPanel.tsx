import { useCallback, useEffect, useRef, useState } from 'react';
import { api } from '../../shared/api';
import type { MeshInfo, MeshMaterial } from './ThreeViewer';
import type { OmniObject, ObjectPropertyDef, TreeNode } from '../../shared/types';
import type { CreationForm, CreationPhase } from '../../shared/types/creation';
import { CATALOG_TYPES } from '../../shared/catalog';
import { CatalogParams } from './create/CatalogParams';
import { PropertyDefPanel } from './PropertyDefPanel';

type ObjectPropertiesPanelProps = {
  selectedIds?: readonly string[];
  tree?: TreeNode[];
  meshInfo?: MeshInfo | null;
  creatingForm?: CreationForm | null;
  creationPhase?: CreationPhase;
  onSaved: () => void;
  onClose: () => void;
  onExtraChange?: (extra: { ifcClass: string; desc: string }) => void;
  onChangeForm?: (partial: Partial<CreationForm>) => void;
  onSubmitCreate?: () => void;
  panelMode?: 'floating' | 'docked';
  onToggleMode?: () => void;
  onGetMaterial?: (id: string) => MeshMaterial | undefined;
  onSetMaterial?: (id: string, mat: MeshMaterial) => void;
};

const SURFACE_TYPES = new Set(['wall', 'slab', 'roof', 'floor']);
const LINEAR_TYPES = new Set(['road', 'pipe', 'beam', 'column', 'duct', 'bridge', 'tunnel']);
const POINT_TYPES = new Set(['point_cloud', 'terrain_model', 'feature_line', 'contour_line']);
const BODY_TYPES = new Set(['tank', 'pump', 'valve', 'gear', 'equipment', 'part', 'assembly']);

export function ObjectPropertiesPanel({ selectedIds, tree, meshInfo, creatingForm, creationPhase, onSaved, onClose, onExtraChange, onChangeForm, onSubmitCreate, panelMode = 'floating', onToggleMode, onGetMaterial, onSetMaterial }: ObjectPropertiesPanelProps) {
  const [object, setObject] = useState<OmniObject | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [propertyDefs, setPropertyDefs] = useState<ObjectPropertyDef[]>([]);
  const [localIfc, setLocalIfc] = useState('');
  const [localDesc, setLocalDesc] = useState('');
  const [material, setMaterial] = useState<MeshMaterial>({ color: '#cccccc', roughness: 0.5, metalness: 0.1 });

  const selectedCount = selectedIds?.length ?? 0;
  const primaryId = selectedCount === 1 ? selectedIds![0] : null;

  // Load object + property defs when single selection changes
  useEffect(() => {
    if (!primaryId) {
      setObject(null);
      setPropertyDefs([]);
      return;
    }
    let cancelled = false;
    async function load() {
      try {
        const [next, defs] = await Promise.all([
          api.object(primaryId!),
          api.objectTypeProperties((meshInfo?.type ?? '').toLowerCase()).catch(() => [] as ObjectPropertyDef[]),
        ]);
        if (cancelled) return;
        setObject(next);
        if (defs.length) {
          setPropertyDefs(defs);
        } else {
          // Fall back to the object's own ifc_class / inferred type
          const typeName = (next.ifc_class || objectTypeNameOf(next.object_type)).toLowerCase();
          const fallback = await api.objectTypeProperties(typeName).catch(() => [] as ObjectPropertyDef[]);
          if (!cancelled) setPropertyDefs(fallback);
        }
      } catch (err) {
        if (!cancelled) setError(err instanceof Error ? err.message : String(err));
      }
    }
    void load();

    // Reactive polling: refresh object data every 2s while selected
    const interval = setInterval(async () => {
      if (cancelled) return;
      try {
        const next = await api.object(primaryId);
        if (!cancelled) setObject(next);
      } catch { /* ignore polling errors */ }
    }, 2000);

    return () => { cancelled = true; clearInterval(interval); };
  }, [primaryId, meshInfo?.type]);

  useEffect(() => {
    if (creatingForm) {
      setLocalIfc(creatingForm.ifcClass);
      setLocalDesc(creatingForm.desc);
    }
  }, [creatingForm?.ifcClass, creatingForm?.desc]);

  useEffect(() => {
    if (primaryId && onGetMaterial) {
      const mat = onGetMaterial(primaryId);
      if (mat) setMaterial(mat);
    }
  }, [primaryId, onGetMaterial]);

  const notifyExtra = useCallback((ifc: string, desc: string) => {
    onExtraChange?.({ ifcClass: ifc, desc });
  }, [onExtraChange]);

  async function save(event: { preventDefault(): void; currentTarget: HTMLFormElement }) {
    event.preventDefault();
    if (!object) return;
    setSaving(true);
    setError(null);
    const form = new FormData(event.currentTarget);
    const response = await api.updateProperties(object.id, {
      name: String(form.get('name') ?? ''),
      description: String(form.get('description') ?? ''),
      ifc_class: String(form.get('ifc_class') ?? ''),
    });
    setSaving(false);
    if (!response.ok) {
      setError(`${response.status} ${response.statusText}`);
      return;
    }
    onSaved();
  }

  async function saveGeoParam(key: string, value: string) {
    if (!object) return;
    try {
      const res = await api.setProperty(object.id, { name: key, value });
      if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
      // Optimistic update — next poll will confirm
      onSaved();
    } catch (err) {
      setError(`保存属性 ${key} 失败: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  // Derive current property values from the live object
  const propertyValues: Record<string, string> = {};
  if (object) {
    for (const [name, pv] of Object.entries(object.properties)) {
      const v = pv?.string_value ?? pv?.double_value?.toString() ?? pv?.int_value?.toString() ?? '';
      if (v) propertyValues[name] = v;
    }
  }

  // === CREATION MODE ===
  if (creatingForm && !primaryId) {
    const ct = creatingForm.objectType;
    const isLinear = LINEAR_TYPES.has(ct);
    const isPoint = POINT_TYPES.has(ct);
    const isBody = BODY_TYPES.has(ct);
    const isCatalog = CATALOG_TYPES.has(ct);
    const isConfiguring = creationPhase === 'configuring';
    const isCreating = creationPhase === 'creating';

    const showPos = isPoint || isBody || (!isLinear && ct !== 'door' && ct !== 'window');
    const posStr = showPos ? `${creatingForm.posX}, ${creatingForm.posY}, ${creatingForm.posZ}` : '';

    const modeClass = panelMode === 'floating' ? 'properties-panel--floating' : 'properties-panel--docked';

    const handleChange = (p: Partial<CreationForm>) => onChangeForm?.(p);

    return (
      <section className={`properties-panel ${modeClass}`}>
        <div className="panel-header">
          <h3>新建图元属性</h3>
          <div className="panel-header-actions">
            <button className="mode-toggle" onClick={onToggleMode} title={panelMode === 'floating' ? '切换至停靠模式' : '切换至悬浮模式'}>
              {panelMode === 'floating' ? '⊞' : '⊟'}
            </button>
            <button className="close-btn" onClick={onClose} title="Close">✕</button>
          </div>
        </div>
        {error && <p className="error">{error}</p>}

        <div className="propsets-section">
          <h3>基本属性</h3>
          <div className="prop-row">
            <span className="prop-label">类型</span>
            <input className="prop-value" readOnly value={ct} />
          </div>
          <div className="prop-row">
            <span className="prop-label">名称</span>
            <input className="prop-value" value={creatingForm.name} placeholder={`${ct}-...`} onChange={(e) => handleChange({ name: e.target.value })} />
          </div>
          <div className="prop-row">
            <span className="prop-label">IFC Class</span>
            <input className="prop-value" value={localIfc} onChange={(e) => { setLocalIfc(e.target.value); notifyExtra(e.target.value, localDesc); }} />
          </div>
          <div className="prop-row">
            <span className="prop-label">描述</span>
            <textarea className="prop-value-text" value={localDesc} onChange={(e) => { setLocalDesc(e.target.value); notifyExtra(localIfc, e.target.value); }} rows={2} />
          </div>
        </div>

        {isCatalog && (
          <div className="propsets-section">
            <h3>构件参数</h3>
            <CatalogParams form={creatingForm} onChange={handleChange} />
          </div>
        )}

        {(isLinear || posStr) && (
          <div className="propsets-section">
            <h3>几何参数</h3>
            {isLinear && (
              <>
                <div className="prop-row">
                  <span className="prop-label">起点</span>
                  <input className="prop-value" readOnly value={startStrFrom(creatingForm)} />
                </div>
                <div className="prop-row">
                  <span className="prop-label">终点</span>
                  <input className="prop-value" readOnly value={endStrFrom(creatingForm)} />
                </div>
                <div className="prop-row">
                  <span className="prop-label">长度</span>
                  <div className="prop-row-inline">
                    <input className="prop-value" value={creatingForm.length} onChange={(e) => handleChange({ length: e.target.value })} />
                    <span className="prop-value">m</span>
                  </div>
                </div>
              </>
            )}
            {!isLinear && posStr && (
              <div className="prop-row">
                <span className="prop-label">位置</span>
                <input className="prop-value" value={posStr} readOnly />
              </div>
            )}
            </div>
        )}

        <div className="propsets-section hint" style={{ padding: '8px', fontSize: '12px', color: '#888' }}>
          {creationPhase === 'picking'
            ? '在 3D 视图中点击放置点，可在此实时调整类型和参数。'
            : isConfiguring
              ? '在面板中设置参数后点击"创建"。'
              : '在此设置 IFC 分类和描述。'}
        </div>

        <div className="form-actions">
          <button className="primary" disabled={isCreating} onClick={onSubmitCreate}>
            {isCreating ? '创建中...' : isConfiguring ? '创建' : '创建'}
          </button>
        </div>
      </section>
    );
  }

  // === MULTI-SELECTION MODE ===
  if (selectedCount > 1) {
    const modeClass = panelMode === 'floating' ? 'properties-panel--floating' : 'properties-panel--docked';
    return (
      <section className={`properties-panel ${modeClass}`}>
        <div className="panel-header">
          <h3>{selectedCount} objects selected</h3>
          <div className="panel-header-actions">
            <button className="mode-toggle" onClick={onToggleMode} title={panelMode === 'floating' ? '切换至停靠模式' : '切换至悬浮模式'}>
              {panelMode === 'floating' ? '⊞' : '⊟'}
            </button>
            <button className="close-btn" onClick={onClose} title="Close">✕</button>
          </div>
        </div>
        <div className="propsets-section">
          <p style={{ color: '#888', padding: '8px' }}>
            {selectedCount} objects are currently selected. Use Ctrl+click or box selection to modify the selection. Press Delete to remove selected objects.
          </p>
        </div>
      </section>
    );
  }

  // === NORMAL MODE: single object ===
  if (!object) return <section className="properties-panel">Loading...</section>;

  const bbox = meshInfo?.bbox || [[0, 0, 0], [0, 0, 0]];
  const size = [
    (bbox[1][0] - bbox[0][0]).toFixed(2),
    (bbox[1][1] - bbox[0][1]).toFixed(2),
    (bbox[1][2] - bbox[0][2]).toFixed(2),
  ];

  const typeName = meshInfo?.type || object.ifc_class || '';
  const objectTypeLower = (meshInfo?.type || '').toLowerCase();
  const isLinear = LINEAR_TYPES.has(objectTypeLower);

  const startPoint = propertyValues['start_point'] || '';
  const endPoint = propertyValues['end_point'] || '';

  const modeClass = panelMode === 'floating' ? 'properties-panel--floating' : 'properties-panel--docked';

  // Collapse the mesh-info group from the data-driven property panel
  const collapsedGroups = new Set<string>(['几何参数']);

  return (
    <section className={`properties-panel ${modeClass}`} key={object.id}>
      <div className="panel-header">
        <h3>{object.name || 'Properties'}</h3>
        <div className="panel-header-actions">
          <button className="mode-toggle" onClick={onToggleMode} title={panelMode === 'floating' ? '切换至停靠模式' : '切换至悬浮模式'}>
            {panelMode === 'floating' ? '⊞' : '⊟'}
          </button>
          <button className="close-btn" onClick={onClose} title="Close">✕</button>
        </div>
      </div>
      {error && <p className="error">{error}</p>}

      <form onSubmit={(event) => void save(event)}>
        <div className="prop-row"><span className="prop-label">Type</span><input className="prop-value" readOnly value={typeName} /></div>
        <div className="prop-row"><span className="prop-label">Name</span><input className="prop-value" name="name" key={`name-${object.id}`} defaultValue={object.name} /></div>
        <div className="prop-row"><span className="prop-label">IFC Class</span><input className="prop-value" name="ifc_class" key={`ifc-${object.id}`} defaultValue={object.ifc_class} /></div>
        {meshInfo && (
          <>
            <div className="prop-row"><span className="prop-label">Triangles</span><input className="prop-value" readOnly value={meshInfo.tcount} /></div>
            <div className="prop-row"><span className="prop-label">Vertices</span><input className="prop-value" readOnly value={meshInfo.vcount} /></div>
            <div className="prop-row"><span className="prop-label">Size (XYZ)</span><input className="prop-value" readOnly value={`${size[0]} × ${size[1]} × ${size[2]}`} /></div>
            <div className="prop-row"><span className="prop-label">BBox Min</span><input className="prop-value" readOnly value={`${bbox[0][0].toFixed(2)}, ${bbox[0][1].toFixed(2)}, ${bbox[0][2].toFixed(2)}`} /></div>
            <div className="prop-row"><span className="prop-label">BBox Max</span><input className="prop-value" readOnly value={`${bbox[1][0].toFixed(2)}, ${bbox[1][1].toFixed(2)}, ${bbox[1][2].toFixed(2)}`} /></div>
          </>
        )}
        {onGetMaterial && onSetMaterial && primaryId && (
          <div className="propsets-section">
            <h3>材质</h3>
            <div className="prop-row">
              <span className="prop-label">颜色</span>
              <input className="prop-value" type="color" value={material.color}
                onChange={(e) => {
                  const next = { ...material, color: e.target.value };
                  setMaterial(next);
                  onSetMaterial(primaryId, next);
                }}
              />
            </div>
            <div className="prop-row">
              <span className="prop-label">粗糙度</span>
              <input className="prop-value" type="range" min="0" max="1" step="0.01" value={material.roughness}
                onChange={(e) => {
                  const next = { ...material, roughness: parseFloat(e.target.value) };
                  setMaterial(next);
                  onSetMaterial(primaryId, next);
                }}
              />
              <span className="prop-value" style={{ width: 32, textAlign: 'right' }}>{material.roughness.toFixed(2)}</span>
            </div>
            <div className="prop-row">
              <span className="prop-label">金属度</span>
              <input className="prop-value" type="range" min="0" max="1" step="0.01" value={material.metalness}
                onChange={(e) => {
                  const next = { ...material, metalness: parseFloat(e.target.value) };
                  setMaterial(next);
                  onSetMaterial(primaryId, next);
                }}
              />
              <span className="prop-value" style={{ width: 32, textAlign: 'right' }}>{material.metalness.toFixed(2)}</span>
            </div>
          </div>
        )}
        <div className="prop-row">
          <span className="prop-label">Description</span>
          <textarea className="prop-value-text" name="description" key={`desc-${object.id}`} defaultValue={object.description} rows={2} />
        </div>
        <div className="form-actions">
          <button type="submit" disabled={saving}>{saving ? 'Saving...' : 'Save'}</button>
        </div>
      </form>

      {/* Data-driven property definitions — replaces the old TYPE_PARAMS system */}
      <PropertyDefPanel
        defs={propertyDefs}
        values={propertyValues}
        onChange={saveGeoParam}
        collapsedGroups={collapsedGroups}
      />

      {/* Linear geometry params (start/end points) — still rendered alongside the data-driven panel */}
      {isLinear && (startPoint || endPoint) && (
        <div className="propsets-section">
          <h3>线性几何参数</h3>
          {startPoint && (
            <ParamRow label="起点" value={startPoint} onChange={(v) => saveGeoParam('start_point', v)} />
          )}
          {endPoint && (
            <ParamRow label="终点" value={endPoint} onChange={(v) => saveGeoParam('end_point', v)} />
          )}
        </div>
      )}

      {object.catalog_ref && (
        <div className="propsets-section">
          <h3>目录引用</h3>
          <div className="prop-row"><span className="prop-label">目录构件</span><input className="prop-value" readOnly value={object.catalog_ref.component_def_id} /></div>
          {object.catalog_ref.series_id && (
            <div className="prop-row"><span className="prop-label">系列</span><input className="prop-value" readOnly value={object.catalog_ref.series_id} /></div>
          )}
          {object.catalog_ref.selector_path.length > 0 && (
            <div className="prop-row"><span className="prop-label">选择路径</span><input className="prop-value" readOnly value={object.catalog_ref.selector_path.join(' → ')} /></div>
          )}
          {Object.keys(object.catalog_ref.param_values).length > 0 && (
            <details className="propset" open>
              <summary className="propset-title">实例参数</summary>
              {Object.entries(object.catalog_ref.param_values).map(([k, v]) => (
                <div key={k} className="propset-prop">
                  <span className="propset-prop-name">{k}</span>
                  <span className="propset-prop-value">{v.toFixed(3)}</span>
                </div>
              ))}
            </details>
          )}
        </div>
      )}

      {object.property_sets.length > 0 && (
        <div className="propsets-section">
          <h3>IFC Property Sets ({object.property_sets.length})</h3>
          {object.property_sets.map((set) => (
            <details key={set.name} className="propset" open>
              <summary className="propset-title">{set.name}</summary>
              {set.properties.length === 0 && <div className="propset-empty">(no properties)</div>}
              {set.properties.map((prop) => (
                <div key={prop.name} className="propset-prop">
                  <span className="propset-prop-name">{prop.name}</span>
                  <span className="propset-prop-value">
                    {prop.value?.string_value ?? prop.value?.double_value?.toFixed(3) ?? prop.value?.int_value ?? String(prop.value?.bool_value ?? '')}
                  </span>
                </div>
              ))}
            </details>
          ))}
        </div>
      )}
    </section>
  );
}

function ParamRow({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => Promise<void> }) {
  const [saving, setSaving] = useState(false);
  const [local, setLocal] = useState(value);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => { setLocal(value); }, [value]);

  async function handleBlur() {
    if (local === value) return;
    setSaving(true);
    await onChange(local);
    setSaving(false);
  }

  return (
    <div className="prop-row">
      <span className="prop-label">{label}</span>
      <input
        ref={inputRef}
        className="prop-value"
        value={local}
        onChange={(e) => setLocal(e.target.value)}
        onBlur={() => void handleBlur()}
        onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
      />
      {saving && <span className="prop-saving">...</span>}
    </div>
  );
}

function startStrFrom(form: CreationForm): string {
  return `${form.startX}, ${form.startY}, ${form.startZ}`;
}

function endStrFrom(form: CreationForm): string {
  return `${form.endX}, ${form.endY}, ${form.endZ}`;
}

const OBJECT_TYPE_NAMES: Record<number, string> = {
  3: 'wall', 4: 'beam', 5: 'column', 6: 'slab', 7: 'door', 8: 'window',
  9: 'stair', 10: 'roof', 11: 'floor', 12: 'duct', 13: 'cable_tray',
  14: 'air_terminal', 17: 'road', 18: 'bridge', 19: 'tunnel',
  26: 'pipe_civil', 28: 'tank', 29: 'pump', 30: 'valve',
  34: 'pipe_plant', 40: 'part', 41: 'assembly', 45: 'gear',
};

function objectTypeNameOf(objectType: number): string {
  return OBJECT_TYPE_NAMES[objectType] ?? 'generic';
}