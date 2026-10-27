import { useState, useCallback } from 'react';
import type { FeatureItem, SketchResponse } from '../shared/types';
import { api } from '../shared/api';

type Props = {
  componentId: string;
  features: FeatureItem[];
  sketches: SketchResponse[];
  onRefresh: () => void;
  onEditFeature: (feature: FeatureItem) => void;
  onSelectSketch: (sketchId: string) => void;
  onRebuilt?: () => void;
};

const FEATURE_ICONS: Record<string, string> = {
  extrude: '⬍', revolve: '↻', sweep: '⟿', loft: 'ℳ',
  boolean: '⊖', chamfer: '⊿', fillet: '⌒', pattern: '⊞', shell: '⊡',
};

export function FeaturePanel({ componentId, features, sketches, onRefresh, onEditFeature, onSelectSketch, onRebuilt }: Props) {
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);

  const handleDeleteFeature = async (featureId: string) => {
    if (!window.confirm('确定删除这个 Feature？')) return;
    try {
      await api.removeFeature(componentId, featureId);
      onRefresh();
    } catch (e) {
      alert('删除失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleToggleSuppress = async (feature: FeatureItem) => {
    try {
      await api.updateFeature(componentId, feature.id, { suppressed: !feature.suppressed });
      onRefresh();
    } catch (e) {
      console.warn('Toggle suppress failed:', e);
    }
  };

  const handleDragStart = useCallback((e: React.DragEvent, featureId: string) => {
    e.dataTransfer.setData('text/plain', featureId);
    e.dataTransfer.effectAllowed = 'move';
    setDraggedId(featureId);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent, featureId: string) => {
    if (!draggedId || draggedId === featureId) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverId(featureId);
  }, [draggedId]);

  const handleDragLeave = useCallback(() => {
    setDragOverId(null);
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent, targetId: string) => {
    e.preventDefault();
    setDragOverId(null);
    const sourceId = e.dataTransfer.getData('text/plain');
    if (!sourceId || sourceId === targetId) return;

    const reordered = [...features];
    const sourceIdx = reordered.findIndex(f => f.id === sourceId);
    const targetIdx = reordered.findIndex(f => f.id === targetId);
    if (sourceIdx === -1 || targetIdx === -1) return;

    const [moved] = reordered.splice(sourceIdx, 1);
    reordered.splice(targetIdx, 0, moved);

    try {
      await api.executeOmcom({
        version: '1.0',
        id: `reorder-${Date.now()}`,
        action: 113,
        target: { object: { object_id: componentId } },
        operations: [{ rebuild_features: { component_id: componentId } }],
      });
      onRefresh();
    } catch (e) {
      console.warn('Reorder failed:', e);
    }
    setDraggedId(null);
  }, [features, componentId, onRefresh]);

  const handleDragEnd = useCallback(() => {
    setDraggedId(null);
    setDragOverId(null);
  }, []);

  const handleRebuild = async () => {
    try {
      await api.rebuildComponent(componentId);
      onRefresh();
    } catch (e) {
      alert('Rebuild 失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const getSketchName = (sketchId: string) => {
    const sketch = sketches.find(s => s.id === sketchId);
    return sketch ? sketch.name : sketchId.slice(0, 8);
  };

  return (
    <div className="feature-panel">
      <div className="feature-panel-header">
        <strong>Feature Tree</strong>
      </div>

      <div className="feature-list">
        {features.length === 0 ? (
          <div className="feature-empty">
            暂无 Feature<br />
            <span style={{ fontSize: 11, color: '#666' }}>点击上方工具栏按钮添加</span>
          </div>
        ) : (
          features.map((feature) => (
            <div
              key={feature.id}
              className={`feature-card ${dragOverId === feature.id ? 'drag-over' : ''} ${feature.suppressed ? 'suppressed' : ''}`}
              draggable
              onDoubleClick={() => onEditFeature(feature)}
              onDragStart={(e) => handleDragStart(e, feature.id)}
              onDragOver={(e) => handleDragOver(e, feature.id)}
              onDragLeave={handleDragLeave}
              onDrop={(e) => handleDrop(e, feature.id)}
              onDragEnd={handleDragEnd}
            >
              <div className="feature-card-header">
                <span className="feature-drag-handle">≡</span>
                <span className="feature-type-icon">
                  {FEATURE_ICONS[feature.feature_type] || '?'}
                </span>
                <span className="feature-name">{feature.name}</span>
                <div className="feature-actions">
                  <button
                    className="feature-action-btn"
                    onClick={() => handleToggleSuppress(feature)}
                    title={feature.suppressed ? '取消抑制' : '抑制'}
                  >{feature.suppressed ? '🔇' : '🔊'}</button>
                  <button
                    className="feature-action-btn"
                    onClick={() => onEditFeature(feature)}
                    title="编辑"
                  >✏️</button>
                  <button
                    className="feature-action-btn danger"
                    onClick={() => handleDeleteFeature(feature.id)}
                    title="删除"
                  >🗑️</button>
                </div>
              </div>
              <div className="feature-card-body">
                <div className="feature-param">
                  <span className="feature-param-label">类型:</span>
                  <span className="feature-param-value">{feature.feature_type}</span>
                </div>
                {feature.sketch_id && (
                  <div className="feature-param">
                    <span className="feature-param-label">Sketch:</span>
                    <span
                      className="feature-param-value feature-link"
                      onClick={() => onSelectSketch(feature.sketch_id)}
                    >{getSketchName(feature.sketch_id)}</span>
                  </div>
                )}
                {feature.feature_type === 'extrude' && (
                  <div className="feature-param">
                    <span className="feature-param-label">Depth:</span>
                    <span className="feature-param-value">{feature.depth}</span>
                  </div>
                )}
                {feature.feature_type === 'revolve' && (
                  <div className="feature-param">
                    <span className="feature-param-label">Angle:</span>
                    <span className="feature-param-value">{feature.angle} rad</span>
                  </div>
                )}
                {feature.suppressed && (
                  <div className="feature-param feature-suppressed-tag">已抑制</div>
                )}
                {feature.dirty && (
                  <div className="feature-param feature-dirty-tag">需重建</div>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <div className="feature-rebuild-section">
        <button className="feature-rebuild-btn" onClick={handleRebuild}>
          🔄 Rebuild All
        </button>
      </div>
    </div>
  );
}
