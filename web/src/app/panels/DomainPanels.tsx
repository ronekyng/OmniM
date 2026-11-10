import { useEffect, useMemo, useState } from 'react';
import { useAppStore } from '../store/appStore';
import { api } from '../../shared/api';
import { notify } from '../../shared/toast';
import { nextAutoName } from '../../shared/naming';
import { useI18n } from '../../shared/i18n';
import type { ComponentDef, JointItem, OpeningsScheduleRow } from '../../shared/types';
import { Glyph, IcoClose, type GlyphName } from '../components/Icons';
import { SkeletonLines } from '../components/Loading';
import { useDelayedLoading } from '../../shared/useDelayedLoading';

// Domain quick-create always bypasses the series gate (components are built-in).
function quickInstantiate(body: { component_def_id: string; instance_name: string; params: Record<string, number> }) {
  return api.instantiateFromCatalog({ ...body, skip_series_gate: true });
}

/** Lowest free `<prefix>-nnn` across the project's current object names. */
async function autoInstanceName(prefix: string): Promise<string> {
  const objects = await api.listObjects().catch(() => []);
  return nextAutoName(prefix, objects.map((o) => o.name));
}

export const DOMAIN_LABEL_KEYS: Record<string, string> = {
  structural: 'domain.structural',
  building: 'domain.building',
  civil: 'domain.civil',
  plant: 'domain.plant',
  mechanical: 'domain.mechanical',
  piping: 'domain.piping',
};

type QuickCreateAction = {
  labelKey: string;
  icon: GlyphName;
  action: () => Promise<void>;
};

