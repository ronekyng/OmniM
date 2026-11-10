import { useEffect, useRef, useState } from 'react';
import type { CreationForm, CreationPhase, ObjectType, GeometryType } from '../../../shared/types/creation';
import { withGenesisForm, EXTENT_TYPES, storeyOptions, type StoreyOption } from '../../../shared/types/creation';
import { GENRULE_TYPES, CATALOG_TYPES, LINEAR_TYPES, LAYERED_TYPES, LINE_GENRULE_TYPES, SURFACE_TYPES, DATUM_TYPES, objectTypeLabel } from '../../../shared/catalog';
import { ParentSelector } from './ParentSelector';
import { SeriesSelector } from './SeriesSelector';
import { CatalogParams } from './CatalogParams';
import { LayerEditor } from './LayerEditor';
import { IcoWarning } from '../Icons';
import { useI18n } from '../../../shared/i18n';
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
    group: 'create.grp.architecture',
    types: ['wall', 'slab', 'beam', 'column', 'door', 'window', 'opening', 'stair', 'roof', 'building', 'storey', 'space', 'site'],
  },
  {
    group: 'create.grp.infrastructure',
    types: ['road', 'bridge', 'tunnel', 'pipe', 'duct', 'feature_line', 'contour_line'],
  },
  {
    group: 'create.grp.equipment',
    types: ['tank', 'pump', 'valve', 'motor', 'part', 'assembly'],
  },
];

