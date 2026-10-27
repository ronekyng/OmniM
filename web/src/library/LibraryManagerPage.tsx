import { useEffect, useState, useCallback, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { api } from '../shared/api';
import type { ComponentDef, ParamDef, CatalogPreviewMesh } from '../shared/types';

export function LibraryManagerPage() {
  const navigate = useNavigate();
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

  const DOMAIN_LABELS: Record<string, string> = {
    structural: '建筑结构',
    building: '建筑',
    civil: '土木',
    plant: '工厂',
    mechanical: '机械',
    piping: '管道',
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
      setPreviewError('预览失败: ' + (e instanceof Error ? e.message : String(e)));
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
      alert('保存失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleCreate = async () => {
    if (!createName || !createCategory) { alert('名称和类别不能为空'); return; }
    try {
      await api.createComponentDef({ name: createName, category: createCategory });
      setCreateName('');
      setCreateCategory('');
      setShowCreateForm(false);
      loadCatalog();
    } catch (e) {
      alert('创建失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm('确定删除此构件定义？')) return;
    try {
      await api.deleteComponentDef(id);
      if (selectedId === id) setSelectedId(null);
      loadCatalog();
    } catch (e) {
      alert('删除失败: ' + (e instanceof Error ? e.message : String(e)));
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
      alert('添加参数失败: ' + (e instanceof Error ? e.message : String(e)));
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
      alert('更新参数失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleDeleteParam = async (idx: number) => {
    if (!selectedId || !window.confirm('确定删除此参数？')) return;
    try {
      await api.deleteComponentParam(selectedId, idx);
      loadCatalog();
    } catch (e) {
      alert('删除参数失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleExport = async () => {
    try {
      const res = await api.exportCatalog();
      setExportMsg(`导出成功: ${res.path} (${res.component_count} 个构件)`);
    } catch (e) {
      setExportMsg('导出失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleImport = async () => {
    if (!importPath) { alert('请输入文件路径'); return; }
    try {
      const res = await api.importCatalog(importPath);
      setImportMsg(`导入成功: ${res.component_count} 个构件 (${res.names.join(', ')})`);
      setImportPath('');
      loadCatalog();
    } catch (e) {
      setImportMsg('导入失败: ' + (e instanceof Error ? e.message : String(e)));
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
      <div className="library-mgr-header">
        <h1>Library Manager</h1>
        <div className="library-mgr-header-actions">
          <button className="library-mgr-btn" onClick={() => setShowCreateForm(!showCreateForm)}>+ 新建构件</button>
          <button className="library-mgr-btn" onClick={handleExport}>导出 .omnimlib</button>
          <button className="library-mgr-back" onClick={() => navigate('/')}>← 返回</button>
        </div>
      </div>

      {showCreateForm && (
        <div className="library-mgr-create-form">
          <input placeholder="名称" value={createName} onChange={e => setCreateName(e.target.value)} />
          <input placeholder="类别" value={createCategory} onChange={e => setCreateCategory(e.target.value)} />
          <button className="library-mgr-btn" onClick={handleCreate}>确认创建</button>
          <button className="library-mgr-btn" onClick={() => setShowCreateForm(false)}>取消</button>
        </div>
      )}

      <div className="library-mgr-io-bar">
        <input placeholder="导入 .omnimlib 文件路径" value={importPath} onChange={e => setImportPath(e.target.value)} />
        <button className="library-mgr-btn" onClick={handleImport}>导入</button>
        {importMsg && <span className="library-mgr-io-msg">{importMsg}</span>}
        {exportMsg && <span className="library-mgr-io-msg">{exportMsg}</span>}
      </div>

      {error && <div className="library-mgr-error">{error}</div>}

      <div className="library-mgr-body">
        <div className="library-mgr-sidebar">
          <h3>构件目录</h3>
          <div className="library-mgr-domain-tabs">
            <button
              className={`library-mgr-domain-tab ${domainFilter === '' ? 'active' : ''}`}
              onClick={() => setDomainFilter('')}
            >全部</button>
            {domains.map(d => (
              <button
                key={d}
                className={`library-mgr-domain-tab ${domainFilter === d ? 'active' : ''}`}
                onClick={() => setDomainFilter(d)}
              >{DOMAIN_LABELS[d] || d}</button>
            ))}
          </div>
          {loading && <div className="library-mgr-loading">加载中...</div>}
          {!loading && filteredCatalog.length === 0 && <div className="library-mgr-empty">暂无构件</div>}
          {categories.map(cat => (
            <div key={cat} className="library-mgr-category">
              <div className="library-mgr-cat-header" onClick={() => toggleCategory(cat)}>
                <span className="library-mgr-cat-arrow">{expandedCategories.has(cat) ? '▼' : '▶'}</span>
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
                  {selectedComp.published ? '已发布' : '草稿'}
                </span>
                <div className="library-mgr-detail-actions">
                  <button className="library-mgr-btn" onClick={handleStartEdit}>编辑</button>
                  <button className="library-mgr-btn" onClick={() => handleDelete(selectedComp.id)}>删除</button>
                </div>
              </div>

              <div className="library-mgr-detail-grid">
                <div className="library-mgr-detail-field">
                  <label>类别</label><span>{selectedComp.category}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>IFC 类</label><span>{selectedComp.ifc_class}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>标准</label><span>{selectedComp.standard_reference}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>领域</label><span>{selectedComp.domain}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>特征树模板</label><span>{selectedComp.feature_tree_id || '无'}</span>
                </div>
                <div className="library-mgr-detail-field">
                  <label>默认材质</label><span>{selectedComp.default_material || '无'}</span>
                </div>
              </div>

              <div className="library-mgr-detail-actions">
                <button className="library-mgr-btn" onClick={handlePreview} disabled={previewLoading}>
                  {previewLoading ? '生成中...' : '🔍 3D 预览'}
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
                  <h4>允许材质</h4>
                  <div className="library-mgr-materials">
                    {selectedComp.allowed_materials.map(m => <span key={m} className="library-mgr-mat">{m}</span>)}
                  </div>
                </div>
              )}

              <div className="library-mgr-section">
                <h4>参数定义 ({selectedComp.params.length})</h4>
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
                          <button className="library-mgr-btn" onClick={() => handleUpdateParam(idx, p)}>保存</button>
                          <button className="library-mgr-btn" onClick={() => setEditingParamIdx(null)}>取消</button>
                        </div>
                      ) : (
                        <>
                          <div className="library-mgr-param-header">
                            <span className="library-mgr-param-name">{p.name}</span>
                            <span className="library-mgr-param-default">{p.default_value} {p.unit}</span>
                            <div className="library-mgr-param-actions">
                              <button className="library-mgr-btn-sm" onClick={() => setEditingParamIdx(idx)}>✏️</button>
                              <button className="library-mgr-btn-sm danger" onClick={() => handleDeleteParam(idx)}>🗑️</button>
                            </div>
                          </div>
                          <div className="library-mgr-param-range">
                            范围: {p.min_value} – {p.max_value} · 步长: {p.step}
                          </div>
                        </>
                      )}
                    </div>
                  ))}
                </div>

                <div className="library-mgr-add-param">
                  <input placeholder="参数名" value={newParamName} onChange={e => setNewParamName(e.target.value)} />
                  <input type="number" placeholder="默认值" value={newParamDefault} onChange={e => setNewParamDefault(e.target.value)} />
                  <input type="number" placeholder="最小值" value={newParamMin} onChange={e => setNewParamMin(e.target.value)} />
                  <input type="number" placeholder="最大值" value={newParamMax} onChange={e => setNewParamMax(e.target.value)} />
                  <input type="number" placeholder="步长" value={newParamStep} onChange={e => setNewParamStep(e.target.value)} />
                  <input placeholder="单位" value={newParamUnit} onChange={e => setNewParamUnit(e.target.value)} />
                  <button className="library-mgr-btn" onClick={handleAddParam}>添加参数</button>
                </div>
              </div>

              <div className="library-mgr-section">
                <h4>连接点 ({selectedComp.connectors.length})</h4>
                {selectedComp.connectors.length === 0 ? (
                  <div className="library-mgr-empty">无连接点</div>
                ) : (
                  selectedComp.connectors.map(c => (
                    <div key={c.id} className="library-mgr-connector-card">
                      <div className="library-mgr-connector-name">{c.name} ({c.id})</div>
                      <div className="library-mgr-connector-detail">
                        位置: ({c.position_x_expr}, {c.position_y_expr}, {c.position_z_expr})
                        · 方向: ({c.direction_x_expr}, {c.direction_y_expr}, {c.direction_z_expr})
                        · 类型: {c.connection_type_code}
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          ) : selectedComp && editing ? (
            <div className="library-mgr-edit-form">
              <h2>编辑构件</h2>
              <div className="library-mgr-edit-field">
                <label>名称</label>
                <input value={editName} onChange={e => setEditName(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>类别</label>
                <input value={editCategory} onChange={e => setEditCategory(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>描述</label>
                <textarea value={editDesc} onChange={e => setEditDesc(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>IFC 类</label>
                <input value={editIfc} onChange={e => setEditIfc(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>标准参考</label>
                <input value={editStd} onChange={e => setEditStd(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>领域</label>
                <input value={editDomain} onChange={e => setEditDomain(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>标签 (逗号分隔)</label>
                <input value={editTags} onChange={e => setEditTags(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>允许材质 (逗号分隔)</label>
                <input value={editMaterials} onChange={e => setEditMaterials(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>默认材质</label>
                <input value={editDefaultMat} onChange={e => setEditDefaultMat(e.target.value)} />
              </div>
              <div className="library-mgr-edit-field">
                <label>特征树模板 ID</label>
                <input value={editFeatureTree} onChange={e => setEditFeatureTree(e.target.value)} placeholder="输入特征树模板 ID" />
              </div>
              <div className="library-mgr-edit-actions">
                <button className="library-mgr-btn" onClick={handleSaveEdit}>保存</button>
                <button className="library-mgr-btn" onClick={() => setEditing(false)}>取消</button>
              </div>
            </div>
          ) : (
            <div className="library-mgr-placeholder">选择一个构件查看详情</div>
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