import { useCallback, useEffect, useState } from 'react';
import { api } from '../../shared/api';
import { useAppStore } from '../store/appStore';
import type { GridLine } from '../../shared/types';

type Draft = {
  id: string | null;
  label: string;
  axis: 'AlongX' | 'AlongY';
  coordinate: string;
  extent_min: string;
  extent_max: string;
};

const EMPTY: Draft = {
  id: null,
  label: '',
  axis: 'AlongY',
  coordinate: '0',
  extent_min: '0',
  extent_max: '0',
};

/** Grid datum panel (P8-S3): CRUD for 轴网 lines; the viewport re-renders on
 *  `reloadScene()` because ThreeViewer refetches `/api/grids` with the mesh. */
export function GridPanel() {
  const reloadScene = useAppStore((s) => s.reloadScene);
  const markDirty = useAppStore((s) => s.markDirty);
  const [lines, setLines] = useState<GridLine[]>([]);
  const [draft, setDraft] = useState<Draft>(EMPTY);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLines(await api.grids());
    } catch {
      setLines([]);
    }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  const body = () => ({
    label: draft.label.trim(),
    axis: draft.axis === 'AlongX' ? 'along_x' : 'along_y',
    coordinate: parseFloat(draft.coordinate) || 0,
    extent_min: parseFloat(draft.extent_min) || 0,
    extent_max: parseFloat(draft.extent_max) || 0,
  });

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      if (draft.id) await api.updateGrid(draft.id, body());
      else await api.createGrid(body());
      markDirty();
      setDraft(EMPTY);
      await refresh();
      reloadScene();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (id: string) => {
    setBusy(true);
    setError(null);
    try {
      await api.deleteGrid(id);
      markDirty();
      if (draft.id === id) setDraft(EMPTY);
      await refresh();
      reloadScene();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const edit = (line: GridLine) => {
    setDraft({
      id: line.id,
      label: line.label,
      axis: line.axis,
      coordinate: String(line.coordinate),
      extent_min: String(line.extent_min),
      extent_max: String(line.extent_max),
    });
  };

  return (
    <div className="grid-panel" style={{ padding: 8, fontSize: 12, display: 'flex', flexDirection: 'column', gap: 6 }}>
      <div style={{ fontWeight: 600 }}>轴网 Grids</div>
      {lines.length === 0 && <div style={{ opacity: 0.6 }}>暂无轴线</div>}
      {lines.map((line) => (
        <div key={line.id} style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <span
            style={{
              fontWeight: 700,
              color: line.axis === 'AlongX' ? '#ff9f43' : '#54a0ff',
              minWidth: 18,
            }}
          >
            {line.label}
          </span>
          <span style={{ flex: 1, opacity: 0.75 }}>
            {line.axis === 'AlongX' ? 'X向' : 'Y向'} @ {line.coordinate.toFixed(2)} m
          </span>
          <button type="button" onClick={() => edit(line)} disabled={busy}>编辑</button>
          <button type="button" onClick={() => void remove(line.id)} disabled={busy}>×</button>
        </div>
      ))}

      <div style={{ borderTop: '1px solid rgba(255,255,255,0.12)', paddingTop: 6, display: 'flex', flexWrap: 'wrap', gap: 4 }}>
        <input
          placeholder="轴号 A/1"
          value={draft.label}
          onChange={(e) => setDraft({ ...draft, label: e.target.value })}
          style={{ width: 70 }}
        />
        <select value={draft.axis} onChange={(e) => setDraft({ ...draft, axis: e.target.value as Draft['axis'] })}>
          <option value="AlongY">Y向（字母）</option>
          <option value="AlongX">X向（数字）</option>
        </select>
        <input
          placeholder="坐标 m"
          value={draft.coordinate}
          onChange={(e) => setDraft({ ...draft, coordinate: e.target.value })}
          style={{ width: 62 }}
        />
        <input
          placeholder="起点"
          value={draft.extent_min}
          onChange={(e) => setDraft({ ...draft, extent_min: e.target.value })}
          style={{ width: 52 }}
        />
        <input
          placeholder="终点"
          value={draft.extent_max}
          onChange={(e) => setDraft({ ...draft, extent_max: e.target.value })}
          style={{ width: 52 }}
        />
        <button type="button" onClick={() => void submit()} disabled={busy || !draft.label.trim()}>
          {draft.id ? '保存修改' : '添加'}
        </button>
        {draft.id && <button type="button" onClick={() => setDraft(EMPTY)} disabled={busy}>取消</button>}
      </div>
      {error && <div style={{ color: '#ff6b6b' }}>{error}</div>}
      <div style={{ opacity: 0.55 }}>范围留 0/0 表示视口内无限延伸（±20 m）</div>
    </div>
  );
}
