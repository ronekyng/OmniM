import { useEffect, useRef, useState } from 'react';
import type { CreationForm, CreationPhase, ObjectType, GeometryType } from '../../../shared/types/creation';
import { GENRULE_TYPES, CATALOG_TYPES, LINEAR_TYPES, LAYERED_TYPES, LINE_GENRULE_TYPES, SURFACE_TYPES, objectTypeLabel } from '../../../shared/catalog';
import { ParentSelector } from './ParentSelector';
import { SeriesSelector } from './SeriesSelector';
import { CatalogParams } from './CatalogParams';
import { LayerEditor } from './LayerEditor';
import type { TreeNode } from '../../../shared/types';
import { api } from '../../../shared/api';

type CreatePanelProps = {
  form: CreationForm;
  phase: CreationPhase;
  errors: Record<string, string>;
  error: string | null;
  tree?: TreeNode[];
  selectedParentId?: string;
  onChange: (partial: Partial<CreationForm>) => void;
  onSubmit: () => void;
  onCancel: () => void;
  onResetToPicking: () => void;
};

const OBJECT_TYPE_OPTIONS: { group: string; types: ObjectType[] }[] = [
  {
    group: '建筑',
    types: ['wall', 'slab', 'beam', 'column', 'door', 'window', 'stair', 'roof', 'building', 'storey', 'space', 'site'],
  },
  {
    group: '基础设施',
    types: ['road', 'bridge', 'tunnel', 'pipe', 'duct', 'feature_line', 'contour_line'],
  },
  {
    group: '设备',
    types: ['tank', 'pump', 'valve', 'motor', 'part', 'assembly'],
  },
];

