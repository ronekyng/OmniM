import { useState } from 'react';
import { api } from '../../../shared/api';
import type { AssemblyComponentDef } from '../../../shared/types';
import { useToastApi } from '../../../shared/toast';

export function AssemblyPanel({ onCreated }: { onCreated: () => void }) {
  const toast = useToastApi();
  const [assayOpen, setAssayOpen] = useState(true);
  const [assyComps, setAssyComps] = useState<AssemblyComponentDef[]>([]);
  const [assyName, setAssyName] = useState('');
  const [editComp, setEditComp] = useState<AssemblyComponentDef>({
    name: 'Component', shape: 'box', p1: 2, p2: 1, p3: 1, tx: 0, ty: 0, tz: 0,
  });

  async function createAssembly() {
    const name = assyName || `assembly-${Date.now()}`;
    await api.genruleCreate({ genrule: 'assembly', name, components: assyComps });
    setAssayOpen(false);
    setAssyName('');
    toast.success(`Created assembly: ${name}`);
    onCreated();
  }

  if (!assayOpen) return null;

  return (
    <div className="create-panel">
      <div className="panel-header">
        <strong>Assembly — 组合多个部件</strong>
        <button className="panel-close" onClick={() => setAssayOpen(false)} title="Close">&times;</button>
      </div>
      <div className="create-row">
        <input placeholder="Assembly 名称" value={assyName} onChange={(e) => setAssyName(e.target.value)} />
      </div>

      {assyComps.length > 0 && (
        <div className="assy-list">
          <label className="field-label">部件列表 ({assyComps.length})</label>
          {assyComps.map((c, i) => (
            <div key={i} className="assy-item">
              <span>{c.name}</span>
              <span className="assy-item-detail">{c.shape} · {[c.p1, c.p2, c.p3].filter(v => v).join('×')}</span>
              <span className="assy-item-detail">pos: {c.tx},{c.ty},{c.tz}</span>
              <button className="assy-remove" onClick={() => setAssyComps(assyComps.filter((_, j) => j !== i))}>✕</button>
            </div>
          ))}
        </div>
      )}

      <div className="assy-add-form">
        <label className="field-label">添加部件</label>
        <div className="create-row">
          <input placeholder="名称" value={editComp.name} onChange={(e) => setEditComp({ ...editComp, name: e.target.value })} />
          <select value={editComp.shape} onChange={(e) => {
            const shape = e.target.value;
            const dims: Record<string, { p1: number; p2: number; p3: number }> = {
              box: { p1: 2, p2: 1, p3: 1 },
              cylinder: { p1: 0.5, p2: 3, p3: 0 },
              sphere: { p1: 1, p2: 0, p3: 0 },
              cone: { p1: 1, p2: 0.3, p3: 2 },
            };
            setEditComp({ ...editComp, shape, ...dims[shape] });
          }}>
            <option value="box">Box 方块</option>
            <option value="cylinder">Cylinder 圆柱</option>
            <option value="sphere">Sphere 球体</option>
            <option value="cone">Cone 圆锥</option>
          </select>
        </div>
        <div className="create-row">
          {(() => {
            const [a, b, c] = assyParamLabels(editComp.shape);
            return <>
              <input placeholder={a || '—'} value={editComp.p1 || ''} onChange={(e) => setEditComp({ ...editComp, p1: parseFloat(e.target.value) || 0 })} />
              <input placeholder={b || '—'} value={editComp.p2 || ''} onChange={(e) => setEditComp({ ...editComp, p2: parseFloat(e.target.value) || 0 })} />
              <input placeholder={c || '—'} value={editComp.p3 || ''} onChange={(e) => setEditComp({ ...editComp, p3: parseFloat(e.target.value) || 0 })} />
            </>;
          })()}
        </div>
        <div className="create-row">
          <input placeholder="X 偏移" value={editComp.tx || ''} onChange={(e) => setEditComp({ ...editComp, tx: parseFloat(e.target.value) || 0 })} />
          <input placeholder="Y 偏移" value={editComp.ty || ''} onChange={(e) => setEditComp({ ...editComp, ty: parseFloat(e.target.value) || 0 })} />
          <input placeholder="Z 偏移" value={editComp.tz || ''} onChange={(e) => setEditComp({ ...editComp, tz: parseFloat(e.target.value) || 0 })} />
        </div>
        <button className="primary" onClick={() => {
          setAssyComps([...assyComps, { ...editComp }]);
          setEditComp({ name: 'Component', shape: 'box', p1: 2, p2: 1, p3: 1, tx: 0, ty: 0, tz: 0 });
        }}>+ 添加到列表</button>
      </div>

      {assyComps.length > 0 && (
        <button className="primary create-btn" onClick={() => void createAssembly().catch((err) => toast.error(`Assembly: ${err instanceof Error ? err.message : String(err)}`))}>
          创建 Assembly ({assyComps.length} 个部件)
        </button>
      )}
    </div>
  );
}

function assyParamLabels(shape: string): [string, string, string] {
  switch (shape) {
    case 'cylinder': return ['半径', '高度', ''];
    case 'sphere':   return ['半径', '', ''];
    case 'cone':     return ['底半径', '顶半径', '高度'];
    default:         return ['宽', '高', '深'];
  }
}