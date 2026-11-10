import { useState, useCallback } from 'react';
import type { FeatureItem, SketchResponse } from '../shared/types';
import { api } from '../shared/api';
import { notify } from '../shared/toast';
import { useConfirm } from '../app/components/ConfirmDialog';
import { useI18n } from '../shared/i18n';
import { Glyph, IcoDelete, IcoEdit, IcoRefresh, IcoVolumeOff, IcoVolumeOn, type GlyphName } from '../app/components/Icons';

type Props = {
  componentId: string;
  features: FeatureItem[];
  sketches: SketchResponse[];
  onRefresh: () => void;
  onEditFeature: (feature: FeatureItem) => void;
  onSelectSketch: (sketchId: string) => void;
  onRebuilt?: () => void;
};

export function FeaturePanel({ componentId, features, sketches, onRefresh, onEditFeature, onSelectSketch, onRebuilt }: Props) {
  const { t } = useI18n();
  const confirm = useConfirm();
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);

  const handleDeleteFeature = async (featureId: string) => {
    const ok = await confirm({ title: 'Delete this feature?', confirmLabel: 'Delete', danger: true });
    if (!ok) return;
    try {
      await api.removeFeature(componentId, featureId);
      onRefresh();
    } catch (e) {
      notify.error(`${t('feature.delete_failed')}: ${e instanceof Error ? e.message : String(e)}`);
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
      notify.error(`${t('feature.rebuild_failed')}: ${e instanceof Error ? e.message : String(e)}`);
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
            {t('feature.empty')}<br />
            <span style={{ fontSize: 11, color: '#666' }}>{t('feature.empty_hint')}</span>
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
                  <Glyph name={feature.feature_type as GlyphName} />
                </span>
                <span className="feature-name">{feature.name}</span>
                <div className="feature-actions">
                  <button
                    className="feature-action-btn"
                    onClick={() => handleToggleSuppress(feature)}
                    title={feature.suppressed ? t('feature.unsuppress') : t('feature.suppress')}
                  >{feature.suppressed ? <IcoVolumeOff /> : <IcoVolumeOn />}</button>
                  <button
                    className="feature-action-btn"
                    onClick={() => onEditFeature(feature)}
                    title={t('feature.btn.edit')}
                    aria-label={t('feature.btn.edit')}
                  ><IcoEdit /></button>
                  <button
                    className="feature-action-btn danger"
                    onClick={() => handleDeleteFeature(feature.id)}
                    title={t('feature.btn.delete')}
                    aria-label={t('feature.btn.delete')}
                  ><IcoDelete /></button>
                </div>
              </div>
              <div className="feature-card-body">
                <div className="feature-param">
                  <span className="feature-param-label">{t('feature.type')}</span>
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
                  <div className="feature-param feature-suppressed-tag">{t('feature.suppressed_tag')}</div>
                )}
                {feature.dirty && (
                  <div className="feature-param feature-dirty-tag">{t('feature.dirty_tag')}</div>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <div className="feature-rebuild-section">
        <button className="feature-rebuild-btn" onClick={handleRebuild}>
          <IcoRefresh /> Rebuild All
        </button>
      </div>
    </div>
  );
}