export function CreatePanel({
  form, phase, errors, error,
  tree, selectedParentId,
  onChange, onSubmit, onCancel, onResetToPicking,
}: CreatePanelProps) {
  const [dirty, setDirty] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const formRef = useRef(form);
  formRef.current = form;

  useEffect(() => {
    if (!selectedParentId || form.parentId) return;
    if (!tree) return;
    const CONTAINER_TYPES = new Set(['Site', 'Building', 'BuildingStorey', 'Storey', 'Space', 'building', 'storey', 'site', 'space']);
    const isContainer = (nodes: TreeNode[]): boolean => {
      for (const n of nodes) {
        if (n.id === selectedParentId) return CONTAINER_TYPES.has(n.object_type) || CONTAINER_TYPES.has(n.ifc_class);
        if (n.children && isContainer(n.children)) return true;
      }
      return false;
    };
    if (isContainer(tree)) {
      onChange({ parentId: selectedParentId });
    }
  }, [selectedParentId, tree]);

  const handleChange = (p: Partial<CreationForm>) => {
    setDirty(true);
    onChange(p);
  };

  const handleObjectTypeChange = (ot: string) => {
    handleChange({ objectType: ot as ObjectType, catalogDefId: '', paramValues: {} });
  };

  const handleGeometryTypeChange = (geometryType: GeometryType) => {
    const defaults: Record<GeometryType, Pick<CreationForm, 'p1' | 'p2' | 'p3'>> = {
      box: { p1: '5', p2: '0.3', p3: '3' },
      cylinder: { p1: '0.5', p2: '4', p3: '16' },
      sphere: { p1: '2', p2: '16', p3: '' },
      cone: { p1: '1', p2: '0.5', p3: '3' },
    };
    handleChange({ geometryType, ...defaults[geometryType] });
  };

  const handleClose = () => {
    if (dirty) {
      setShowConfirm(true);
    } else {
      onCancel();
    }
  };

  const handleConfirmClose = () => {
    setShowConfirm(false);
    onCancel();
  };

  const isCreating = phase === 'creating';
  const isError = phase === 'error';
  const isConfiguring = phase === 'configuring';

  const buttonText = isCreating ? '创建中...' : isError ? '重试' : '创建';

  const paramLabels = (type: GeometryType): [string, string, string] => {
    if (type === 'cylinder') return ['半径', '高度', '段数'];
    if (type === 'sphere') return ['半径', '段数', ''];
    if (type === 'cone') return ['底半径', '顶半径', '高度'];
    return ['长', '宽', '高'];
  };

  const isPrimitive = !GENRULE_TYPES.has(form.objectType) && !CATALOG_TYPES.has(form.objectType) && !LINE_GENRULE_TYPES.has(form.objectType);
  const isLayeredSurface = LAYERED_TYPES.has(form.objectType);
  const isGenrule = GENRULE_TYPES.has(form.objectType);
  const isLinear = LINEAR_TYPES.has(form.objectType);
  /** Road / bridge / tunnel define a cross-section (width × thickness) directly. */
  const isCrossSectionLinear = LINEAR_TYPES.has(form.objectType) && (form.objectType === 'road' || form.objectType === 'bridge' || form.objectType === 'tunnel');

  return (
    <>
      <div className="create-panel">
        <div className="panel-header">
          <strong>创建图元</strong>
          <button className="panel-close" onClick={handleClose} title="Close">&times;</button>
        </div>

        {showConfirm && (
          <div className="confirm-overlay" style={{ padding: '8px', background: '#2a2d35', borderRadius: '6px', marginBottom: '8px' }}>
            <p style={{ fontSize: '12px', marginBottom: '6px' }}>表单已修改，确认关闭吗？</p>
            <button className="primary" style={{ marginRight: '6px', fontSize: '11px', padding: '2px 8px' }} onClick={handleConfirmClose}>确认关闭</button>
            <button style={{ fontSize: '11px', padding: '2px 8px' }} onClick={() => setShowConfirm(false)}>继续编辑</button>
          </div>
        )}

        {isError && error && (
          <div className="error-panel" style={{ padding: '8px', background: '#3d1f1f', borderRadius: '6px', marginBottom: '8px', color: '#ff6b6b' }}>
            <strong>创建失败：</strong>{error}
            <div style={{ marginTop: '6px' }}>
              <button className="primary" style={{ marginRight: '6px', fontSize: '11px', padding: '2px 8px' }} onClick={onResetToPicking}>重试选点</button>
              <button style={{ fontSize: '11px', padding: '2px 8px' }} onClick={onCancel}>取消</button>
            </div>
          </div>
        )}

        <div className="create-row">
          <select value={form.objectType} onChange={(e) => handleObjectTypeChange(e.target.value)}>
            {OBJECT_TYPE_OPTIONS.map((g) => (
              <optgroup key={g.group} label={g.group}>
                {g.types.map((t) => (
                  <option key={t} value={t}>{objectTypeLabel(t)} {t}</option>
                ))}
              </optgroup>
            ))}
          </select>
          <input placeholder="名称" value={form.name} onChange={(e) => handleChange({ name: e.target.value })} />
        </div>
        {errors.name && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.name}</small>}

        <ParentSelector
          tree={tree}
          value={form.parentId}
          onChange={(id) => handleChange({ parentId: id })}
        />

        {form.objectType === 'roof' && (
          <div className="create-row create-row--labeled">
            <span className="field-label">屋顶模式</span>
            <select
              value={form.roofMode}
              onChange={(e) => {
                const roofMode = e.target.value as 'footprint' | 'extrusion';
                handleChange({
                  roofMode,
                  pickMode: roofMode === 'extrusion' ? { mode: 'segment', minClicks: 2 } : { mode: 'polygon', minClicks: 3 },
                });
              }}
            >
              <option value="footprint">轮廓（多边形）</option>
              <option value="extrusion">坡屋面（段）</option>
            </select>
            <small className="hint">
              {form.roofMode === 'footprint' ? '沿轮廓选点，封闭后完成' : '指定屋面坡向段（两点）'}
            </small>
          </div>
        )}

        {form.objectType === 'column' && (
          <div className="create-row create-row--labeled">
            <span className="field-label">柱模式</span>
            <select
              value={form.columnMode}
              onChange={(e) => {
                const columnMode = e.target.value as 'place' | 'segment';
                handleChange({
                  columnMode,
                  pickMode: columnMode === 'segment' ? { mode: 'segment', minClicks: 2 } : { mode: 'place', minClicks: 1 },
                });
              }}
            >
              <option value="place">单点放置（Revit）</option>
              <option value="segment">两点（Tekla）</option>
            </select>
            {form.columnMode === 'place' && (
              <div className="create-row create-row--labeled">
                <span className="field-label">柱高</span>
                <input placeholder="m" value={form.columnHeight} onChange={(e) => handleChange({ columnHeight: e.target.value })} />
              </div>
            )}
          </div>
        )}

        {(isGenrule || CATALOG_TYPES.has(form.objectType)) && (
          <SeriesSelector
            value={form.seriesId}
            onChange={(id) => handleChange({ seriesId: id })}
            objectType={form.objectType}
          />
        )}

        {isGenrule && (
          <DoorWindowFields form={form} errors={errors} onChange={handleChange} />
        )}

        {isPrimitive && (
          <div className="create-row create-row--labeled">
            <select value={form.geometryType} onChange={(e) => handleGeometryTypeChange(e.target.value as GeometryType)}>
              <option value="box">Box 长方体</option>
              <option value="cylinder">Cylinder 圆柱</option>
              <option value="sphere">Sphere 球体</option>
              <option value="cone">Cone 圆锥</option>
            </select>
            <span className="field-label">{paramLabels(form.geometryType)[0]}</span>
            <input placeholder="" value={form.p1} onChange={(e) => handleChange({ p1: e.target.value })} />
            <span className="field-label">{paramLabels(form.geometryType)[1]}</span>
            <input placeholder="" value={form.p2} onChange={(e) => handleChange({ p2: e.target.value })} />
            {paramLabels(form.geometryType)[2] && (
              <>
                <span className="field-label">{paramLabels(form.geometryType)[2]}</span>
                <input placeholder="" value={form.p3} onChange={(e) => handleChange({ p3: e.target.value })} />
              </>
            )}
          </div>
        )}

        {CATALOG_TYPES.has(form.objectType) && (
          <CatalogParams form={form} onChange={handleChange} />
        )}

        {isLayeredSurface && (
          <LayerEditor
            value={form.layers}
            mode={form.objectType === 'wall' ? 'wall' : 'slab'}
            onChange={(layers) => handleChange({ layers, layersJson: JSON.stringify(layers) })}
          />
        )}

        {isCrossSectionLinear && (
          <div className="create-row create-row--labeled" style={{ marginBottom: '6px' }}>
            <span className="field-label">断面宽</span>
            <input value={form.width} onChange={(e) => handleChange({ width: e.target.value })} />
            <span className="field-label">断面高</span>
            <input value={form.thickness} onChange={(e) => handleChange({ thickness: e.target.value })} />
          </div>
        )}

        {isLinear ? (
          <>
            <div className="create-row create-row--labeled">
              <span className="field-label">起点</span>
              <input placeholder="X" value={form.startX} onChange={(e) => handleChange({ startX: e.target.value })} />
              <input placeholder="Y (高度)" value={form.startY} onChange={(e) => handleChange({ startY: e.target.value })} />
              <input placeholder="Z" value={form.startZ} onChange={(e) => handleChange({ startZ: e.target.value })} />
            </div>
            <div className="create-row create-row--labeled">
              <span className="field-label">终点</span>
              <input placeholder="X" value={form.endX} onChange={(e) => handleChange({ endX: e.target.value })} />
              <input placeholder="Y (高度)" value={form.endY} onChange={(e) => handleChange({ endY: e.target.value })} />
              <input placeholder="Z" value={form.endZ} onChange={(e) => handleChange({ endZ: e.target.value })} />
            </div>
          </>
        ) : (
          !isGenrule && (
            <div className="create-row create-row--labeled">
              <span className="field-label">位置</span>
              <input placeholder="X" value={form.posX} onChange={(e) => handleChange({ posX: e.target.value })} />
              <input placeholder="Y (高度)" value={form.posY} onChange={(e) => handleChange({ posY: e.target.value })} />
              <input placeholder="Z" value={form.posZ} onChange={(e) => handleChange({ posZ: e.target.value })} />
            </div>
          )
        )}

        <button
          className="primary"
          disabled={isCreating}
          onClick={onSubmit}
        >
          {buttonText} {objectTypeLabel(form.objectType)}
        </button>
      </div>
    </>
  );
}