export function CreatePanel({
  form, phase, errors, error,
  tree, selectedParentId,
  onChange, onSubmit, onCancel, onResetToPicking,
}: CreatePanelProps) {
    const { t } = useI18n();
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

  const buttonText = isCreating ? t('create.saving') : isError ? t('panel.domain.retry') : t('create.submit');

  const paramLabels = (type: GeometryType): [string, string, string] => {
    if (type === 'cylinder') return [t('create.p.radius'), t('create.p.height'), t('create.p.segments')];
    if (type === 'sphere') return [t('create.p.radius'), t('create.p.segments'), ''];
    if (type === 'cone') return [t('create.p.r_base'), t('create.p.r_top'), t('create.p.height')];
    return [t('create.p.length'), t('create.p.width'), t('create.p.height')];
  };

  const isPrimitive = !GENRULE_TYPES.has(form.objectType) && !CATALOG_TYPES.has(form.objectType) && !LINE_GENRULE_TYPES.has(form.objectType) && !DATUM_TYPES.has(form.objectType);
  const isDatum = DATUM_TYPES.has(form.objectType);
  const isLayeredSurface = LAYERED_TYPES.has(form.objectType);
  const isGenrule = GENRULE_TYPES.has(form.objectType);
  const isLinear = LINEAR_TYPES.has(form.objectType);
  /** Road / bridge / tunnel define a cross-section (width × thickness) directly. */
  const isCrossSectionLinear = LINEAR_TYPES.has(form.objectType) && (form.objectType === 'road' || form.objectType === 'bridge' || form.objectType === 'tunnel');

  return (
    <>
      <div className="create-panel">
        <div className="panel-header">
          <strong>{t('create.panel_title')}</strong>
          <button className="panel-close" onClick={handleClose} title="Close">&times;</button>
        </div>

        {showConfirm && (
          <div className="confirm-overlay" style={{ padding: '8px', background: '#2a2d35', borderRadius: '6px', marginBottom: '8px' }}>
            <p style={{ fontSize: '12px', marginBottom: '6px' }}>{t('create.confirm_close.msg')}</p>
            <button className="primary" style={{ marginRight: '6px', fontSize: '11px', padding: '2px 8px' }} onClick={handleConfirmClose}>{t('create.confirm_close.ok')}</button>
            <button style={{ fontSize: '11px', padding: '2px 8px' }} onClick={() => setShowConfirm(false)}>{t('create.confirm_close.keep')}</button>
          </div>
        )}

        {isError && error && (
          <div className="error-panel" style={{ padding: '8px', background: '#3d1f1f', borderRadius: '6px', marginBottom: '8px', color: '#ff6b6b' }}>
            <strong>{t('create.failed_alert')}</strong>{error}
            <div style={{ marginTop: '6px' }}>
              <button className="primary" style={{ marginRight: '6px', fontSize: '11px', padding: '2px 8px' }} onClick={onResetToPicking}>{t('create.retry_pick')}</button>
              <button style={{ fontSize: '11px', padding: '2px 8px' }} onClick={onCancel}>{t('create.dlg.cancel')}</button>
            </div>
          </div>
        )}

        <div className="create-row">
          <select aria-label="Object type" value={form.objectType} onChange={(e) => handleObjectTypeChange(e.target.value)}>
            {OBJECT_TYPE_OPTIONS.map((g) => (
              <optgroup key={g.group} label={t(g.group as never)}>
                {g.types.map((t) => (
                  <option key={t} value={t}>{objectTypeLabel(t)} {t}</option>
                ))}
              </optgroup>
            ))}
          </select>
          <input aria-label="Name" placeholder={t('create.ph.name')} value={form.name} onChange={(e) => handleChange({ name: e.target.value })} />
        </div>
        {errors.name && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.name}</small>}

        <ParentSelector
          tree={tree}
          value={form.parentId}
          onChange={(id) => handleChange({ parentId: id })}
        />

        {form.objectType === 'wall' && (
          <div className="create-row create-row--labeled">
            <span className="field-label">{t('create.wall.form')}</span>
            <select
              aria-label="Wall form"
              value={form.form}
              onChange={(e) => handleChange(withGenesisForm(form, e.target.value as 'panel' | 'region'))}
            >
              <option value="panel">{t('create.wall.form_panel')}</option>
              <option value="region">{t('create.wall.form_region')}</option>
            </select>
            <small className="hint">
              {form.form === 'panel' ? t('create.wall.hint_panel') : t('create.wall.hint_region')}
            </small>
          </div>
        )}

        {form.objectType === 'roof' && (
          <div className="create-row create-row--labeled">
            <span className="field-label">{t('create.roof.mode')}</span>
            <select
              aria-label="Roof mode"
              value={form.roofMode}
              onChange={(e) => {
                const roofMode = e.target.value as 'footprint' | 'extrusion';
                handleChange(withGenesisForm(
                  { ...form, roofMode },
                  roofMode === 'extrusion' ? 'panel' : 'region',
                ));
              }}
            >
              <option value="footprint">{t('create.roof.footprint')}</option>
              <option value="extrusion">{t('create.roof.extrusion')}</option>
            </select>
            <small className="hint">
              {form.roofMode === 'footprint' ? t('create.roof.hint_footprint') : t('create.roof.hint_extrusion')}
            </small>
          </div>
        )}

        {form.objectType === 'column' && (
          <div className="create-row create-row--labeled">
            <span className="field-label">{t('create.column.mode')}</span>
            <select
              aria-label="Column mode"
              value={form.columnMode}
              onChange={(e) => {
                const columnMode = e.target.value as 'place' | 'segment';
                handleChange({
                  columnMode,
                  pickMode: columnMode === 'segment' ? { mode: 'segment', minClicks: 2 } : { mode: 'place', minClicks: 1 },
                });
              }}
            >
              <option value="place">{t('create.column.place')}</option>
              <option value="segment">{t('create.column.segment')}</option>
            </select>
            {form.columnMode === 'place' && (
              <div className="create-row create-row--labeled">
                <span className="field-label">{t('create.column.height')}</span>
                <input aria-label="Column height" placeholder="m" value={form.columnHeight} onChange={(e) => handleChange({ columnHeight: e.target.value })} />
<span className="field-unit">m</span>
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

        {form.objectType === 'storey' && (
          <div className="create-row create-row--labeled">
            <span className="field-label">{t('create.elevation')}</span>
            <input aria-label="Elevation" placeholder={t('create.ph.elevation')} value={form.elevation} onChange={(e) => handleChange({ elevation: e.target.value })} />
<span className="field-unit">m</span>
            {errors.elevation && <span className="field-error">{errors.elevation}</span>}
            <small className="hint">{t('create.hint_storey')}</small>
          </div>
        )}

        {isDatum && form.objectType !== 'storey' && (
          <div className="create-row">
            <span className="hint">{t('create.hint_container')}</span>
          </div>
        )}

        {EXTENT_TYPES.has(form.objectType) && (
          <LevelFields form={form} errors={errors} onChange={handleChange} />
        )}

        {isPrimitive && (
          <div className="create-row create-row--labeled">
            <select aria-label="Geometry type" value={form.geometryType} onChange={(e) => handleGeometryTypeChange(e.target.value as GeometryType)}>
              <option value="box">{t('create.geom.box')}</option>
              <option value="cylinder">{t('create.geom.cylinder')}</option>
              <option value="sphere">{t('create.geom.sphere')}</option>
              <option value="cone">{t('create.geom.cone')}</option>
            </select>
            <span className="field-label">{paramLabels(form.geometryType)[0]}</span>
            <input aria-label={paramLabels(form.geometryType)[0]} placeholder="" value={form.p1} onChange={(e) => handleChange({ p1: e.target.value })} />
            <span className="field-label">{paramLabels(form.geometryType)[1]}</span>
            <input aria-label={paramLabels(form.geometryType)[1]} placeholder="" value={form.p2} onChange={(e) => handleChange({ p2: e.target.value })} />
            {paramLabels(form.geometryType)[2] && (
              <>
                <span className="field-label">{paramLabels(form.geometryType)[2]}</span>
                <input aria-label={paramLabels(form.geometryType)[2]} placeholder="" value={form.p3} onChange={(e) => handleChange({ p3: e.target.value })} />
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
            <span className="field-label">{t('create.section.w')}</span>
            <input aria-label="Section width" value={form.width} onChange={(e) => handleChange({ width: e.target.value })} />
<span className="field-unit">mm</span>
            <span className="field-label">{t('create.section.h')}</span>
            <input aria-label="Section height" value={form.thickness} onChange={(e) => handleChange({ thickness: e.target.value })} />
<span className="field-unit">mm</span>
          </div>
        )}

        {isLinear ? (
          <>
            <div className="create-row create-row--labeled">
              <span className="field-label">{t('create.start')}</span>
              <input aria-label="Start X" placeholder="X" value={form.startX} onChange={(e) => handleChange({ startX: e.target.value })} />
<span className="field-unit">m</span>
              <input aria-label="Start Y" placeholder={t('create.ph.y_height')} value={form.startY} onChange={(e) => handleChange({ startY: e.target.value })} />
<span className="field-unit">m</span>
              <input aria-label="Start Z" placeholder="Z" value={form.startZ} onChange={(e) => handleChange({ startZ: e.target.value })} />
<span className="field-unit">m</span>
            </div>
            <div className="create-row create-row--labeled">
              <span className="field-label">{t('create.end')}</span>
              <input aria-label="End X" placeholder="X" value={form.endX} onChange={(e) => handleChange({ endX: e.target.value })} />
<span className="field-unit">m</span>
              <input aria-label="End Y" placeholder={t('create.ph.y_height')} value={form.endY} onChange={(e) => handleChange({ endY: e.target.value })} />
<span className="field-unit">m</span>
              <input aria-label="End Z" placeholder="Z" value={form.endZ} onChange={(e) => handleChange({ endZ: e.target.value })} />
<span className="field-unit">m</span>
            </div>
          </>
        ) : (
          !isGenrule && !isDatum && (
            <div className="create-row create-row--labeled">
              <span className="field-label">{t('create.position')}</span>
              <input aria-label="Position X" placeholder="X" value={form.posX} onChange={(e) => handleChange({ posX: e.target.value })} />
<span className="field-unit">m</span>
              <input aria-label="Position Y" placeholder={t('create.ph.y_height')} value={form.posY} onChange={(e) => handleChange({ posY: e.target.value })} />
<span className="field-unit">m</span>
              <input aria-label="Position Z" placeholder="Z" value={form.posZ} onChange={(e) => handleChange({ posZ: e.target.value })} />
<span className="field-unit">m</span>
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

/** Bottom/top level selectors for the vertical-extent contract (P8-S3). */
function LevelFields({
  form, errors, onChange,
}: {
  form: CreationForm;
  errors: Record<string, string>;
  onChange: (p: Partial<CreationForm>) => void;
}) {
    const { t } = useI18n();
  const [levels, setLevels] = useState<StoreyOption[]>([]);

  useEffect(() => {
    let cancelled = false;
    api.listObjects()
      .then((objects) => { if (!cancelled) setLevels(storeyOptions(objects)); })
      .catch(() => { if (!cancelled) setLevels([]); });
    return () => { cancelled = true; };
  }, []);

  const label = (l: StoreyOption) => `${l.name}（${l.elevation.toFixed(2)} m）`;

  if (levels.length === 0) {
    return (
      <div className="create-row">
        <span className="hint">{t('create.levels.none')}</span>
      </div>
    );
  }

  return (
    <div className="create-row create-row--labeled">
      <span className="field-label">{t('create.levels.bottom')}</span>
      <select aria-label="Bottom level" value={form.bottomLevelId} onChange={(e) => onChange({ bottomLevelId: e.target.value })}>
        <option value="">{t('create.levels.none_opt')}</option>
        {levels.map((l) => <option key={l.id} value={l.id}>{label(l)}</option>)}
      </select>
      <span className="field-label">{t('create.levels.top')}</span>
      <select aria-label="Top level" value={form.topLevelId} onChange={(e) => onChange({ topLevelId: e.target.value })}>
        <option value="">{t('create.levels.none_opt')}</option>
        {levels.map((l) => <option key={l.id} value={l.id}>{label(l)}</option>)}
      </select>
      {errors.bottomLevelId && <span className="field-error">{errors.bottomLevelId}</span>}
    </div>
  );
}

function DoorWindowFields({ form, errors, onChange }: {
  form: CreationForm;
  errors: Record<string, string>;
  onChange: (p: Partial<CreationForm>) => void;
}) {
  const { t } = useI18n();
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

  if (loading) return <div className="create-row"><span>{t('create.walls.loading')}</span></div>;
  if (walls.length === 0) return <div className="create-row"><span className="hint"><IcoWarning /> {t('create.walls.empty')}</span></div>;

  return (
    <>
      <div className="create-row">
        <select aria-label="Target wall" value={form.targetWallId} onChange={(e) => onChange({ targetWallId: e.target.value })}>
          <option value="">{t('create.walls.pick')}</option>
          {walls.map((w) => <option key={w.id} value={w.id}>{w.name}</option>)}
        </select>
      </div>
      {errors.targetWallId && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.targetWallId}</small>}
      <div className="create-row create-row--labeled">
        <span className="field-label">{t('create.opening_w')}</span>
        <input aria-label="Opening width" placeholder="m" value={form.openingWidth} onChange={(e) => onChange({ openingWidth: e.target.value })} />
<span className="field-unit">m</span>
        <span className="field-label">{t('create.opening_h')}</span>
        <input aria-label="Opening height" placeholder="m" value={form.openingHeight} onChange={(e) => onChange({ openingHeight: e.target.value })} />
<span className="field-unit">m</span>
      </div>
      {errors.openingWidth && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.openingWidth}</small>}
      {errors.openingHeight && <small className="hint" style={{ color: '#ff6b6b' }}>{errors.openingHeight}</small>}
      <div className="create-row create-row--labeled">
        <span className="field-label">{t('create.position_x')}</span>
        <input aria-label="Horizontal offset" placeholder="m" value={form.posX} onChange={(e) => onChange({ posX: e.target.value })} />
<span className="field-unit">m</span>
        <span className="field-label">{form.objectType === 'door' ? t('create.z_door') : t('create.z_sill')}</span>
        <input aria-label="Vertical offset" placeholder="m" value={form.posZ} onChange={(e) => onChange({ posZ: e.target.value })} />
<span className="field-unit">m</span>
      </div>
      <small className="hint">{t('create.hint_opening', { kind: form.objectType === 'door' ? t('panel.domain.type_door') : t('panel.domain.type_window') })}</small>
    </>
  );
}
