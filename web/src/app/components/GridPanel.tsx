import { useCallback, useEffect, useState } from 'react';
import { api } from '../../shared/api';
import { useAppStore } from '../store/appStore';
import { useI18n } from '../../shared/i18n';
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

type StoreyRow = { id: string; name: string; elevation: number };
type StoreyDraft = { id: string | null; name: string; elevation: string };

const EMPTY_STOREY: StoreyDraft = { id: null, name: '', elevation: '0' };

/** Grid datum panel (P8-S3): CRUD for 轴网 lines; the viewport re-renders on
 *  `reloadScene()` because ThreeViewer refetches `/api/grids` with the mesh. */
export function GridPanel() {
  const { t } = useI18n();
  const reloadScene = useAppStore((s) => s.reloadScene);
  const markDirty = useAppStore((s) => s.markDirty);
  const [lines, setLines] = useState<GridLine[]>([]);
  const [draft, setDraft] = useState<Draft>(EMPTY);
  const [storeys, setStoreys] = useState<StoreyRow[]>([]);
  const [storeyDraft, setStoreyDraft] = useState<StoreyDraft>(EMPTY_STOREY);
  const [storeyBusy, setStoreyBusy] = useState(false);
  const [storeyError, setStoreyError] = useState<string | null>(null);
  // F3 (UI/UX audit): a semantic toggle for the "extends through the
  // viewport" case instead of teaching users the 0/0 sentinel.
  const [fullSpan, setFullSpan] = useState(true);
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

  /** P8-S1/S4: levels live next to grids in the 轴网 tab (IA_SHELL §3.4).
   *  Storeys are datum-only objects — no geometry is created here. */
  const refreshStoreys = useCallback(async () => {
    try {
      const objects = await api.listObjects();
      setStoreys(
        objects
          .filter((o) => o.object_type === 3)
          .map((o) => ({ id: o.id, name: o.name, elevation: o.elevation ?? 0 }))
          .sort((a, b) => a.elevation - b.elevation),
      );
    } catch {
      setStoreys([]);
    }
  }, []);

  useEffect(() => { void refreshStoreys(); }, [refreshStoreys]);

  const submitStorey = async () => {
    const name = storeyDraft.name.trim();
    if (!name) return;
    setStoreyBusy(true);
    setStoreyError(null);
    const elevation = parseFloat(storeyDraft.elevation);
    try {
      if (storeyDraft.id) {
        await api.updateObject(storeyDraft.id, { name, elevation: Number.isFinite(elevation) ? elevation : 0 });
      } else {
        await api.createObject({
          object_type: 'storey',
          name,
          elevation: Number.isFinite(elevation) ? elevation : 0,
          ifc_class: 'IfcBuildingStorey',
        });
      }
      markDirty();
      setStoreyDraft(EMPTY_STOREY);
      await refreshStoreys();
      reloadScene();
    } catch (e) {
      setStoreyError(e instanceof Error ? e.message : String(e));
    } finally {
      setStoreyBusy(false);
    }
  };

  const removeStorey = async (id: string) => {
    setStoreyBusy(true);
    setStoreyError(null);
    try {
      await api.deleteObject(id);
      markDirty();
      if (storeyDraft.id === id) setStoreyDraft(EMPTY_STOREY);
      await refreshStoreys();
      reloadScene();
    } catch (e) {
      setStoreyError(e instanceof Error ? e.message : String(e));
    } finally {
      setStoreyBusy(false);
    }
  };

  const editStorey = (storey: StoreyRow) => {
    setStoreyDraft({
      id: storey.id,
      name: storey.name,
      elevation: String(storey.elevation),
    });
  };

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
      setFullSpan(true);
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
    setFullSpan(line.extent_min === 0 && line.extent_max === 0);
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
      <div className="grid-panel-section">
        <div className="grid-form-title">{t('grid.levels.title')}</div>
        <div className="grid-panel-list">
          {storeys.length === 0 && <div className="grid-empty">{t('grid.no_storeys')}</div>}
          {storeys.map((storey) => (
            <div key={storey.id} className="grid-line-row">
              <span className="grid-label">{storey.name}</span>
              <span className="grid-line-meta">{storey.elevation.toFixed(2)} m</span>
              <button type="button" className="grid-icon-btn" onClick={() => editStorey(storey)} disabled={storeyBusy}>{t('grid.btn.edit')}</button>
              <button type="button" className="grid-icon-btn danger" onClick={() => void removeStorey(storey.id)} disabled={storeyBusy} title={t('grid.btn.delete')}>×</button>
            </div>
          ))}
        </div>

        <form
          className="grid-form"
          onSubmit={(event) => {
            event.preventDefault();
            void submitStorey();
          }}
        >
          <div className="grid-form-title">{storeyDraft.id ? t('grid.edit_storey') : t('grid.add_storey')}</div>
          <label>
            <span>{t('grid.f.name')}</span>
            <input
              placeholder={t('grid.ph.f1')}
              value={storeyDraft.name}
              onChange={(e) => setStoreyDraft({ ...storeyDraft, name: e.target.value })}
            />
          </label>
          <label>
            <span>{t('grid.f.elevation')}</span>
            <input
              placeholder="0.00"
              value={storeyDraft.elevation}
              onChange={(e) => setStoreyDraft({ ...storeyDraft, elevation: e.target.value })}
            />
          </label>
          <div className="grid-form-actions">
            {storeyDraft.id && (
              <button type="button" onClick={() => setStoreyDraft(EMPTY_STOREY)} disabled={storeyBusy}>{t('grid.dlg.cancel')}</button>
            )}
            <button type="submit" className="primary" disabled={storeyBusy || !storeyDraft.name.trim()}>
              {storeyDraft.id ? t('grid.save_changes') : t('grid.add_storey')}
            </button>
          </div>
          <p className="grid-hint">{t('grid.levels.hint')}</p>
          {storeyError && <div className="grid-error">{storeyError}</div>}
        </form>
      </div>

      <div className="grid-divider" />
      <div className="grid-form-title">{t('grid.grids.title')}</div>

      <div className="grid-panel-list">
        {lines.length === 0 && <div className="grid-empty">{t('grid.no_grids')}</div>}
        {lines.map((line) => (
          <div key={line.id} className="grid-line-row">
            <span className={`grid-label ${line.axis === 'AlongX' ? 'axis-x' : 'axis-y'}`}>
              {line.label}
            </span>
            <span className="grid-line-meta">
              {line.axis === 'AlongX' ? 'X' : 'Y'} · {line.coordinate.toFixed(2)} m
            </span>
            <button type="button" className="grid-icon-btn" onClick={() => edit(line)} disabled={busy}>{t('grid.btn.edit')}</button>
            <button type="button" className="grid-icon-btn danger" onClick={() => void remove(line.id)} disabled={busy} title={t('grid.btn.delete')}>×</button>
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
        <div className="grid-form-title">{draft.id ? t('grid.edit_grid') : t('grid.add_grid')}</div>
        <label>
          <span>{t('grid.f.label')}</span>
          <input
            placeholder="A / 1"
            value={draft.label}
            onChange={(e) => setDraft({ ...draft, label: e.target.value })}
          />
        </label>
        <label>
          <span>{t('grid.f.direction')}</span>
          <select value={draft.axis} onChange={(e) => setDraft({ ...draft, axis: e.target.value as Draft['axis'] })}>
            <option value="AlongY">{t('grid.axis.y')}</option>
            <option value="AlongX">{t('grid.axis.x')}</option>
          </select>
        </label>
        <label>
          <span>{t('grid.f.coordinate')}</span>
          <input
            placeholder="0.00"
            value={draft.coordinate}
            onChange={(e) => setDraft({ ...draft, coordinate: e.target.value })}
          />
        </label>
        <div className="grid-form-extent">
          <label className="grid-fullspan-toggle">
            <input
              type="checkbox"
              checked={fullSpan}
              disabled={busy}
              onChange={(e) => {
                const on = e.target.checked;
                setFullSpan(on);
                if (on) setDraft({ ...draft, extent_min: '0', extent_max: '0' });
              }}
            />
            <span>{t('grid.span_full')}</span>
          </label>
          <label>
            <span>{t('grid.f.from')}</span>
            <input
              placeholder="0"
              disabled={fullSpan}
              value={draft.extent_min}
              onChange={(e) => setDraft({ ...draft, extent_min: e.target.value })}
            />
          </label>
          <label>
            <span>{t('grid.f.to')}</span>
            <input
              placeholder="0"
              disabled={fullSpan}
              value={draft.extent_max}
              onChange={(e) => setDraft({ ...draft, extent_max: e.target.value })}
            />
          </label>
        </div>
        <div className="grid-form-actions">
          {draft.id && (
            <button type="button" onClick={() => setDraft(EMPTY)} disabled={busy}>{t('grid.dlg.cancel')}</button>
          )}
          <button type="submit" className="primary" disabled={busy || !draft.label.trim()}>
            {draft.id ? t('grid.save_changes') : t('grid.add_grid')}
          </button>
        </div>
        {/* B2 (UI/UX audit): implementation detail → semantic copy. */}
        <p className="grid-hint">
          {t('grid.span_hint')}
        </p>
        {error && <div className="grid-error">{error}</div>}
      </form>
    </div>
  );
}