function DoorWindowFields({ form, errors, onChange }: {
  form: CreationForm;
  errors: Record<string, string>;
  onChange: (p: Partial<CreationForm>) => void;
}) {
  const [walls, setWalls] = useState<{ id: string; name: string }[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    api.tree().then((tree) => {
      if (cancelled) return;
      const flat: { id: string; name: string }[] = [];
      function walk(nodes: typeof tree) {
        for (const n of nodes) {
          if (n.object_type === 'Wall') flat.push({ id: n.id, name: n.name });
          walk(n.children);
        }
      }
      walk(tree);
      setWalls(flat);
      setLoading(false);
    }).catch(() => { setLoading(false); });
    return () => { cancelled = true; };
  }, []);

  if (loading) return <div className="create-row"><span>加载墙体列表...</span></div>;
  if (walls.length === 0) return <div className="create-row"><span className="hint">⚠️ 请先创建一面墙体</span></div>;

  return (
    <>
      <div className="create-row">
        <select value={form.targetWallId} onChange={(e) => onChange({ targetWallId: e.target.value })}>
          <option value="">-- 选择目标墙体 --</option>
          {walls.map((w) => <option key={w.id} value={w.id}>{w.name}</option>)}
        </select>
      </div>
      {errors.targetWallId && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.targetWallId}</small>}
      <div className="create-row create-row--labeled">
        <span className="field-label">洞口宽</span>
        <input placeholder="m" value={form.openingWidth} onChange={(e) => onChange({ openingWidth: e.target.value })} />
        <span className="field-label">洞口高</span>
        <input placeholder="m" value={form.openingHeight} onChange={(e) => onChange({ openingHeight: e.target.value })} />
      </div>
      {errors.openingWidth && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.openingWidth}</small>}
      {errors.openingHeight && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.openingHeight}</small>}
      <div className="create-row create-row--labeled">
        <span className="field-label">水平位</span>
        <input placeholder="m" value={form.posX} onChange={(e) => onChange({ posX: e.target.value })} />
        <span className="field-label">{form.objectType === 'door' ? '离地高' : '窗台高'}</span>
        <input placeholder="m" value={form.posZ} onChange={(e) => onChange({ posZ: e.target.value })} />
      </div>
      <small className="hint">在选中的墙体上开{form.objectType === 'door' ? '门' : '窗'}洞</small>
    </>
  );
}
