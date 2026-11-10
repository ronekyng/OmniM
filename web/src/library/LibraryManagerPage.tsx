import { useEffect, useState, useCallback, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useI18n } from '../shared/i18n';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { api } from '../shared/api';
import { notify } from '../shared/toast';
import { useConfirm } from '../app/components/ConfirmDialog';
import { PageBreadcrumb } from '../app/components/PageBreadcrumb';
import { IcoChevronDown, IcoChevronRight, IcoDelete, IcoEdit, IcoSearch } from '../app/components/Icons';
import type { ComponentDef, ParamDef, CatalogPreviewMesh } from '../shared/types';

export function LibraryManagerPage() {
  const navigate = useNavigate();
  const confirm = useConfirm();
  const { t } = useI18n();
  const [catalog, setCatalog] = useState<ComponentDef[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState('');
  const [editCategory, setEditCategory] = useState('');
  const [editDesc, setEditDesc] = useState('');
  const [editIfc, setEditIfc] = useState('');
  const [editStd, setEditStd] = useState('');
  const [editDomain, setEditDomain] = useState('');
  const [editTags, setEditTags] = useState('');
  const [editMaterials, setEditMaterials] = useState('');
  const [editDefaultMat, setEditDefaultMat] = useState('');
  const [newParamName, setNewParamName] = useState('');
  const [newParamDefault, setNewParamDefault] = useState('0');
  const [newParamMin, setNewParamMin] = useState('0');
  const [newParamMax, setNewParamMax] = useState('1000');
  const [newParamStep, setNewParamStep] = useState('0.1');
  const [newParamUnit, setNewParamUnit] = useState('');
  const [editingParamIdx, setEditingParamIdx] = useState<number | null>(null);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [createName, setCreateName] = useState('');
  const [createCategory, setCreateCategory] = useState('');
  const [importPath, setImportPath] = useState('');
  const [importMsg, setImportMsg] = useState('');
  const [exportMsg, setExportMsg] = useState('');
  const [previewMesh, setPreviewMesh] = useState<CatalogPreviewMesh | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState('');
  const [editFeatureTree, setEditFeatureTree] = useState('');
  const [domainFilter, setDomainFilter] = useState<string>('');

  const loadCatalog = useCallback(async () => {
    try {
      const list = await api.listCatalog();
      setCatalog(list);
      setError('');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { loadCatalog(); }, [loadCatalog]);

  const selectedComp = catalog.find(c => c.id === selectedId) || null;

  const domainLabelOf = (d: string): string => {
    const key = `domain.${d}`;
    const v = t(key as never);
    return v !== key ? v : d;
  };

  const domains = [...new Set(catalog.map(c => c.domain))].sort();
  const filteredCatalog = domainFilter ? catalog.filter(c => c.domain === domainFilter) : catalog;
  const categories = [...new Set(filteredCatalog.map(c => c.category))].sort();

  const handleSelect = (id: string) => {
    setSelectedId(id);
    setEditing(false);
    setEditingParamIdx(null);
  };

  const handleStartEdit = () => {
    if (!selectedComp) return;
    setEditName(selectedComp.name);
    setEditCategory(selectedComp.category);
    setEditDesc(selectedComp.description);
    setEditIfc(selectedComp.ifc_class);
    setEditStd(selectedComp.standard_reference);
    setEditDomain(selectedComp.domain);
    setEditTags(selectedComp.tags.join(', '));
    setEditMaterials(selectedComp.allowed_materials.join(', '));
    setEditDefaultMat(selectedComp.default_material);
    setEditFeatureTree(selectedComp.feature_tree_id);
    setEditing(true);
  };

  const handlePreview = async () => {
    if (!selectedId) return;
    setPreviewLoading(true);
    setPreviewError('');
    setPreviewMesh(null);
    try {
      const params: Record<string, number> = {};
      if (selectedComp) {
        selectedComp.params.forEach(p => { params[p.name] = p.default_value; });
      }
      const mesh = await api.previewCatalogComponent({ component_def_id: selectedId, params });
      setPreviewMesh(mesh);
    } catch (e) {
      setPreviewError(`${t('library.preview_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      setPreviewLoading(false);
    }
  };

  const handleSaveEdit = async () => {
    if (!selectedId) return;
    try {
      await api.updateComponentDef(selectedId, {
        name: editName,
        category: editCategory,
        description: editDesc,
        ifc_class: editIfc,
        standard_reference: editStd,
        domain: editDomain,
        tags: editTags.split(',').map(t => t.trim()).filter(Boolean),
        allowed_materials: editMaterials.split(',').map(m => m.trim()).filter(Boolean),
        default_material: editDefaultMat,
      });
      if (editFeatureTree) {
        await api.assignFeatureTree(selectedId, editFeatureTree);
      }
      setEditing(false);
      setPreviewMesh(null);
      loadCatalog();
    } catch (e) {
      notify.error(`${t('library.save_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleCreate = async () => {
    if (!createName || !createCategory) { notify.error(t('library.name_category_required')); return; }
    try {
      await api.createComponentDef({ name: createName, category: createCategory });
      setCreateName('');
      setCreateCategory('');
      setShowCreateForm(false);
      loadCatalog();
    } catch (e) {
      notify.error(`${t('library.create_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleDelete = async (id: string) => {
    if (!(await confirm({ title: 'Delete this component definition?', confirmLabel: 'Delete', danger: true }))) return;
    try {
      await api.deleteComponentDef(id);
      if (selectedId === id) setSelectedId(null);
      loadCatalog();
    } catch (e) {
      notify.error(`${t('library.delete_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleAddParam = async () => {
    if (!selectedId || !newParamName) return;
    try {
      await api.addComponentParam(selectedId, {
        name: newParamName,
        default_value: parseFloat(newParamDefault) || 0,
        min_value: parseFloat(newParamMin) || 0,
        max_value: parseFloat(newParamMax) || 1000,
        step: parseFloat(newParamStep) || 0.1,
        unit: newParamUnit,
      });
      setNewParamName('');
      setNewParamDefault('0');
      setNewParamMin('0');
      setNewParamMax('1000');
      setNewParamStep('0.1');
      setNewParamUnit('');
      loadCatalog();
    } catch (e) {
      notify.error(`${t('library.add_param_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleUpdateParam = async (idx: number, param: ParamDef) => {
    if (!selectedId) return;
    try {
      await api.updateComponentParam(selectedId, idx, {
        name: param.name,
        default_value: param.default_value,
        min_value: param.min_value,
        max_value: param.max_value,
        step: param.step,
        unit: param.unit,
      });
      setEditingParamIdx(null);
      loadCatalog();
    } catch (e) {
      notify.error(`${t('library.update_param_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleDeleteParam = async (idx: number) => {
    if (!selectedId || !(await confirm({ title: 'Delete this parameter?', confirmLabel: 'Delete', danger: true }))) return;
    try {
      await api.deleteComponentParam(selectedId, idx);
      loadCatalog();
    } catch (e) {
      notify.error(`${t('library.delete_param_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleExport = async () => {
    try {
      const res = await api.exportCatalog();
      setExportMsg(t('library.export_ok', { path: res.path, count: res.component_count }));
    } catch (e) {
      setExportMsg(`${t('library.export_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const handleImport = async () => {
    if (!importPath) { notify.error(t('library.path_required')); return; }
    try {
      const res = await api.importCatalog(importPath);
      setImportMsg(t('library.import_ok', { count: res.component_count, names: res.names.join(', ') }));
      setImportPath('');
      loadCatalog();
    } catch (e) {
      setImportMsg(`${t('library.import_failed')}: ${e instanceof Error ? e.message : String(e)}`);
    }
  };

  const toggleCategory = (cat: string) => {
    setExpandedCategories(prev => {
      const next = new Set(prev);
      if (next.has(cat)) next.delete(cat);
      else next.add(cat);
      return next;
    });
  };

  return (
    <div className="library-mgr-page">
      <PageBreadcrumb trail={['crumb.projects', 'crumb.editors', 'Library Manager']} />
      <div className="library-mgr-header">
        <h1>Library Manager</h1>
        <div className="library-mgr-header-actions">
          <button className="library-mgr-btn" onClick={() => setShowCreateForm(!showCreateForm)}>{t('library.btn.new')}</button>
          <button className="library-mgr-btn" onClick={handleExport}>{t('library.btn.export')}</button>
          <button className="library-mgr-back" onClick={() => navigate('/')}>{t('library.btn.back')}</button>
        </div>
      </div>

      {showCreateForm && (
        <div className="library-mgr-create-form">
          <input placeholder={t('library.ph.name')} value={createName} onChange={e => setCreateName(e.target.value)} />
          <input placeholder={t('library.ph.category')} value={createCategory} onChange={e => setCreateCategory(e.target.value)} />
          <button className="library-mgr-btn" onClick={handleCreate}>{t('library.btn.create')}</button>
          <button className="library-mgr-btn" onClick={() => setShowCreateForm(false)}>{t('library.btn.cancel')}</button>
        </div>
      )}

      <div className="library-mgr-io-bar">
        <input placeholder={t('library.ph.import_path')} value={importPath} onChange={e => setImportPath(e.target.value)} />
        <button className="library-mgr-btn" onClick={handleImport}>{t('library.btn.import')}</button>
        {importMsg && <span className="library-mgr-io-msg">{importMsg}</span>}
        {exportMsg && <span className="library-mgr-io-msg">{exportMsg}</span>}
      </div>

      {error && <div className="library-mgr-error">{error}</div>}

      <div className="library-mgr-body">
        <div className="library-mgr-sidebar">
          <h3>{t('library.catalog.title')}</h3>
          <div className="library-mgr-domain-tabs">
            <button
              className={`library-mgr-domain-tab ${domainFilter === '' ? 'active' : ''}`}
              onClick={() => setDomainFilter('')}
            >{t('library.filter.all')}</button>
            {domains.map(d => (
              <button
                key={d}
                className={`library-mgr-domain-tab ${domainFilter === d ? 'active' : ''}`}
                onClick={() => setDomainFilter(d)}
              >{domainLabelOf(d)}</button>
            ))}
          </div>
          {loading && <div className="library-mgr-loading">{t('library.list.loading')}</div>}
          {!loading && filteredCatalog.length === 0 && <div className="library-mgr-empty">{t('library.list.empty')}</div>}
          {categories.map(cat => (
            <div key={cat} className="library-mgr-category">
              <div className="library-mgr-cat-header" onClick={() => toggleCategory(cat)}>
                <span className="library-mgr-cat-arrow">{expandedCategories.has(cat) ? <IcoChevronDown /> : <IcoChevronRight />}</span>
                <span className="library-mgr-cat-name">{cat}</span>
                <span className="library-mgr-cat-count">({filteredCatalog.filter(c => c.category === cat).length})</span>
              </div>
              {expandedCategories.has(cat) && filteredCatalog
                .filter(c => c.category === cat)
                .map(c => (
                  <div
                    key={c.id}
                    className={`library-mgr-item ${selectedId === c.id ? 'selected' : ''} ${c.published ? 'published' : ''}`}
                    onClick={() => handleSelect(c.id)}
                  >
                    <div className="library-mgr-item-name">{c.name}</div>
                    <div className="library-mgr-item-meta">{c.ifc_class} · {c.params.length} params</div>
                  </div>
                ))}
            </div>
          ))}
        </div>

        <div className="library-mgr-content">
          {selectedComp && !editing ? (
            <div className="library-mgr-detail">
              <div className="library-mgr-detail-header">
                <h2>{selectedComp.name}</h2>
                <span className={`library-mgr-badge ${selectedComp.published ? 'published' : 'draft'}`}>
                  {selectedComp.published ? t('library.badge.published') : t('library.badge.draft')}
                </span>
                <div className="library-mgr-detail-actions">
                  <button className="library-mgr-btn" onClick={handleStartEdit}>{t('library.btn.edit')}</button>
                  <button className="library-mgr-btn" onClick={() => handleDelete(selectedComp.id)}>{t('library.btn.delete')}</button>
                </div>
              </div>

              <div className="library-mgr-detail-grid">
                <div className="library-mgr-detail-field">
                  <label>{t('library.field.category')}</label><span>{selectedComp.category}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>{t('library.field.ifc')}</label><span>{selectedComp.ifc_class}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>{t('library.field.standard')}</label><span>{selectedComp.standard_reference}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>{t('library.field.domain')}</label><span>{selectedComp.domain}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>{t('library.field.feature_tree')}</label><span>{selectedComp.feature_tree_id || t('library.none')}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>{t('library.field.default_material')}</label><span>{selectedComp.default_material || t('library.none')}</span>
                </div>
              </div>

              <div className="library-mgr-detail-actions">
                <button className="library-mgr-btn" onClick={handlePreview} disabled={previewLoading}>
                  {previewLoading ? t('library.preview.generating') : <><IcoSearch /> {t('library.preview.open')}</>}
                </button>
              </div>
              {previewError && <div className="library-mgr-error">{previewError}</div>}
              {previewMesh && (
                <div className="library-mgr-preview">
                  <PreviewCanvas mesh={previewMesh} />
                </div>
              )}

              <p className="library-mgr-detail-desc">{selectedComp.description}</p>

              {selectedComp.tags.length > 0 && (
                <div className="library-mgr-tags">
                  {selectedComp.tags.map(t => <span key={t} className="library-mgr-tag">{t}</span>)}
                </div>
              )}

              {selectedComp.allowed_materials.length > 0 && (
                <div className="library-mgr-section">
                  <h4>{t('library.materials.title')}</h4>
                  <div className="library-mgr-materials">
                    {selectedComp.allowed_materials.map(m => <span key={m} className="library-mgr-mat">{m}</span>)}
                  </div>
                </div>
              )}

              <div className="library-mgr-section">
                <h4>{t('library.params.title', { count: selectedComp.params.length })}</h4>
                <div className="library-mgr-params">
                  {selectedComp.params.map((p, idx) => (
                    <div key={p.name + idx} className="library-mgr-param-card">
                      {editingParamIdx === idx ? (
                        <div className="library-mgr-param-edit">
                          <input value={p.name} onChange={e => {
                            const updated = [...selectedComp.params];
                            updated[idx] = { ...updated[idx], name: e.target.value };
                          }} />
                          <input type="number" value={p.default_value} onChange={e => {
                            // Can't directly update via local state, reload
                          }} />
                          <button className="library-mgr-btn" onClick={() => handleUpdateParam(idx, p)}>{t('library.btn.save')}</button>
                          <button className="library-mgr-btn" onClick={() => setEditingParamIdx(null)}>{t('library.btn.cancel')}</button>
                        </div>
                      ) : (
                        <>
                          <div className="library-mgr-param-header">
                            <span className="library-mgr-param-name">{p.name}</span>
                            <span className="library-mgr-param-default">{p.default_value} {p.unit}</span>
                            <div className="library-mgr-param-actions">
                              <button className="library-mgr-btn-sm" onClick={() => setEditingParamIdx(idx)} title={t('library.param.edit')} aria-label={t('library.param.edit')}><IcoEdit /></button>
                              <button className="library-mgr-btn-sm danger" onClick={() => handleDeleteParam(idx)} title={t('library.param.delete')} aria-label={t('library.param.delete')}><IcoDelete /></button>
                            </div>
                          </div>
                          <div className="library-mgr-param-range">
                            {t('library.param.range', { min: p.min_value, max: p.max_value, step: p.step })}
                          </div>
                        </>
                      )}
                    </div>
                  ))}
                </div>

                <div className="library-mgr-add-param">
                  <input placeholder={t('library.param.ph_name')} value={newParamName} onChange={e => setNewParamName(e.target.value)} />
                  <input type="number" placeholder={t('library.param.ph_default')} value={newParamDefault} onChange={e => setNewParamDefault(e.target.value)} />
                  <input type="number" placeholder={t('library.param.ph_min')} value={newParamMin} onChange={e => setNewParamMin(e.target.value)} />
                  <input type="number" placeholder={t('library.param.ph_max')} value={newParamMax} onChange={e => setNewParamMax(e.target.value)} />
                  <input type="number" placeholder={t('library.param.ph_step')} value={newParamStep} onChange={e => setNewParamStep(e.target.value)} />
                  <input placeholder={t('library.param.ph_unit')} value={newParamUnit} onChange={e => setNewParamUnit(e.target.value)} />
                  <button className="library-mgr-btn" onClick={handleAddParam}>{t('library.param.add')}</button>
                </div>
              </div>

              <div className="library-mgr-section">
                <h4>{t('library.connectors.title', { count: selectedComp.connectors.length })}</h4>
                {selectedComp.connectors.length === 0 ? (
                  <div className="library-mgr-empty">{t('library.connectors.none')}</div>
                ) : (
                  selectedComp.connectors.map(c => (
                    <div key={c.id} className="library-mgr-connector-card">
                      <div className="library-mgr-connector-name">{c.name} ({c.id})</div>
                      <div className="library-mgr-connector-detail">
                        {t('library.connectors.pos', { x: c.position_x_expr, y: c.position_y_expr, z: c.position_z_expr })}
                        {t('library.connectors.dir', { x: c.direction_x_expr, y: c.direction_y_expr, z: c.direction_z_expr })}
                        {t('library.connectors.type', { code: c.connection_type_code })}
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          ) : selectedComp && editing ? (
            <div className="library-mgr-edit-form">
              <h2>{t('library.edit.title')}</h2>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.name')}</label>
                <input value={editName} onChange={e => setEditName(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.category')}</label>
                <input value={editCategory} onChange={e => setEditCategory(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.description')}</label>
                <textarea value={editDesc} onChange={e => setEditDesc(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.ifc')}</label>
                <input value={editIfc} onChange={e => setEditIfc(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.standard_ref')}</label>
                <input value={editStd} onChange={e => setEditStd(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.domain')}</label>
                <input value={editDomain} onChange={e => setEditDomain(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.tags')}</label>
                <input value={editTags} onChange={e => setEditTags(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.materials_list')}</label>
                <input value={editMaterials} onChange={e => setEditMaterials(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.default_material')}</label>
                <input value={editDefaultMat} onChange={e => setEditDefaultMat(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>{t('library.field.feature_tree_id')}</label>
                <input value={editFeatureTree} onChange={e => setEditFeatureTree(e.target.value)} placeholder={t('library.ph.feature_tree_id')} />
              </div>
              <div className="library-mgr-edit-actions">
                <button className="library-mgr-btn" onClick={handleSaveEdit}>{t('library.btn.save')}</button>
                <button className="library-mgr-btn" onClick={() => setEditing(false)}>{t('library.btn.cancel')}</button>
              </div>
            </div>
          ) : (
            <div className="library-mgr-placeholder">{t('library.empty')}</div>
          )}
        </div>
      </div>
    </div>
  );
}

function PreviewCanvas({ mesh }: { mesh: CatalogPreviewMesh }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<{
    scene: THREE.Scene; camera: THREE.PerspectiveCamera;
    renderer: THREE.WebGLRenderer; controls: OrbitControls; animId: number;
  } | null>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a1a2e);

    const w = container.clientWidth;
    const h = 300;
    const camera = new THREE.PerspectiveCamera(45, w / h, 0.1, 200);
    camera.position.set(5, -5, 4);
    camera.up.set(0, 0, 1);
    camera.lookAt(0, 0, 0);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(w, h);
    renderer.setPixelRatio(window.devicePixelRatio);
    container.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.target.set(0, 0, 0);

    const ambientLight = new THREE.AmbientLight(0x404060, 0.5);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1);
    dirLight.position.set(5, 10, 7);
    scene.add(dirLight);
    const dirLight2 = new THREE.DirectionalLight(0x4488ff, 0.3);
    dirLight2.position.set(-5, -5, -5);
    scene.add(dirLight2);

    const gridHelper = new THREE.GridHelper(10, 10, 0x444466, 0x333355);
    scene.add(gridHelper);

    if (mesh.vertices.length > 0 && mesh.faces.length > 0) {
      const positions = new Float32Array(mesh.vertices);
      const normals = mesh.normals.length > 0 ? new Float32Array(mesh.normals) : undefined;
      const indices = new Uint32Array(mesh.faces);

      const geo = new THREE.BufferGeometry();
      geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
      if (normals) geo.setAttribute('normal', new THREE.BufferAttribute(normals, 3));
      geo.setIndex(new THREE.BufferAttribute(indices, 1));
      geo.computeVertexNormals();

      const mat = new THREE.MeshPhongMaterial({
        color: 0x4fc3f7,
        side: THREE.DoubleSide,
        flatShading: false,
        shininess: 30,
      });
      const mesh3d = new THREE.Mesh(geo, mat);
      scene.add(mesh3d);

      geo.computeBoundingSphere();
      const center = geo.boundingSphere?.center;
      const radius = geo.boundingSphere?.radius || 1;
      if (center) {
        controls.target.set(center.x, center.y, center.z);
        camera.position.set(center.x + radius * 2.5, center.y - radius * 2.5, center.z + radius * 2);
      }
    }

    const animate = () => {
      controls.update();
      renderer.render(scene, camera);
      sceneRef.current!.animId = requestAnimationFrame(animate);
    };
    animate();

    sceneRef.current = { scene, camera, renderer, controls, animId: 0 };
    sceneRef.current.animId = 0;

    const ro = new ResizeObserver(() => {
      const w2 = container.clientWidth;
      renderer.setSize(w2, h);
      camera.aspect = w2 / h;
      camera.updateProjectionMatrix();
    });
    ro.observe(container);

    return () => {
      cancelAnimationFrame(sceneRef.current?.animId ?? 0);
      ro.disconnect();
      renderer.dispose();
      container.removeChild(renderer.domElement);
    };
  }, [mesh]);

  return <div ref={containerRef} className="library-mgr-preview-canvas" style={{ height: 300 }} />;
}