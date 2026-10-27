import { useEffect, useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../shared/api';
import type { Series, Selector, Answer, SpecComponent, ComponentDef, ParamConstraint } from '../shared/types';

export function SeriesManagerPage() {
  const navigate = useNavigate();
  const [seriesList, setSeriesList] = useState<Series[]>([]);
  const [selectedSeries, setSelectedSeries] = useState<Series | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [editing, setEditing] = useState(false);
  const [editName, setEditName] = useState('');
  const [editDesc, setEditDesc] = useState('');
  const [editDomain, setEditDomain] = useState('');
  const [editStd, setEditStd] = useState('');
  const [editPhases, setEditPhases] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [createName, setCreateName] = useState('');
  const [createDomain, setCreateDomain] = useState('');
  const [saveMsg, setSaveMsg] = useState('');
  const [loadPath, setLoadPath] = useState('');
  const [catalog, setCatalog] = useState<ComponentDef[]>([]);
  const [catalogSearch, setCatalogSearch] = useState('');
  const [showAddSelector, setShowAddSelector] = useState(false);
  const [newSelectorId, setNewSelectorId] = useState('');
  const [newSelectorQ, setNewSelectorQ] = useState('');
  const [showAddAnswer, setShowAddAnswer] = useState<string | null>(null);
  const [newAnswerId, setNewAnswerId] = useState('');
  const [newAnswerValue, setNewAnswerValue] = useState('');
  const [draggedCompDefId, setDraggedCompDefId] = useState<string | null>(null);
  const [selectedTab, setSelectedTab] = useState<'tree' | 'catalog' | 'mapping'>('tree');

  const loadSeries = useCallback(async () => {
    try {
      const list = await api.listSeries();
      setSeriesList(list);
      setError('');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const loadCatalog = useCallback(async () => {
    try {
      const list = await api.listCatalog();
      setCatalog(list);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => { loadSeries(); loadCatalog(); }, [loadSeries, loadCatalog]);

  const handleSelect = async (s: Series) => {
    try {
      const detail = await api.getSeries(s.id);
      setSelectedSeries(detail);
      setEditing(false);
    } catch (e) {
      setError('加载系列详情失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleStartEdit = () => {
    if (!selectedSeries) return;
    setEditName(selectedSeries.name);
    setEditDesc(selectedSeries.description);
    setEditDomain(selectedSeries.domain);
    setEditStd(selectedSeries.standard_reference);
    setEditPhases(selectedSeries.applicable_phases.join(', '));
    setEditing(true);
  };

  const handleSaveEdit = async () => {
    if (!selectedSeries) return;
    try {
      await api.updateSeries(selectedSeries.id, {
        name: editName, description: editDesc, domain: editDomain,
        standard_reference: editStd,
        applicable_phases: editPhases.split(',').map(p => p.trim()).filter(Boolean),
      });
      setEditing(false);
      loadSeries();
      await handleSelect({ ...selectedSeries, name: editName, description: editDesc, domain: editDomain, standard_reference: editStd });
    } catch (e) {
      alert('保存失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleCreate = async () => {
    if (!createName) { alert('名称不能为空'); return; }
    try {
      await api.createSeries({ name: createName, domain: createDomain });
      setCreateName(''); setCreateDomain(''); setShowCreateForm(false);
      loadSeries();
    } catch (e) {
      alert('创建失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleDeleteSeries = async (id: string) => {
    if (!window.confirm('确定删除此系列？')) return;
    try {
      await api.deleteSeries(id);
      if (selectedSeries?.id === id) setSelectedSeries(null);
      loadSeries();
    } catch (e) {
      alert('删除失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleAddSelector = async () => {
    if (!selectedSeries || !newSelectorId || !newSelectorQ) return;
    try {
      await api.addSelector(selectedSeries.id, { selector_id: newSelectorId, question: newSelectorQ });
      setNewSelectorId(''); setNewSelectorQ(''); setShowAddSelector(false);
      const detail = await api.getSeries(selectedSeries.id);
      setSelectedSeries(detail);
    } catch (e) {
      alert('添加选择器失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleAddAnswer = async (selectorId: string, compDefId?: string) => {
    if (!selectedSeries || !newAnswerId || !newAnswerValue) return;
    try {
      await api.addAnswer(selectedSeries.id, { selector_id: selectorId, answer_id: newAnswerId, answer_value: newAnswerValue });
      if (compDefId) {
        await api.addSpecComponent(selectedSeries.id, { selector_id: selectorId, answer_id: newAnswerId, component_def_id: compDefId });
      }
      setNewAnswerId(''); setNewAnswerValue(''); setShowAddAnswer(null);
      const detail = await api.getSeries(selectedSeries.id);
      setSelectedSeries(detail);
    } catch (e) {
      alert('添加答案失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleMapComponent = async (answerId: string, compDefId: string) => {
    if (!selectedSeries) return;
    try {
      await api.addSpecComponent(selectedSeries.id, { selector_id: '', answer_id: answerId, component_def_id: compDefId });
      const detail = await api.getSeries(selectedSeries.id);
      setSelectedSeries(detail);
    } catch (e) {
      alert('映射构件失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleSave = async () => {
    try {
      const res = await api.saveSeries();
      setSaveMsg(`保存成功: ${res.path} (${res.saved_count} 个系列)`);
    } catch (e) {
      setSaveMsg('保存失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleLoad = async () => {
    if (!loadPath) { alert('请输入文件路径'); return; }
    try {
      const res = await api.loadSeries(loadPath, 'standalone');
      setLoadPath('');
      loadSeries();
      alert(`加载成功: ${res.loaded_count} 个系列 (${res.series_names.join(', ')})`);
    } catch (e) {
      alert('加载失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleDropOnAnswer = (answerId: string) => {
    if (draggedCompDefId) {
      handleMapComponent(answerId, draggedCompDefId);
      setDraggedCompDefId(null);
    }
  };

  const handleAddConstraint = async (compDefId: string, paramName: string) => {
    if (!selectedSeries || !paramName) return;
    try {
      await api.addParamConstraint(selectedSeries.id, {
        component_def_id: compDefId,
        param_name: paramName,
        default_value: 0,
        min_value: 0,
        max_value: 1000,
        step: 0.1,
      });
      const detail = await api.getSeries(selectedSeries.id);
      setSelectedSeries(detail);
    } catch (e) {
      alert('添加约束失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleDeleteConstraint = async (compDefId: string, constraintIdx: number) => {
    if (!selectedSeries) return;
    try {
      await api.deleteParamConstraint(selectedSeries.id, constraintIdx, compDefId);
      const detail = await api.getSeries(selectedSeries.id);
      setSelectedSeries(detail);
    } catch (e) {
      alert('删除约束失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const filteredCatalog = catalog.filter(c =>
    !catalogSearch || c.name.toLowerCase().includes(catalogSearch.toLowerCase()) ||
    c.category.toLowerCase().includes(catalogSearch.toLowerCase()) ||
    c.tags.some(t => t.toLowerCase().includes(catalogSearch.toLowerCase()))
  );

  const mappedCompDefIds = new Set<string>();
  const collectMappedIds = (sel: Selector) => {
    for (const ans of sel.answers) {
      if (ans.outcome?.component) mappedCompDefIds.add(ans.outcome.component.component_def_id);
      if (ans.outcome?.next_selector) collectMappedIds(ans.outcome.next_selector);
    }
  };
  if (selectedSeries?.root_selector) collectMappedIds(selectedSeries.root_selector);

  return (
    <div className="series-mgr-page">
      <div className="series-mgr-header">
        <h1>SeriesStore Manager</h1>
        <div className="series-mgr-header-actions">
          <button className="series-mgr-btn" onClick={() => setShowCreateForm(!showCreateForm)}>+ 新建系列</button>
          <button className="series-mgr-btn" onClick={handleSave}>保存 .omnimseries</button>
          <button className="series-mgr-back" onClick={() => navigate('/')}>← 返回</button>
        </div>
      </div>

      <div className="series-mgr-io-bar">
        <input placeholder="加载 .omnimseries 文件路径" value={loadPath} onChange={e => setLoadPath(e.target.value)} />
        <button className="series-mgr-btn" onClick={handleLoad}>加载</button>
        {saveMsg && <span className="series-mgr-io-msg">{saveMsg}</span>}
      </div>

      {showCreateForm && (
        <div className="series-mgr-create-form">
          <input placeholder="系列名称" value={createName} onChange={e => setCreateName(e.target.value)} />
          <input placeholder="领域 (可选)" value={createDomain} onChange={e => setCreateDomain(e.target.value)} />
          <button className="series-mgr-btn" onClick={handleCreate}>确认创建</button>
          <button className="series-mgr-btn" onClick={() => setShowCreateForm(false)}>取消</button>
        </div>
      )}

      {error && <div className="series-mgr-error">{error}</div>}

      <div className="series-mgr-body">
        <div className="series-mgr-sidebar">
          <h3>系列列表</h3>
          {loading && <div className="series-mgr-loading">加载中...</div>}
          {!loading && seriesList.length === 0 && <div className="series-mgr-empty">暂无系列</div>}
          {seriesList.map((s) => (
            <div
              key={s.id}
              className={`series-mgr-item ${selectedSeries?.id === s.id ? 'selected' : ''}`}
              onClick={() => handleSelect(s)}
            >
              <div className="series-mgr-item-name">{s.name}</div>
              <div className="series-mgr-item-domain">{s.domain}</div>
            </div>
          ))}
        </div>

        <div className="series-mgr-content">
          {selectedSeries && !editing ? (
            <>
              <div className="series-mgr-info">
                <div className="series-mgr-info-header">
                  <h2>{selectedSeries.name}</h2>
                  <div className="series-mgr-info-actions">
                    <button className="series-mgr-btn" onClick={handleStartEdit}>编辑</button>
                    <button className="series-mgr-btn danger" onClick={() => handleDeleteSeries(selectedSeries.id)}>删除</button>
                  </div>
                </div>
                <div className="series-mgr-meta">
                  <span>领域: {selectedSeries.domain}</span>
                  <span>标准: {selectedSeries.standard_reference}</span>
                  <span>阶段: {selectedSeries.applicable_phases.join(', ') || '通用'}</span>
                </div>
                <p className="series-mgr-desc">{selectedSeries.description}</p>
              </div>

              <div className="series-mgr-tabs">
                <button className={`series-mgr-tab ${selectedTab === 'tree' ? 'active' : ''}`} onClick={() => setSelectedTab('tree')}>决策树</button>
                <button className={`series-mgr-tab ${selectedTab === 'catalog' ? 'active' : ''}`} onClick={() => setSelectedTab('catalog')}>构件目录</button>
                <button className={`series-mgr-tab ${selectedTab === 'mapping' ? 'active' : ''}`} onClick={() => setSelectedTab('mapping')}>映射概览</button>
              </div>

              {selectedTab === 'tree' && (
                <div className="series-mgr-tree">
                  <div className="series-mgr-tree-header">
                    <h3>决策树</h3>
                    <button className="series-mgr-btn" onClick={() => setShowAddSelector(!showAddSelector)}>+ 添加选择器</button>
                  </div>
                  {showAddSelector && (
                    <div className="series-mgr-inline-form">
                      <input placeholder="选择器ID" value={newSelectorId} onChange={e => setNewSelectorId(e.target.value)} />
                      <input placeholder="问题" value={newSelectorQ} onChange={e => setNewSelectorQ(e.target.value)} />
                      <button className="series-mgr-btn" onClick={handleAddSelector}>确认</button>
                      <button className="series-mgr-btn" onClick={() => setShowAddSelector(false)}>取消</button>
                    </div>
                  )}
                  {selectedSeries.root_selector ? (
                    <SelectorNode
                      selector={selectedSeries.root_selector}
                      depth={0}
                      seriesId={selectedSeries.id}
                      catalog={catalog}
                      draggedCompDefId={draggedCompDefId}
                      onDropOnAnswer={handleDropOnAnswer}
                      showAddAnswer={showAddAnswer}
                      onShowAddAnswer={setShowAddAnswer}
                      newAnswerId={newAnswerId}
                      setNewAnswerId={setNewAnswerId}
                      newAnswerValue={newAnswerValue}
                      setNewAnswerValue={setNewAnswerValue}
                      onAddAnswer={handleAddAnswer}
                      onRefresh={() => selectedSeries && handleSelect(selectedSeries)}
                      onAddConstraint={handleAddConstraint}
                      onDeleteConstraint={handleDeleteConstraint}
                    />
                  ) : (
                    <div className="series-mgr-empty">此系列没有决策树，请添加选择器</div>
                  )}
                </div>
              )}

              {selectedTab === 'catalog' && (
                <div className="series-mgr-catalog-panel">
                  <h3>构件目录</h3>
                  <input
                    className="series-mgr-catalog-search"
                    placeholder="搜索构件名称、类别、标签..."
                    value={catalogSearch}
                    onChange={e => setCatalogSearch(e.target.value)}
                  />
                  <div className="series-mgr-catalog-list">
                    {filteredCatalog.length === 0 && <div className="series-mgr-empty">无匹配构件</div>}
                    {filteredCatalog.map(c => (
                      <div
                        key={c.id}
                        className={`series-mgr-catalog-card ${mappedCompDefIds.has(c.id) ? 'mapped' : ''}`}
                        draggable
                        onDragStart={() => setDraggedCompDefId(c.id)}
                        onDragEnd={() => setDraggedCompDefId(null)}
                      >
                        <div className="series-mgr-catalog-card-name">{c.name}</div>
                        <div className="series-mgr-catalog-card-meta">{c.category} · {c.ifc_class}</div>
                        <div className="series-mgr-catalog-card-tags">{c.tags.join(', ')}</div>
                        {mappedCompDefIds.has(c.id) && <span className="series-mgr-catalog-card-badge">已映射</span>}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {selectedTab === 'mapping' && (
                <div className="series-mgr-mapping-panel">
                  <h3>系列 → 构件映射概览</h3>
                  {selectedSeries.root_selector ? (
                    <MappingOverview selector={selectedSeries.root_selector} depth={0} />
                  ) : (
                    <div className="series-mgr-empty">无映射</div>
                  )}
                </div>
              )}
            </>
          ) : selectedSeries && editing ? (
            <div className="series-mgr-edit-form">
              <h2>编辑系列</h2>
              <div className="series-mgr-edit-field">
                <label>名称</label><input value={editName} onChange={e => setEditName(e.target.value)} />
              </div>
              <div className="series-mgr-edit-field">
                <label>描述</label><textarea value={editDesc} onChange={e => setEditDesc(e.target.value)} />
              </div>
              <div className="series-mgr-edit-field">
                <label>领域</label><input value={editDomain} onChange={e => setEditDomain(e.target.value)} />
              </div>
              <div className="series-mgr-edit-field">
                <label>标准参考</label><input value={editStd} onChange={e => setEditStd(e.target.value)} />
              </div>
              <div className="series-mgr-edit-field">
                <label>适用阶段 (逗号分隔)</label><input value={editPhases} onChange={e => setEditPhases(e.target.value)} />
              </div>
              <div className="series-mgr-edit-actions">
                <button className="series-mgr-btn" onClick={handleSaveEdit}>保存</button>
                <button className="series-mgr-btn" onClick={() => setEditing(false)}>取消</button>
              </div>
            </div>
          ) : (
            <div className="series-mgr-placeholder">选择一个系列查看详情</div>
          )}
        </div>
      </div>
    </div>
  );
}

function MappingOverview({ selector, depth }: { selector: Selector; depth: number }) {
  return (
    <div className="mapping-overview" style={{ marginLeft: depth * 16 }}>
      <div className="mapping-overview-selector">
        ❓ {selector.question}
      </div>
      {selector.answers.map(ans => (
        <div key={ans.id} className="mapping-overview-answer" style={{ marginLeft: 16 }}>
          <div className="mapping-overview-answer-value">▸ {ans.value}</div>
          {ans.outcome?.component && (
            <div className="mapping-overview-component">
              📦 {ans.outcome.component.component_def_id}
              {ans.outcome.component.param_constraints.length > 0 && (
                <span className="mapping-overview-constraint-count">
                  ({ans.outcome.component.param_constraints.length} 约束)
                </span>
              )}
            </div>
          )}
          {ans.outcome?.next_selector && (
            <MappingOverview selector={ans.outcome.next_selector} depth={depth + 1} />
          )}
        </div>
      ))}
    </div>
  );
}

type SelectorNodeProps = {
  selector: Selector;
  depth: number;
  seriesId: string;
  catalog: ComponentDef[];
  draggedCompDefId: string | null;
  onDropOnAnswer: (answerId: string) => void;
  showAddAnswer: string | null;
  onShowAddAnswer: (id: string | null) => void;
  newAnswerId: string;
  setNewAnswerId: (id: string) => void;
  newAnswerValue: string;
  setNewAnswerValue: (v: string) => void;
  onAddAnswer: (selectorId: string, compDefId?: string) => void;
  onRefresh: () => void;
  onAddConstraint: (compDefId: string, paramName: string) => void;
  onDeleteConstraint: (compDefId: string, constraintIdx: number) => void;
};

function SelectorNode({ selector, depth, seriesId, catalog, draggedCompDefId, onDropOnAnswer, showAddAnswer, onShowAddAnswer, newAnswerId, setNewAnswerId, newAnswerValue, setNewAnswerValue, onAddAnswer, onRefresh, onAddConstraint, onDeleteConstraint }: SelectorNodeProps) {
  return (
    <div className="selector-node" style={{ marginLeft: depth * 20 }}>
      <div className="selector-question">
        <span className="selector-icon">❓</span>
        {selector.question}
        <span className="selector-id" style={{ color: '#666', fontSize: 10, marginLeft: 8 }}>({selector.id})</span>
      </div>
      <div className="selector-answers">
        {selector.answers.map((answer) => (
          <AnswerNode
            key={answer.id}
            answer={answer}
            depth={depth + 1}
            seriesId={seriesId}
            catalog={catalog}
            draggedCompDefId={draggedCompDefId}
            onDrop={onDropOnAnswer}
            onRefresh={onRefresh}
            onAddConstraint={onAddConstraint}
            onDeleteConstraint={onDeleteConstraint}
          />
        ))}
        <div className="selector-add-answer">
          {showAddAnswer === selector.id ? (
            <div className="series-mgr-inline-form">
              <input placeholder="答案ID" value={newAnswerId} onChange={e => setNewAnswerId(e.target.value)} />
              <input placeholder="答案值" value={newAnswerValue} onChange={e => setNewAnswerValue(e.target.value)} />
              <button className="series-mgr-btn" onClick={() => onAddAnswer(selector.id)}>添加</button>
              <button className="series-mgr-btn" onClick={() => onAddAnswer(selector.id, catalog[0]?.id)}>+ 映射</button>
              <button className="series-mgr-btn" onClick={() => onShowAddAnswer(null)}>取消</button>
            </div>
          ) : (
            <button className="series-mgr-btn-sm" onClick={() => onShowAddAnswer(selector.id)}>+ 答案</button>
          )}
        </div>
      </div>
    </div>
  );
}

function AnswerNode({ answer, depth, seriesId, catalog, draggedCompDefId, onDrop, onRefresh, onAddConstraint, onDeleteConstraint }: {
  answer: Answer; depth: number; seriesId: string; catalog: ComponentDef[];
  draggedCompDefId: string | null; onDrop: (answerId: string) => void;
  onRefresh: () => void;
  onAddConstraint: (compDefId: string, paramName: string) => void;
  onDeleteConstraint: (compDefId: string, constraintIdx: number) => void;
}) {
  const outcome = answer.outcome;
  const [showMapPicker, setShowMapPicker] = useState(false);
  const [showConstrEditor, setShowConstrEditor] = useState(false);
  const [newConstrParam, setNewConstrParam] = useState('');

  const isDragOver = draggedCompDefId !== null;
  const compDef = outcome?.component
    ? catalog.find(c => c.id === outcome.component!.component_def_id)
    : null;

  return (
    <div
      className={`answer-node ${isDragOver ? 'drag-over' : ''}`}
      style={{ marginLeft: depth * 20 }}
      onDragOver={e => { if (isDragOver) e.preventDefault(); }}
      onDrop={e => { e.preventDefault(); onDrop(answer.id); }}
    >
      <div className="answer-value">
        <span className="answer-icon">▸</span>
        {answer.value}
        {answer.is_numeric_range && (
          <span className="answer-range"> ({answer.range_min}–{answer.range_max})</span>
        )}
      </div>

      {outcome?.next_selector && (
        <SelectorNode
          selector={outcome.next_selector} depth={depth + 1} seriesId={seriesId}
          catalog={catalog} draggedCompDefId={draggedCompDefId} onDropOnAnswer={onDrop}
          showAddAnswer={null} onShowAddAnswer={() => {}} newAnswerId="" setNewAnswerId={() => {}}
          newAnswerValue="" setNewAnswerValue={() => {}} onAddAnswer={async () => {}}
          onRefresh={onRefresh} onAddConstraint={onAddConstraint} onDeleteConstraint={onDeleteConstraint}
        />
      )}

      {outcome?.component && (
        <div className="component-leaf" style={{ marginLeft: depth * 20 }}>
          <div className="component-leaf-header">
            <span className="component-leaf-icon">📦</span>
            <span className="component-leaf-def">{outcome.component.component_def_id}</span>
            {compDef && <span className="component-leaf-name">{compDef.name}</span>}
            <button className="series-mgr-btn-sm" onClick={() => setShowMapPicker(!showMapPicker)}>✏️</button>
          </div>

          {showMapPicker && (
            <div className="series-mgr-inline-form">
              <select
                value={outcome.component.component_def_id}
                onChange={async e => {
                  await api.addSpecComponent(seriesId, { selector_id: '', answer_id: answer.id, component_def_id: e.target.value });
                  onRefresh();
                }}
              >
                {catalog.map(c => <option key={c.id} value={c.id}>{c.name} ({c.category})</option>)}
              </select>
              <button className="series-mgr-btn" onClick={() => setShowMapPicker(false)}>关闭</button>
            </div>
          )}

          {outcome.component.material && (
            <div className="component-leaf-prop">材质: {outcome.component.material}</div>
          )}

          {outcome.component.param_constraints.length > 0 && (
            <div className="component-leaf-constraints">
              <div className="component-leaf-constraints-header">
                <span>参数约束</span>
                <button className="series-mgr-btn-sm" onClick={() => setShowConstrEditor(!showConstrEditor)}>+</button>
              </div>
              {outcome.component.param_constraints.map((pc, ci) => (
                <ConstraintEditor
                  key={pc.param_name + ci}
                  constraint={pc}
                  seriesId={seriesId}
                  compDefId={outcome.component!.component_def_id}
                  constraintIdx={ci}
                  onRefresh={onRefresh}
                  onDelete={onDeleteConstraint}
                />
              ))}
            </div>
          )}

          {showConstrEditor && (
            <div className="series-mgr-inline-form">
              <select value={newConstrParam} onChange={e => setNewConstrParam(e.target.value)}>
                <option value="">选择参数</option>
                {compDef?.params.map(p => <option key={p.name} value={p.name}>{p.name}</option>)}
              </select>
              <button className="series-mgr-btn" onClick={() => { if (newConstrParam) { onAddConstraint(outcome.component!.component_def_id, newConstrParam); setNewConstrParam(''); setShowConstrEditor(false); } }}>添加</button>
              <button className="series-mgr-btn" onClick={() => setShowConstrEditor(false)}>取消</button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function ConstraintEditor({ constraint, seriesId, compDefId, constraintIdx, onRefresh, onDelete }: {
  constraint: ParamConstraint; seriesId: string; compDefId: string; constraintIdx: number;
  onRefresh: () => void; onDelete: (compDefId: string, constraintIdx: number) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [editDefault, setEditDefault] = useState(constraint.default_value.toString());
  const [editMin, setEditMin] = useState(constraint.min_value.toString());
  const [editMax, setEditMax] = useState(constraint.max_value.toString());
  const [editStep, setEditStep] = useState(constraint.step.toString());

  const handleSave = async () => {
    try {
      await api.updateParamConstraint(seriesId, constraintIdx, {
        component_def_id: compDefId,
        param_name: constraint.param_name,
        default_value: parseFloat(editDefault) || 0,
        min_value: parseFloat(editMin) || 0,
        max_value: parseFloat(editMax) || 1000,
        step: parseFloat(editStep) || 0.1,
      });
      setEditing(false);
      onRefresh();
    } catch (e) {
      alert('更新约束失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  return (
    <div className="component-leaf-constraint">
      {editing ? (
        <div className="series-mgr-inline-form">
          <span className="component-leaf-constraint-name">{constraint.param_name}</span>
          <input type="number" value={editDefault} onChange={e => setEditDefault(e.target.value)} title="默认值" />
          <input type="number" value={editMin} onChange={e => setEditMin(e.target.value)} title="最小值" />
          <input type="number" value={editMax} onChange={e => setEditMax(e.target.value)} title="最大值" />
          <input type="number" value={editStep} onChange={e => setEditStep(e.target.value)} title="步长" />
          <button className="series-mgr-btn" onClick={handleSave}>保存</button>
          <button className="series-mgr-btn" onClick={() => setEditing(false)}>取消</button>
        </div>
      ) : (
        <div className="component-leaf-constraint-row">
          <span className="component-leaf-constraint-name">{constraint.param_name}</span>
          <span className="component-leaf-constraint-val">默认: {constraint.default_value}</span>
          <span className="component-leaf-constraint-val">范围: {constraint.min_value}–{constraint.max_value}</span>
          <span className="component-leaf-constraint-val">步长: {constraint.step}</span>
          <button className="series-mgr-btn-sm" onClick={() => setEditing(true)}>✏️</button>
          <button className="series-mgr-btn-sm" onClick={() => onDelete(compDefId, constraintIdx)}>🗑️</button>
        </div>
      )}
    </div>
  );
}