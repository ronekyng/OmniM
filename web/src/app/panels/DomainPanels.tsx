import { useEffect, useMemo, useState } from 'react';
import { useAppStore } from '../store/appStore';
import { api } from '../../shared/api';
import type { ComponentDef } from '../../shared/types';
import { IcoClose } from '../components/Icons';

// Domain quick-create always bypasses the series gate (components are built-in).
function quickInstantiate(body: { component_def_id: string; instance_name: string; params: Record<string, number> }) {
  return api.instantiateFromCatalog({ ...body, skip_series_gate: true });
}

export const DOMAIN_LABELS: Record<string, string> = {
  structural: '建筑结构',
  building: '建筑',
  civil: '土木',
  plant: '工厂',
  mechanical: '机械',
  piping: '管道',
};

type QuickCreateAction = {
  label: string;
  icon: string;
  action: () => Promise<void>;
};

function DomainPanelContent({ domain }: { domain: string }) {
  const setOpen = useAppStore((s) => s.setDomainPanelOpen);
  const [catalog, setCatalog] = useState<ComponentDef[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState<string | null>(null);

  useEffect(() => {
    api.listCatalog()
      .then((comps) => setCatalog(comps))
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, []);

  const filtered = useMemo(() => catalog.filter((c) => c.domain === domain), [catalog, domain]);

  const quickActions = useMemo((): QuickCreateAction[] => {
    const actions: QuickCreateAction[] = [];
    if (domain === 'building') {
      actions.push({
        label: '墙体 5×3m', icon: '▦',
        action: async () => { await api.genruleCreate({ genrule: 'wall', name: `wall-${Date.now()}`, length: 5, height: 3, core_thickness: 0.2, finish_thickness: 0.02 }); },
      });
      actions.push({
        label: '楼板 4×5m', icon: '▬',
        action: async () => { await api.genruleCreate({ genrule: 'slab', name: `slab-${Date.now()}`, length: 4, width: 5, structural_thickness: 0.2, finish_thickness: 0.02 }); },
      });
      actions.push({
        label: '钢柱 HEA 4m', icon: '▯',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Column-HEA', instance_name: `column-${Date.now()}`, params: { height: 300, width: 300, web_thickness: 8, flange_thickness: 14, column_height: 4000 } }); },
      });
      actions.push({
        label: '混凝土柱 4m', icon: '▯',
        action: async () => { await quickInstantiate({ component_def_id: 'Concrete-Column-Rect', instance_name: `column-${Date.now()}`, params: { width: 400, depth: 400, column_height: 4000 } }); },
      });
      actions.push({
        label: '钢梁 IPE 6m', icon: '━',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Beam-IPE', instance_name: `beam-${Date.now()}`, params: { height: 240, width: 120, web_thickness: 6.2, flange_thickness: 9.8, length: 6000 } }); },
      });
      actions.push({
        label: '混凝土梁 6m', icon: '━',
        action: async () => { await quickInstantiate({ component_def_id: 'Concrete-Beam-Rect', instance_name: `beam-${Date.now()}`, params: { width: 300, depth: 500, length: 6000 } }); },
      });
      actions.push({
        label: '门 1×2.1m', icon: '⊞',
        action: async () => { await quickInstantiate({ component_def_id: 'Door-Single', instance_name: `door-${Date.now()}`, params: { width: 1000, height: 2100, frame_depth: 200 } }); },
      });
      actions.push({
        label: '窗 1.5×1.5m', icon: '⊟',
        action: async () => { await quickInstantiate({ component_def_id: 'Window-Fixed', instance_name: `window-${Date.now()}`, params: { width: 1500, height: 1500, frame_depth: 200 } }); },
      });
      actions.push({
        label: '钢筋笼 Ø400 3m', icon: '⤤',
        action: async () => { await quickInstantiate({ component_def_id: 'Rebar-Cage', instance_name: `rebar-${Date.now()}`, params: { bar_ring_radius: 200, main_bars: 8, length: 3000, diameter: 16, stirrup_spacing: 200, stirrup_radius: 192 } }); },
      });
      actions.push({
        label: '矩形风管 0.5×0.25m 3m', icon: '▦',
        action: async () => { await quickInstantiate({ component_def_id: 'Duct-Rect', instance_name: `duct-${Date.now()}`, params: { width: 500, height: 250, length: 3000 } }); },
      });
      actions.push({
        label: '电缆桥架 0.3m 3m', icon: '▬',
        action: async () => { await quickInstantiate({ component_def_id: 'CableTray', instance_name: `tray-${Date.now()}`, params: { width: 300, height: 100, thickness: 3, length: 3000 } }); },
      });
    } else if (domain === 'civil') {
      actions.push({
        label: '道路 10×7m', icon: '═',
        action: async () => { await api.genruleCreate({ genrule: 'road', name: `road-${Date.now()}`, length: 10, width: 7, thickness: 0.3 }); },
      });
      actions.push({
        label: '管道 DN300 6m', icon: '◎',
        action: async () => { await api.genruleCreate({ genrule: 'pipe', name: `pipe-${Date.now()}`, length: 6, radius: 0.15 }); },
      });
      actions.push({
        label: '检查井 Ø1.2×2.5m', icon: '◉',
        action: async () => { await quickInstantiate({ component_def_id: 'Manhole', instance_name: `manhole-${Date.now()}`, params: { diameter: 1200, depth: 2500 } }); },
      });
    } else if (domain === 'plant') {
      actions.push({
        label: '立式储罐 Ø3×5m', icon: '⏆',
        action: async () => { await quickInstantiate({ component_def_id: 'VerticalTank', instance_name: `tank-${Date.now()}`, params: { diameter: 3000, height: 5000, wall_thickness: 8 } }); },
      });
      actions.push({
        label: '离心泵 DN100', icon: '⏥',
        action: async () => { await quickInstantiate({ component_def_id: 'CentrifugalPump', instance_name: `pump-${Date.now()}`, params: { DN: 100, height: 600, width: 400 } }); },
      });
      actions.push({
        label: '闸阀 DN150', icon: '⏤',
        action: async () => { await quickInstantiate({ component_def_id: 'GateValve-CS', instance_name: `valve-${Date.now()}`, params: { DN: 150, pressure_class: 150.0 } }); },
      });
      actions.push({
        label: '管壳式换热器 Ø0.5×6m', icon: '⏦',
        action: async () => { await quickInstantiate({ component_def_id: 'HeatExchanger', instance_name: `hx-${Date.now()}`, params: { shell_diameter: 500, shell_length: 6000 } }); },
      });
      actions.push({
        label: '压缩机 2×1.2m', icon: '⏧',
        action: async () => { await quickInstantiate({ component_def_id: 'Compressor', instance_name: `comp-${Date.now()}`, params: { length: 2000, width: 1200, height: 1200 } }); },
      });
      actions.push({
        label: '管道 CS DN200 6m', icon: '◎',
        action: async () => { await api.genruleCreate({ genrule: 'pipe', name: `pipe-${Date.now()}`, length: 6, radius: 0.1 }); },
      });
    } else if (domain === 'mechanical') {
      actions.push({
        label: '钢梁 IPE 6m', icon: '━',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Beam-IPE', instance_name: `beam-${Date.now()}`, params: { height: 240, width: 120, web_thickness: 6.2, flange_thickness: 9.8, length: 6000 } }); },
      });
      actions.push({
        label: '钢柱 HEB 4m', icon: '▯',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Column-HEB', instance_name: `column-${Date.now()}`, params: { height: 300, width: 300, web_thickness: 11, flange_thickness: 19, column_height: 4000 } }); },
      });
      actions.push({
        label: 'CHS 管 Ø200 6m', icon: '◎',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-CHS', instance_name: `chs-${Date.now()}`, params: { diameter: 200, thickness: 8, length: 6000 } }); },
      });
      actions.push({
        label: '角钢 L100×10 6m', icon: '∟',
        action: async () => { await quickInstantiate({ component_def_id: 'Steel-Angle', instance_name: `angle-${Date.now()}`, params: { leg1: 100, leg2: 100, thickness: 10, length: 6000 } }); },
      });
      actions.push({
        label: '直齿轮 m4 z20', icon: '⦿',
        action: async () => { await quickInstantiate({ component_def_id: 'Gear', instance_name: `gear-${Date.now()}`, params: { module: 4, teeth: 20, width: 40 } }); },
      });
    }
    return actions;
  }, [domain]);

  async function handleQuickAction(action: QuickCreateAction) {
    setBusy(action.label);
    try {
      await action.action();
      const store = useAppStore.getState();
      await store.refresh();
      store.reloadScene();
    } catch (e) {
      alert(`创建失败: ${e instanceof Error ? e.message : String(e)}`);
    }
    setBusy(null);
  }

  function handleDragStart(e: React.DragEvent, comp: ComponentDef) {
    const params: Record<string, number> = {};
    comp.params.forEach((p) => { params[p.name] = p.default_value; });
    e.dataTransfer.setData('application/x-omnim-catalog', JSON.stringify({
      component_def_id: comp.id,
      instance_name: comp.name + '-' + Date.now(),
      params,
    }));
    e.dataTransfer.effectAllowed = 'copy';
  }

  function handleDragStartQuick(e: React.DragEvent, action: QuickCreateAction) {
    e.dataTransfer.setData('text/plain', action.label);
    e.dataTransfer.effectAllowed = 'copy';
  }

  async function handleInstantiate(comp: ComponentDef) {
    setBusy(comp.name);
    try {
      const params: Record<string, number> = {};
      comp.params.forEach((p) => { params[p.name] = p.default_value; });
      await quickInstantiate({
        component_def_id: comp.id,
        instance_name: comp.name + '-' + Date.now(),
        params,
      });
      const store = useAppStore.getState();
      await store.refresh();
      store.reloadScene();
    } catch (e) {
      alert(`实例化失败: ${e instanceof Error ? e.message : String(e)}`);
    }
    setBusy(null);
  }

  return (
    <div className="domain-panel">
      <div className="domain-panel-header">
        <span>{DOMAIN_LABELS[domain] || domain}</span>
        <button className="icon-btn" onClick={() => setOpen(domain, false)} title="Close"><IcoClose /></button>
      </div>

      {/* Quick-create buttons */}
      <div className="domain-panel-section">
        <label className="domain-panel-section-label">快速创建</label>
        <div className="domain-panel-actions">
          {quickActions.map((a) => (
            <button
              key={a.label}
              className="domain-panel-action-btn"
              disabled={busy === a.label}
              onClick={() => void handleQuickAction(a)}
            >
              <span className="domain-panel-action-icon">{a.icon}</span>
              <span className="domain-panel-action-label">{a.label}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Domain component list */}
      <div className="domain-panel-section">
        <label className="domain-panel-section-label">
          内置构件 ({filtered.length})
        </label>
        <div className="domain-panel-list">
          {loading && <div className="muted">加载中…</div>}
          {error && <div className="error-text">{error}</div>}
          {!loading && !error && filtered.length === 0 && <div className="muted">暂无构件</div>}
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
                title="添加到模型"
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

export function BuildingPanel() { return <DomainPanelContent domain="building" />; }
export function CivilPanel() { return <DomainPanelContent domain="civil" />; }
export function PlantPanel() { return <DomainPanelContent domain="plant" />; }
export function MechanicalPanel() { return <DomainPanelContent domain="mechanical" />; }