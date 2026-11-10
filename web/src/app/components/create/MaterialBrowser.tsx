import { useEffect, useMemo, useState } from 'react';
import { api } from '../../../shared/api';
import type { Material } from '../../../shared/types';
import { useI18n } from '../../../shared/i18n';
import { MATERIAL_CATEGORY_KEYS } from '../../../shared/types';
import { IcoStar } from '../Icons';

type Props = {
  selectedId?: string;
  onSelect: (material: Material) => void;
  onClose?: () => void;
  filterCategory?: number;
};

/** Browse the material library from the backend (/api/materials).
 *  Supports category browsing, search, and selection (dev plan §2.2). */
export function MaterialBrowser({ selectedId, onSelect, onClose, filterCategory }: Props) {
  const { t } = useI18n();
  const [materials, setMaterials] = useState<Material[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState<number>(filterCategory ?? -1);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());

  useEffect(() => {
    try {
      setFavorites(new Set(JSON.parse(localStorage.getItem('omnim.material.favorites') || '[]')));
    } catch { /* ignore */ }
    api.listMaterials()
      .then((list) => setMaterials(list))
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, []);

  const categories = useMemo(() => {
    const set = new Set<number>();
    materials.forEach((m) => set.add(m.category));
    return [...set].sort((a, b) => a - b);
  }, [materials]);

  const filtered = useMemo(() => materials.filter((m) => {
    if (category >= 0 && m.category !== category) return false;
    if (filterCategory !== undefined && m.category !== filterCategory) return false;
    if (search) {
      const q = search.toLowerCase();
      if (!m.name.toLowerCase().includes(q) && !m.id.toLowerCase().includes(q)) return false;
    }
    return true;
  }), [materials, category, search, filterCategory]);

  const toggleFavorite = (id: string) => {
    const next = new Set(favorites);
    next.has(id) ? next.delete(id) : next.add(id);
    setFavorites(next);
    localStorage.setItem('omnim.material.favorites', JSON.stringify([...next]));
  };

  const catLabel = (c: number): string => {
    const k = MATERIAL_CATEGORY_KEYS[c];
    return k ? t(k as never) : t('mat.uncategorized', { cat: c });
  };

  return (
    <div className="material-browser">
      <div className="material-browser-header">
        <h3>{t('mat.title')}</h3>
        {onClose && <button className="material-browser-close" onClick={onClose} title={t('mat.close')}>×</button>}
      </div>

      <div className="material-browser-toolbar">
        <input
          className="material-browser-search"
          type="text"
          placeholder={t('mat.search_ph')}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        <select
          className="material-browser-cat"
          value={category}
          onChange={(e) => setCategory(parseInt(e.target.value, 10))}
        >
          <option value={-1}>{t('mat.all')}</option>
          {categories.map((c) => (
            <option key={c} value={c}>{catLabel(c)}</option>
          ))}
        </select>
      </div>

      {loading && <div className="material-browser-loading">{t('mat.loading')}</div>}
      {error && <div className="material-browser-error">{error}</div>}

      {!loading && !error && (
        <div className="material-browser-list">
          {filtered.length === 0 && <div className="material-browser-empty">{t('mat.none')}</div>}
          {filtered.map((m) => {
            const isSel = m.id === selectedId;
            const isFav = favorites.has(m.id);
            return (
              <div
                key={m.id}
                className={`material-card ${isSel ? 'selected' : ''}`}
                onClick={() => onSelect(m)}
              >
                <div className="material-card-top">
                  <span className="material-swatch" style={{
                    background: m.color ? `rgb(${Math.round(m.color.r * 255)},${Math.round(m.color.g * 255)},${Math.round(m.color.b * 255)})` : '#999',
                  }} />
                  <span className="material-card-name">{m.name}</span>
                  <button
                    className={`material-fav-btn ${isFav ? 'active' : ''}`}
                    onClick={(e) => { e.stopPropagation(); toggleFavorite(m.id); }}
                    title={isFav ? t('mat.fav_remove') : t('mat.fav_add')}
                  >
                    <IcoStar filled={isFav} />
                  </button>
                </div>
                <div className="material-card-cat">{catLabel(m.category)}</div>
                <div className="material-card-props">
                  {m.density > 0 && <span>ρ={m.density.toFixed(0)} kg/m³</span>}
                  {m.thermal_conductivity > 0 && <span>λ={m.thermal_conductivity.toFixed(2)} W/mK</span>}
                  {m.youngs_modulus > 0 && <span>E={fmtExp(m.youngs_modulus)}</span>}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function fmtExp(v: number): string {
  if (v >= 1e9) return `${(v / 1e9).toFixed(1)}GPa`;
  if (v >= 1e6) return `${(v / 1e6).toFixed(0)}MPa`;
  return `${v.toFixed(0)}Pa`;
}
