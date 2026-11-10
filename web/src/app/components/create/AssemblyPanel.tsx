import { useMemo, useState } from 'react';
import { api } from '../../../shared/api';
import { nextAutoName } from '../../../shared/naming';
import { ensureCatalogLoaded, useCatalog } from '../../../shared/catalog';
import { ComponentParamPanel } from './ComponentParamPanel';
import type {
  AssemblyPartInput,
  ComponentDef,
  ParamDef,
  TreeNode,
} from '../../../shared/types';
import { useToastApi } from '../../../shared/toast';
import { useI18n } from '../../../shared/i18n';
import { IcoClose } from '../Icons';

type PartSource = 'component' | 'primitive';

type DraftPart = {
  name: string;
  source: PartSource;
  componentDefId: string;
  params: Record<string, number>;
  shape: string;
  p1: number;
  p2: number;
  p3: number;
  x: number;
  y: number;
  z: number;
  rotationDeg: number;
};

const EMPTY_DRAFT: DraftPart = {
  name: '',
  source: 'primitive',
  componentDefId: '',
  params: {},
  shape: 'box',
  p1: 2,
  p2: 1,
  p3: 1,
  x: 0,
  y: 0,
  z: 0,
  rotationDeg: 0,
};

function flattenTree(nodes: TreeNode[], depth = 0, out: { id: string; label: string }[] = []) {
  for (const node of nodes) {
    out.push({ id: node.id, label: `${'· '.repeat(depth)}${node.name} (${node.object_type})` });
    flattenTree(node.children ?? [], depth + 1, out);
  }
  return out;
}

function shapeLabels(shape: string, t: (k: 'create.p.radius' | 'create.p.height' | 'create.p.r_base' | 'create.p.r_top' | 'create.p.width' | 'create.p.depth_assy') => string): [string, string, string] {
  switch (shape) {
    case 'cylinder':
      return [t('create.p.radius'), t('create.p.height'), ''];
    case 'sphere':
      return [t('create.p.radius'), '', ''];
    case 'cone':
      return [t('create.p.r_base'), t('create.p.r_top'), t('create.p.height')];
    default:
      return [t('create.p.width'), t('create.p.height'), t('create.p.depth_assy')];
  }
}

function defaultParams(def: ComponentDef | undefined): Record<string, number> {
  const out: Record<string, number> = {};
  for (const p of def?.params ?? []) out[p.name] = p.default_value;
  return out;
}

