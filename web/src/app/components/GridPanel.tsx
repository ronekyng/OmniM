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
    <div className="grid-panel">
      <div className="grid-panel-list">
        {lines.length === 0 && <div className="grid-empty">No grid lines yet.</div>}
        {lines.map((line) => (
          <div key={line.id} className="grid-line-row">
            <span className={`grid-label ${line.axis === 'AlongX' ? 'axis-x' : 'axis-y'}`}>
              {line.label}
            </span>
            <span className="grid-line-meta">
              {line.axis === 'AlongX' ? 'X' : 'Y'} · {line.coordinate.toFixed(2)} m
            </span>
            <button type="button" className="grid-icon-btn" onClick={() => edit(line)} disabled={busy}>Edit</button>
            <button type="button" className="grid-icon-btn danger" onClick={() => void remove(line.id)} disabled={busy} title="Delete">×</button>
          </div>
        ))}
      </div>

      <form
        className="grid-form"
        onSubmit={(event) => {
          event.preventDefault();
          void submit();
        }}
      >
        <div className="grid-form-title">{draft.id ? 'Edit grid line' : 'Add grid line'}</div>
        <label>
          <span>Label</span>
          <input
            placeholder="A / 1"
            value={draft.label}
            onChange={(e) => setDraft({ ...draft, label: e.target.value })}
          />
        </label>
        <label>
          <span>Direction</span>
          <select value={draft.axis} onChange={(e) => setDraft({ ...draft, axis: e.target.value as Draft['axis'] })}>
            <option value="AlongY">Y (letters)</option>
            <option value="AlongX">X (numbers)</option>
          </select>
        </label>
        <label>
          <span>Coordinate (m)</span>
          <input
            placeholder="0.00"
            value={draft.coordinate}
            onChange={(e) => setDraft({ ...draft, coordinate: e.target.value })}
          />
        </label>
        <div className="grid-form-extent">
          <label>
            <span>From (m)</span>
            <input
              placeholder="0"
              value={draft.extent_min}
              onChange={(e) => setDraft({ ...draft, extent_min: e.target.value })}
            />
          </label>
          <label>
            <span>To (m)</span>
            <input
              placeholder="0"
              value={draft.extent_max}
              onChange={(e) => setDraft({ ...draft, extent_max: e.target.value })}
            />
          </label>
        </div>
        <div className="grid-form-actions">
          {draft.id && (
            <button type="button" onClick={() => setDraft(EMPTY)} disabled={busy}>Cancel</button>
          )}
          <button type="submit" className="primary" disabled={busy || !draft.label.trim()}>
            {draft.id ? 'Save changes' : 'Add grid line'}
          </button>
        </div>
        <p className="grid-hint">0 / 0 extends the line across the viewport (±20 m).</p>
        {error && <div className="grid-error">{error}</div>}
      </form>
    </div>
  );
}
