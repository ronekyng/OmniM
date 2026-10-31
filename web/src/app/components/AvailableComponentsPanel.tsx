import { useEffect, useState, useCallback } from 'react';
import { api } from '../../shared/api';
import { notify } from '../../shared/toast';
import { nextAutoName } from '../../shared/naming';
import type { ComponentDef, ParamDef, Series } from '../../shared/types';
import { IcoClose } from './Icons';
import { useAppStore } from '../store/appStore';
import { useCatalog, ensureCatalogLoaded, isFavorite, toggleFavorite } from '../../shared/catalog';
import { ComponentParamPanel } from './create/ComponentParamPanel';
import { MaterialBrowser } from './create/MaterialBrowser';
import { SkeletonLines } from './Loading';
import { useDelayedLoading } from '../../shared/useDelayedLoading';

type ParamValues = Record<string, number>;

export function AvailableComponentsPanel({ embedded = false }: { embedded?: boolean } = {}) {
  const setCatalogPanelOpen = useAppStore((s) => s.setCatalogPanelOpen);
  const panelMode = useAppStore((s) => s.panelMode);
  const markDirty = useAppStore((s) => s.markDirty);
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);

  const [seriesList, setSeriesList] = useState<Series[]>([]);
  const { status, catalog, error: catalogError } = useCatalog();
  const [loading, setLoading] = useState(true);
  const [catalogReload, setCatalogReload] = useState(0);
  const showSkeleton = useDelayedLoading(loading);
  const [error, setError] = useState('');
  const [selectedSeries, setSelectedSeries] = useState<string>('');
  const [searchText, setSearchText] = useState('');
  const [selectedComp, setSelectedComp] = useState<ComponentDef | null>(null);
  const [paramValues, setParamValues] = useState<ParamValues>({});
  const [instantiating, setInstantiating] = useState(false);
  const [materialFilter, setMaterialFilter] = useState<string>('');
  const [domainFilter, setDomainFilter] = useState<string>('');
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [favVersion, setFavVersion] = useState(0);
  const [showMaterials, setShowMaterials] = useState(false);

  const [objectNames, setObjectNames] = useState<string[]>([]);
  useEffect(() => {
    api.listObjects()
      .then((list) => setObjectNames(list.map((o) => o.name)))
      .catch(() => { /* names only power auto-naming; keep last list */ });
    ensureCatalogLoaded(catalogReload > 0)
      .then(() => setLoading(false))
      .catch(() => setLoading(false));
    api.listSeries()
      .then((list) => setSeriesList(list))
      .catch((err: unknown) =>
        notify.error(`系列列表加载失败: ${err instanceof Error ? err.message : String(err)}`),
      );
  }, [catalogReload]);

  const DOMAIN_LABELS: Record<string, string> = {
    structural: '建筑结构',
    building: '建筑',
    civil: '土木',
    plant: '工厂',
    mechanical: '机械',
    piping: '管道',
  };

  const domains = [...new Set(catalog.map((c) => c.domain))].sort();

  const filtered = catalog.filter((c) => {
    if (favoritesOnly && !isFavorite(c.id)) return false;
    if (domainFilter && c.domain !== domainFilter) return false;
    if (selectedSeries) {
      const series = seriesList.find((s) => s.id === selectedSeries);
      if (series && c.domain !== series.domain) return false;
    }
    if (searchText) {
      const q = searchText.toLowerCase();
      if (!c.name.toLowerCase().includes(q) &&
          !c.category.toLowerCase().includes(q) &&
          !c.tags.some((t) => t.toLowerCase().includes(q)) &&
          !c.description.toLowerCase().includes(q)) return false;
    }
    if (materialFilter) {
      if (!c.allowed_materials.includes(materialFilter)) return false;
    }
    return true;
  });

  const allMaterials = new Set<string>();
  filtered.forEach((c) => c.allowed_materials.forEach((m) => allMaterials.add(m)));

  const handleSelectComp = useCallback((comp: ComponentDef) => {
    setSelectedComp(comp);
    const vals: ParamValues = {};
    comp.params.forEach((p) => {
      vals[p.name] = p.default_value;
    });
    setParamValues(vals);
  }, []);

  const handleParamChange = useCallback((name: string, value: number) => {
    setParamValues((prev) => ({ ...prev, [name]: value }));
  }, []);

  const handleDragStart = useCallback((e: React.DragEvent, comp: ComponentDef) => {
    const params: Record<string, number> = {};
    comp.params.forEach((p) => { params[p.name] = p.default_value; });
    e.dataTransfer.setData('application/x-omnim-catalog', JSON.stringify({
      component_def_id: comp.id,
      instance_name: nextAutoName(comp.name.toLowerCase().replace(/[^a-z0-9]+/g, '-'), objectNames),
      params,
    }));
    e.dataTransfer.effectAllowed = 'copy';
  }, []);

  const handleInstantiate = async () => {
    if (!selectedComp) return;
    setInstantiating(true);
    try {
      await api.instantiateFromCatalog({
        component_def_id: selectedComp.id,
        instance_name: nextAutoName(selectedComp.name.toLowerCase().replace(/[^a-z0-9]+/g, '-'), objectNames),
        params: paramValues,
        skip_series_gate: !selectedSeries,
      });
      markDirty();
      refresh();
      reloadScene();
      setCatalogPanelOpen(false);
    } catch (e) {
      notify.error('实例化失败: ' + (e instanceof Error ? e.message : String(e)));
    }
    setInstantiating(false);
  };

  const modeClass = embedded
    ? 'avail-comp--embedded'
    : panelMode === 'floating' ? 'avail-comp--floating' : 'avail-comp--docked';
  const loadError = error || catalogError || '';

  return (
    <div className={`avail-comp-panel ${modeClass}`}>
      <div className="avail-comp-header">
        <h2>可用构件</h2>
        <div className="avail-comp-header-actions">
          <button
            className={showMaterials ? 'active' : ''}
            onClick={() => setShowMaterials((v) => !v)}
            title="材料库"
          >
            材料库
          </button>
          {!embedded && (
            <button onClick={() => setCatalogPanelOpen(false)}><IcoClose /></button>
          )}
        </div>
      </div>

      {showMaterials ? (
        <MaterialBrowser
          onSelect={() => {}}
        />
      ) : (
        <>
          {loading && showSkeleton && (
            <div aria-busy="true" aria-label="Loading catalog">
              <SkeletonLines count={6} />
            </div>
          )}
          {loadError && (
            <div className="avail-comp-error" role="alert">
              <span>{loadError}</span>
              <button type="button" onClick={() => setCatalogReload((n) => n + 1)}>Retry</button>
            </div>
          )}

          {!loading && !loadError && (
            <div className="avail-comp-body">
              <div className="avail-comp-sidebar">
                <div className="avail-comp-filters">
                  <div className="avail-comp-fav-row">
                    <label className="avail-comp-fav-toggle">
                      <input
                        type="checkbox"
                        checked={favoritesOnly}
                        onChange={(e) => { setFavoritesOnly(e.target.checked); setFavVersion((v) => v + 1); }}
                      />
                      只看收藏
                    </label>
                  </div>
                  <select
                    className="avail-comp-select"
                    value={domainFilter}
                    onChange={(e) => { setDomainFilter(e.target.value); setMaterialFilter(''); }}
                  >
                    <option value="">所有领域</option>
                    {domains.map((d) => (
                      <option key={d} value={d}>{DOMAIN_LABELS[d] || d}</option>
                    ))}
                  </select>

                  <select
                    className="avail-comp-select"
                    value={selectedSeries}
                    onChange={(e) => setSelectedSeries(e.target.value)}
                  >
                    <option value="">所有系列</option>
                    {seriesList.map((s) => (
                      <option key={s.id} value={s.id}>{s.name} ({s.domain})</option>
                    ))}
                  </select>

                  <input
                    className="avail-comp-search"
                    type="text"
                    placeholder="搜索构件名称、类别、标签..."
                    value={searchText}
                    onChange={(e) => setSearchText(e.target.value)}
                  />

                  {allMaterials.size > 0 && (
                    <select
                      className="avail-comp-select"
                      value={materialFilter}
                      onChange={(e) => setMaterialFilter(e.target.value)}
                    >
                      <option value="">所有材质</option>
                      {[...allMaterials].map((m) => (
                        <option key={m} value={m}>{m}</option>
                      ))}
                    </select>
                  )}
                </div>

                <div className="avail-comp-list">
                  {filtered.length === 0 && (
                    <div className="avail-comp-empty">无匹配构件</div>
                  )}
                  {filtered.map((c) => {
                    const fav = isFavorite(c.id);
                    return (
                      <div
                        key={c.id}
                        className={`avail-comp-card ${selectedComp?.id === c.id ? 'selected' : ''}`}
                        onClick={() => handleSelectComp(c)}
                        draggable
                        onDragStart={(e) => handleDragStart(e, c)}
                      >
                        <div className="avail-comp-card-top">
                          <span className="avail-comp-card-name">{c.name}</span>
                          <button
                            className={`avail-comp-fav ${fav ? 'active' : ''}`}
                            onClick={(e) => { e.stopPropagation(); toggleFavorite(c.id); setFavVersion((v) => v + 1); }}
                            title={fav ? '取消收藏' : '收藏'}
                          >
                            {fav ? '★' : '☆'}
                          </button>
                        </div>
                        <div className="avail-comp-card-meta">{c.category} &middot; {c.ifc_class}</div>
                        <div className="avail-comp-card-desc">{c.description}</div>
                        <div className="avail-comp-card-tags">{c.tags.join(', ')}</div>
                        <div className="avail-comp-card-params">{c.params.length} 参数</div>
                      </div>
                    );
                  })}
                </div>
              </div>

              <div className="avail-comp-detail">
                {selectedComp ? (
                  <>
                    <h3 className="avail-comp-detail-title">{selectedComp.name}</h3>
                    <div className="avail-comp-detail-meta">
                      <span>类别: {selectedComp.category}</span>
                      <span>IFC: {selectedComp.ifc_class}</span>
                      <span>标准: {selectedComp.standard_reference}</span>
                    </div>
                    <p className="avail-comp-detail-desc">{selectedComp.description}</p>

                    {selectedComp.allowed_materials.length > 0 && (
                      <div className="avail-comp-detail-section">
                        <label>材质</label>
                        <select
                          className="avail-comp-select"
                          value={selectedComp.default_material}
                          onChange={(e) => {}}
                        >
                          {selectedComp.allowed_materials.map((m) => (
                            <option key={m} value={m}>{m}</option>
                          ))}
                        </select>
                      </div>
                    )}

                    {selectedComp.params.length > 0 && (
                      <div className="avail-comp-detail-section">
                        <label>参数</label>
                        <ComponentParamPanel
                          params={selectedComp.params}
                          values={paramValues}
                          onChange={handleParamChange}
                        />
                      </div>
                    )}

                    <button
                      className="avail-comp-instantiate-btn"
                      onClick={handleInstantiate}
                      disabled={instantiating}
                    >
                      {instantiating ? '创建中...' : '添加到模型'}
                    </button>
                  </>
                ) : (
                  <div className="avail-comp-detail-empty">
                    选择一个构件查看参数
                  </div>
                )}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
}