function DomainPanelContent({ domain, extra }: { domain: string; extra?: React.ReactNode }) {
  const setOpen = useAppStore((s) => s.setDomainPanelOpen);
  const { t } = useI18n();
  const [catalog, setCatalog] = useState<ComponentDef[]>([]);
  const [objectNames, setObjectNames] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [reloadKey, setReloadKey] = useState(0);
  const showSkeleton = useDelayedLoading(loading);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.listCatalog(), api.listObjects().catch(() => [])])
      .then(([comps, objects]) => {
        setCatalog(comps);
        setObjectNames(objects.map((o) => o.name));
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, [reloadKey]);

  const filtered = useMemo(() => catalog.filter((c) => c.domain === domain), [catalog, domain]);

  const quickActions = useMemo((): QuickCreateAction[] => {
    const actions: QuickCreateAction[] = [];
    if (domain === 'building') {
      actions.push({
        labelKey: 'quick.wall_5x3', icon: 'wall',
        action: async () => { await api.genruleCreate({ genrule: 'wall', name: await autoInstanceName('wall'), length: 5, height: 3, core_thickness: 0.2, finish_thickness: 0.02 }); },
      });
      actions.push({
        labelKey: 'quick.slab_4x5', icon: 'slab',
        action: async () => { await api.genruleCreate({ genrule: 'slab', name: await autoInstanceName('slab'), length: 4, width: 5, structural_thickness: 0.2, finish_thickness: 0.02 }); },
      });
      actions.push({
        labelKey: 'quick.col_hea4', icon: 'column',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Column-HEA', instance_name: await autoInstanceName('column'), params: { height: 300, width: 300, web_thickness: 8, flange_thickness: 14, column_height: 4000 } }); },
      });
      actions.push({
        labelKey: 'quick.col_rc4', icon: 'column',
        action: async () => { await quickInstantiate({ component_def_id: 'Concrete-Column-Rect', instance_name: await autoInstanceName('column'), params: { width: 400, depth: 400, column_height: 4000 } }); },
      });
      actions.push({
        labelKey: 'quick.beam_ipe6', icon: 'beam',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Beam-IPE', instance_name: await autoInstanceName('beam'), params: { height: 240, width: 120, web_thickness: 6.2, flange_thickness: 9.8, length: 6000 } }); },
      });
      actions.push({
        labelKey: 'quick.beam_rc6', icon: 'beam',
        action: async () => { await quickInstantiate({ component_def_id: 'Concrete-Beam-Rect', instance_name: await autoInstanceName('beam'), params: { width: 300, depth: 500, length: 6000 } }); },
      });
      actions.push({
        labelKey: 'quick.door_1x21', icon: 'door',
        action: async () => { await quickInstantiate({ component_def_id: 'Door-Single', instance_name: await autoInstanceName('door'), params: { width: 1000, height: 2100, frame_depth: 200 } }); },
      });
      actions.push({
        labelKey: 'quick.win_15', icon: 'window',
        action: async () => { await quickInstantiate({ component_def_id: 'Window-Fixed', instance_name: await autoInstanceName('window'), params: { width: 1500, height: 1500, frame_depth: 200 } }); },
      });
      actions.push({
        labelKey: 'quick.rebar400', icon: 'rebar',
        action: async () => { await quickInstantiate({ component_def_id: 'Rebar-Cage', instance_name: await autoInstanceName('rebar'), params: { bar_ring_radius: 200, main_bars: 8, length: 3000, diameter: 16, stirrup_spacing: 200, stirrup_radius: 192 } }); },
      });
      actions.push({
        labelKey: 'quick.duct_rect', icon: 'duct',
        action: async () => { await quickInstantiate({ component_def_id: 'Duct-Rect', instance_name: await autoInstanceName('duct'), params: { width: 500, height: 250, length: 3000 } }); },
      });
      actions.push({
        labelKey: 'quick.tray_3m', icon: 'tray',
        action: async () => { await quickInstantiate({ component_def_id: 'CableTray', instance_name: await autoInstanceName('tray'), params: { width: 300, height: 100, thickness: 3, length: 3000 } }); },
      });
    } else if (domain === 'civil') {
      actions.push({
        labelKey: 'quick.road_10x7', icon: 'road',
        action: async () => { await api.genruleCreate({ genrule: 'road', name: await autoInstanceName('road'), length: 10, width: 7, thickness: 0.3 }); },
      });
      actions.push({
        labelKey: 'quick.pipe_dn300', icon: 'pipe',
        action: async () => { await api.genruleCreate({ genrule: 'pipe', name: await autoInstanceName('pipe'), length: 6, radius: 0.15 }); },
      });
      actions.push({
        labelKey: 'quick.manhole', icon: 'manhole',
        action: async () => { await quickInstantiate({ component_def_id: 'Manhole', instance_name: await autoInstanceName('manhole'), params: { diameter: 1200, depth: 2500 } }); },
      });
    } else if (domain === 'plant') {
      actions.push({
        labelKey: 'quick.tank_v', icon: 'tank',
        action: async () => { await quickInstantiate({ component_def_id: 'VerticalTank', instance_name: await autoInstanceName('tank'), params: { diameter: 3000, height: 5000, wall_thickness: 8 } }); },
      });
      actions.push({
        labelKey: 'quick.pump_dn100', icon: 'pump',
        action: async () => { await quickInstantiate({ component_def_id: 'CentrifugalPump', instance_name: await autoInstanceName('pump'), params: { DN: 100, height: 600, width: 400 } }); },
      });
      actions.push({
        labelKey: 'quick.valve_dn150', icon: 'valve',
        action: async () => { await quickInstantiate({ component_def_id: 'GateValve-CS', instance_name: await autoInstanceName('valve'), params: { DN: 150, pressure_class: 150.0 } }); },
      });
      actions.push({
        labelKey: 'quick.hx', icon: 'hx',
        action: async () => { await quickInstantiate({ component_def_id: 'HeatExchanger', instance_name: await autoInstanceName('hx'), params: { shell_diameter: 500, shell_length: 6000 } }); },
      });
      actions.push({
        labelKey: 'quick.compressor', icon: 'compressor',
        action: async () => { await quickInstantiate({ component_def_id: 'Compressor', instance_name: await autoInstanceName('comp'), params: { length: 2000, width: 1200, height: 1200 } }); },
      });
      actions.push({
        labelKey: 'quick.pipe_cs_dn200', icon: 'pipe',
        action: async () => { await api.genruleCreate({ genrule: 'pipe', name: await autoInstanceName('pipe'), length: 6, radius: 0.1 }); },
      });
    } else if (domain === 'mechanical') {
      actions.push({
        labelKey: 'quick.beam_ipe6', icon: 'beam',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Beam-IPE', instance_name: await autoInstanceName('beam'), params: { height: 240, width: 120, web_thickness: 6.2, flange_thickness: 9.8, length: 6000 } }); },
      });
      actions.push({
        labelKey: 'quick.col_heb4', icon: 'column',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Column-HEB', instance_name: await autoInstanceName('column'), params: { height: 300, width: 300, web_thickness: 11, flange_thickness: 19, column_height: 4000 } }); },
      });
      actions.push({
        labelKey: 'quick.chs_200', icon: 'pipe',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-CHS', instance_name: await autoInstanceName('chs'), params: { diameter: 200, thickness: 8, length: 6000 } }); },
      });
      actions.push({
        labelKey: 'quick.angle_100x10', icon: 'angle',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Angle', instance_name: await autoInstanceName('angle'), params: { leg1: 100, leg2: 100, thickness: 10, length: 6000 } }); },
      });
      actions.push({
        labelKey: 'quick.gear_m4z20', icon: 'gear',
        action: async () => { await quickInstantiate({ component_def_id: 'Gear', instance_name: await autoInstanceName('gear'), params: { module: 4, teeth: 20, width: 40 } }); },
      });
    }
    return actions;
  }, [domain]);

  async function handleQuickAction(action: QuickCreateAction) {
    setBusy(action.labelKey);
    try {
      await action.action();
      const store = useAppStore.getState();
      store.markDirty();
      await store.refresh();
      store.reloadScene();
    } catch (e) {
      notify.error(`${t('quick.failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
    setBusy(null);
  }

  function handleDragStart(e: React.DragEvent, comp: ComponentDef) {
    const params: Record<string, number> = {};
    comp.params.forEach((p) => { params[p.name] = p.default_value; });
    e.dataTransfer.setData('application/x-omnim-catalog', JSON.stringify({
      component_def_id: comp.id,
      instance_name: nextAutoName(comp.name.toLowerCase().replace(/[^a-z0-9]+/g, '-'), objectNames),
      params,
    }));
    e.dataTransfer.effectAllowed = 'copy';
  }

  function handleDragStartQuick(e: React.DragEvent, action: QuickCreateAction) {
    e.dataTransfer.setData('text/plain', t(action.labelKey as never));
    e.dataTransfer.effectAllowed = 'copy';
  }

  async function handleInstantiate(comp: ComponentDef) {
    setBusy(comp.name);
    try {
      const params: Record<string, number> = {};
      comp.params.forEach((p) => { params[p.name] = p.default_value; });
      await quickInstantiate({
        component_def_id: comp.id,
        instance_name: nextAutoName(comp.name.toLowerCase().replace(/[^a-z0-9]+/g, '-'), objectNames),
        params,
      });
      const store = useAppStore.getState();
      store.markDirty();
      await store.refresh();
      store.reloadScene();
    } catch (e) {
      notify.error(`${t('quick.instantiate_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
    setBusy(null);
  }

  return (
    <div className="domain-panel">
      <div className="domain-panel-header">
        <span>{(() => { const k = DOMAIN_LABEL_KEYS[domain]; return k ? t(k as never) : domain; })()}</span>
        <button className="icon-btn" onClick={() => setOpen(domain, false)} title="Close"><IcoClose /></button>
      </div>

      {/* Quick-create buttons */}
      <div className="domain-panel-section">
        <label className="domain-panel-section-label">{t('panel.domain.quick_create')}</label>
        <div className="domain-panel-actions">
          {quickActions.map((a) => (
            <button
              key={a.labelKey}
              className="domain-panel-action-btn"
              disabled={busy === a.labelKey}
              onClick={() => void handleQuickAction(a)}
            >
              <span className="domain-panel-action-icon"><Glyph name={a.icon} /></span>
              <span className="domain-panel-action-label">{t(a.labelKey as never)}</span>
            </button>
          ))}
        </div>
      </div>

      {extra}

      {/* Domain component list */}
      <div className="domain-panel-section">
        <label className="domain-panel-section-label">
          {t('panel.domain.builtins')} ({filtered.length})
        </label>
        <div className="domain-panel-list">
          {loading && showSkeleton && (
            <div aria-busy="true" aria-label="Loading components">
              <SkeletonLines count={4} />
            </div>
          )}
          {error && (
            <div className="panel-state panel-state--error" role="alert">
              <p>{error}</p>
              <button type="button" onClick={() => setReloadKey((n) => n + 1)}>Retry</button>
            </div>
          )}
          {!loading && !error && filtered.length === 0 && (
            <div className="panel-state">
                        {t('panel.domain.builtins_none')}

            </div>
          )}
          {filtered.map((c) => (
            <div className="domain-panel-item" key={c.id} draggable onDragStart={(e) => handleDragStart(e, c)}>
              <div className="domain-panel-item-info">
                <span className="name">{c.name}</span>
                <span className="meta">{c.category}</span>
              </div>
              <button
                className="domain-panel-item-btn"
                disabled={busy === c.name}
                onClick={() => void handleInstantiate(c)}
                title={t('avail.add_to_model')}
              >
                +
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

/**
 * Phase 8-5: domain panels also surface the domain-specific side reads —
 * the mechanical panel lists motion joints touching the selected object
 * (ASSEMBLY §9); the building panel lists the door/window schedule.
 */
function SelectedJointsSection() {
  const { t } = useI18n();
  const primaryId = useAppStore((s) => s.selectedIds[0]) ?? null;
  const [joints, setJoints] = useState<JointItem[] | null>(null);
  const [error, setError] = useState(false);
  useEffect(() => {
    let cancelled = false;
    setJoints(null);
    setError(false);
    if (!primaryId) return undefined;
    api
      .listJoints(primaryId)
      .then((list) => {
        if (!cancelled) setJoints(list);
      })
      .catch(() => {
        if (!cancelled) setError(true);
      });
    return () => {
      cancelled = true;
    };
  }, [primaryId]);

  return (
    <div className="domain-panel-section">
      <label className="domain-panel-section-label">
        {primaryId ? t('panel.domain.joints') : t('panel.domain.joints').replace(/（.*/, '') === t('panel.domain.joints') ? t('panel.domain.joints') : t('panel.domain.joints')}
      </label>
      {joints === null && primaryId && !error && (
        <div className="panel-state" aria-busy="true">
          <SkeletonLines count={2} />
        </div>
      )}
      {error && (
        <div className="panel-state panel-state--error" role="alert">
          {t('panel.domain.joints_error')}
          <button type="button" onClick={() => { setJoints(null); setError(false); }}>
            {t('panel.domain.retry')}
          </button>
        </div>
      )}
      {joints !== null && joints.length === 0 && (
        <div className="panel-state">{t('panel.domain.joints_none')}</div>
      )}
      {joints !== null && joints.length > 0 && (
        <div className="domain-panel-list" role="list" aria-label="Motion joints">
          {joints.map((j) => (
            <div className="domain-panel-item" role="listitem" key={j.id}>
              <div className="domain-panel-item-info">
                <span className="name">{j.name}</span>
                <span className="meta">{j.kind}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function OpeningsScheduleSection() {
  const { t } = useI18n();
  const [rows, setRows] = useState<OpeningsScheduleRow[] | null>(null);
  const [error, setError] = useState(false);
  const [reloadKey, setReloadKey] = useState(0);
  useEffect(() => {
    let cancelled = false;
    setRows(null);
    setError(false);
    api
      .openingsSchedule()
      .then((list) => {
        if (!cancelled) setRows(list);
      })
      .catch(() => {
        if (!cancelled) setError(true);
      });
    return () => {
      cancelled = true;
    };
  }, [reloadKey]);

  const total = rows?.reduce((acc, r) => acc + r.count, 0) ?? 0;

  return (
    <div className="domain-panel-section">
      <label className="domain-panel-section-label">
        {t('panel.domain.schedule')} ({total})
        {rows !== null && (
          <button
            type="button"
            className="schedule-refresh"
            onClick={() => setReloadKey((n) => n + 1)}
            title={t('panel.domain.schedule_refresh')}
          >
            {t('panel.domain.schedule_refresh')}
          </button>
        )}
      </label>
      {rows === null && !error && (
        <div className="panel-state" aria-busy="true">
          <SkeletonLines count={3} />
        </div>
      )}
      {error && (
        <div className="panel-state panel-state--error" role="alert">
          {t('panel.domain.schedule_error')}
          <button type="button" onClick={() => { setRows(null); setError(false); }}>{t('panel.domain.retry')}</button>
        </div>
      )}
      {rows !== null && rows.length === 0 && (
        <div className="panel-state">{t('panel.domain.schedule_none')}</div>
      )}
      {rows !== null && rows.length > 0 && (
        <div className="domain-panel-list" role="table" aria-label="Openings schedule">
          <div className="schedule-row schedule-row-head" role="row">
            <span role="columnheader">{t('panel.domain.col_type')}</span>
            <span role="columnheader">{t('panel.domain.col_size')}</span>
            <span role="columnheader">{t('panel.domain.col_count')}</span>
          </div>
          {rows.map((r, i) => (
            <div
              className="schedule-row"
              role="row"
              key={`${r.object_type}-${r.width_mm}x${r.height_mm}-${r.leaf_count ?? 0}-${i}`}
            >
              <span role="cell">
                {r.object_type === 'door' ? t('panel.domain.type_door') : t('panel.domain.type_window')}
                {r.leaf_count && r.leaf_count > 1 ? `×${r.leaf_count}` : ''}
              </span>
              <span role="cell">{r.width_mm}×{r.height_mm}</span>
              <span role="cell">{r.count}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export function BuildingPanel() { return <DomainPanelContent domain="building" extra={<OpeningsScheduleSection />} />; }
export function CivilPanel() { return <DomainPanelContent domain="civil" />; }
export function PlantPanel() { return <DomainPanelContent domain="plant" />; }
export function MechanicalPanel() { return <DomainPanelContent domain="mechanical" extra={<SelectedJointsSection />} />; }