export function AssemblyPanel({
  onCreated,
  tree = [],
}: {
  onCreated: () => void;
  tree?: TreeNode[];
}) {
  const toast = useToastApi();
  const { t } = useI18n();
  const [panelOpen, setPanelOpen] = useState(true);
  const [assyName, setAssyName] = useState('');
  const [parentId, setParentId] = useState('');
  const [parts, setParts] = useState<AssemblyPartInput[]>([]);
  const [draft, setDraft] = useState<DraftPart>(EMPTY_DRAFT);
  const [busy, setBusy] = useState(false);
  const { catalog } = useCatalog();
  void ensureCatalogLoaded().catch(() => { /* surfaced on create */ });

  const parentOptions = useMemo(() => flattenTree(tree), [tree]);
  const selectedDef = useMemo(
    () => catalog.find((d) => d.id === draft.componentDefId),
    [catalog, draft.componentDefId],
  );

  function updateDraft(partial: Partial<DraftPart>) {
    setDraft((prev) => {
      const next = { ...prev, ...partial };
      if (partial.componentDefId !== undefined) {
        next.params = defaultParams(
          catalog.find((d) => d.id === partial.componentDefId),
        );
      }
      if (partial.shape !== undefined) {
        const dims: Record<string, [number, number, number]> = {
          box: [2, 1, 1],
          cylinder: [0.5, 3, 0],
          sphere: [1, 0, 0],
          cone: [1, 0.3, 2],
        };
        const [p1, p2, p3] = dims[partial.shape] ?? [1, 1, 1];
        Object.assign(next, { p1, p2, p3 });
      }
      return next;
    });
  }

  function addPart() {
    const name = draft.name.trim();
    if (!name) {
      toast.error(t('assy.err_name_required'));
      return;
    }
    const position = { x: draft.x, y: draft.y, z: draft.z };
    if (draft.source === 'component') {
      if (!draft.componentDefId) {
        toast.error(t('assy.err_def_required'));
        return;
      }
      setParts((prev) => [
        ...prev,
        {
          name,
          component_def_id: draft.componentDefId,
          params: draft.params,
          position,
          rotation_z: (draft.rotationDeg * Math.PI) / 180,
        },
      ]);
    } else {
      const dims = [draft.p1, draft.p2, draft.p3];
      const valid =
        draft.shape === 'box'
          ? dims.every((v) => v > 0)
          : draft.shape === 'cylinder'
            ? draft.p1 > 0 && draft.p2 > 0
            : draft.shape === 'sphere'
              ? draft.p1 > 0
              : draft.p1 > 0 && draft.p3 > 0;
      if (!valid) {
        toast.error(t('assy.err_dims_positive'));
        return;
      }
      setParts((prev) => [
        ...prev,
        {
          name,
          shape: draft.shape,
          p1: draft.p1,
          p2: draft.p2,
          p3: draft.p3,
          position,
          rotation_z: (draft.rotationDeg * Math.PI) / 180,
        },
      ]);
    }
    setDraft({ ...EMPTY_DRAFT, source: draft.source });
  }

  async function createAssembly() {
    if (parts.length === 0) return;
    setBusy(true);
    try {
      const names = (await api.listObjects()).map((o) => o.name);
      const name = assyName.trim() || nextAutoName('assembly', names);
      const res = await api.createAssembly({
        name,
        parent_id: parentId || undefined,
        skip_series_gate: true,
        parts,
      });
      toast.success(t('assy.created', { name, count: res.part_count }));
      setParts([]);
      setAssyName('');
      setParentId('');
      setPanelOpen(false);
      onCreated();
    } catch (err) {
      toast.error(`${t('assy.create_failed')}: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setBusy(false);
    }
  }

  if (!panelOpen) return null;

  return (
    <div className="create-panel">
      <div className="panel-header">
        <strong>{t('assy.title')}</strong>
        <button className="panel-close" onClick={() => setPanelOpen(false)} title="Close">
          &times;
        </button>
      </div>
      {/* F4 (UI/UX audit): the mm/m unit trap — catalog params are mm while
          positions are metres. */}
      <p className="field-label" style={{ color: 'var(--text-muted, #9aa3ad)' }}>
        {t('assy.units_note')}
        
      </p>

      <div className="create-row">
        <input
          placeholder={t('assy.name_ph')}
          value={assyName}
          onChange={(e) => setAssyName(e.target.value)}
        />
      </div>
      <div className="create-row">
        <select value={parentId} onChange={(e) => setParentId(e.target.value)}>
          <option value="">{t('assy.parent_root')}</option>
          {parentOptions.map((o) => (
            <option key={o.id} value={o.id}>
              {o.label}
            </option>
          ))}
        </select>
      </div>

      {parts.length > 0 && (
        <div className="assy-list">
          <label className="field-label">{t('assy.parts_count', { count: parts.length })}</label>
          {parts.map((p, i) => (
            <div key={i} className="assy-item">
              <span>{p.name}</span>
              <span className="assy-item-detail">{p.component_def_id ? t('assy.def_source') : p.shape}</span>
              <span className="assy-item-detail">
                pos: {p.position?.x ?? 0},{p.position?.y ?? 0},{p.position?.z ?? 0}
              </span>
              <button
                className="assy-remove"
                onClick={() => setParts(parts.filter((_, j) => j !== i))}
                title={t('assy.remove')}
              >
                <IcoClose />
              </button>
            </div>
          ))}
        </div>
      )}

      <div className="assy-add-form">
        <label className="field-label">{t('assy.add_part')}</label>
        <div className="create-row">
          <input
            placeholder={t('layer.name_ph')}
            value={draft.name}
            onChange={(e) => updateDraft({ name: e.target.value })}
          />
          <select
            value={draft.source}
            onChange={(e) => updateDraft({ source: e.target.value as PartSource })}
          >
            <option value="primitive">{t('assy.kind_primitive')}</option>
            <option value="component">{t('assy.kind_component')}</option>
          </select>
        </div>

        {draft.source === 'component' ? (
          <>
            <div className="create-row">
              <select
                value={draft.componentDefId}
                onChange={(e) => updateDraft({ componentDefId: e.target.value })}
              >
                <option value="">{t('assy.pick_def')}</option>
                {catalog.map((d) => (
                  <option key={d.id} value={d.id}>
                    {d.name}
                  </option>
                ))}
              </select>
            </div>
            {selectedDef && selectedDef.params.length > 0 && (
              <ComponentParamPanel
                params={selectedDef.params as ParamDef[]}
                values={draft.params}
                onChange={(name, value) =>
                  setDraft((prev) => ({ ...prev, params: { ...prev.params, [name]: value } }))
                }
                compact
              />
            )}
          </>
        ) : (
          <>
            <div className="create-row">
              <select
                value={draft.shape}
                onChange={(e) => updateDraft({ shape: e.target.value })}
              >
                <option value="box">{t('assy.prim.box')}</option>
                <option value="cylinder">{t('assy.prim.cylinder')}</option>
                <option value="sphere">{t('assy.prim.sphere')}</option>
                <option value="cone">{t('assy.prim.cone')}</option>
              </select>
            </div>
            <div className="create-row">
              {(() => {
                const [a, b, c] = shapeLabels(draft.shape, t);
                return (
                  <>
                    <input
                      placeholder={a ? `${a} (m)` : '— (m)'}
                      value={draft.p1 || ''}
                      onChange={(e) => updateDraft({ p1: parseFloat(e.target.value) || 0 })}
                    />
                    <input
                      placeholder={b ? `${b} (m)` : '— (m)'}
                      value={draft.p2 || ''}
                      onChange={(e) => updateDraft({ p2: parseFloat(e.target.value) || 0 })}
                    />
                    <input
                      placeholder={c ? `${c} (m)` : '— (m)'}
                      value={draft.p3 || ''}
                      onChange={(e) => updateDraft({ p3: parseFloat(e.target.value) || 0 })}
                    />
                  </>
                );
              })()}
            </div>
          </>
        )}

        <div className="create-row">
          <input
            placeholder={t('assy.ph.x')}
            value={draft.x || ''}
            onChange={(e) => updateDraft({ x: parseFloat(e.target.value) || 0 })}
          />
          <input
            placeholder={t('assy.ph.y')}
            value={draft.y || ''}
            onChange={(e) => updateDraft({ y: parseFloat(e.target.value) || 0 })}
          />
          <input
            placeholder={t('assy.ph.z')}
            value={draft.z || ''}
            onChange={(e) => updateDraft({ z: parseFloat(e.target.value) || 0 })}
          />
          <input
            placeholder={t('assy.ph.rot_z')}
            value={draft.rotationDeg || ''}
            onChange={(e) => updateDraft({ rotationDeg: parseFloat(e.target.value) || 0 })}
          />
        </div>

        <button className="primary" onClick={addPart}>
          {t('assy.btn.add')}
        </button>
      </div>

      {parts.length > 0 && (
        <button
          className="primary create-btn"
          disabled={busy}
          onClick={() => void createAssembly()}
        >
          {busy ? t('create.saving') : t('assy.btn.submit', { count: parts.length })}
        </button>
      )}
    </div>
  );
}
