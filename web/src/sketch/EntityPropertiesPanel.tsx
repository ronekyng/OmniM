import { useState } from 'react';
import type { SketchEntity } from '../shared/types';
import { api } from '../shared/api';

type Props = {
  entity: SketchEntity;
  entityIdx: number;
  onUpdate: () => void;
};

export function EntityPropertiesPanel({ entity, entityIdx, onUpdate }: Props) {
  const [editing, setEditing] = useState(false);

  const updatePoint = async (ptIdx: number, field: 'x' | 'y' | 'z', value: string) => {
    const num = parseFloat(value);
    if (isNaN(num)) return;
    const newPoints = entity.points.map((p, i) =>
      i === ptIdx ? { ...p, [field]: num } : { ...p }
    );
    await api.updateSketchEntity(entity.id, entityIdx, { points: newPoints });
    onUpdate();
  };

  const updateRadius = async (value: string) => {
    const num = parseFloat(value);
    if (isNaN(num) || num <= 0) return;
    await api.updateSketchEntity(entity.id, entityIdx, { radius: num });
    onUpdate();
  };

  const updateSides = async (value: string) => {
    const num = parseInt(value, 10);
    if (isNaN(num) || num < 3) return;
    await api.updateSketchEntity(entity.id, entityIdx, { sides: num });
    onUpdate();
  };

  const updateZ = async (value: string) => {
    const num = parseFloat(value);
    if (isNaN(num)) return;
    const newPoints = entity.points.map(p => ({ ...p, z: num }));
    await api.updateSketchEntity(entity.id, entityIdx, { points: newPoints });
    onUpdate();
  };

  const currentZ = entity.points.length > 0 ? entity.points[0].z : 0;

  return (
    <div className="entity-props-panel">
      <div className="entity-props-header">
        <span className="entity-props-title">{entity.entity_type}</span>
        <button className="entity-props-close" onClick={() => setEditing(false)}>×</button>
      </div>

      <div className="entity-props-body">
        <div className="entity-props-row">
          <label>Entity #</label>
          <span>{entityIdx}</span>
        </div>

        <div className="entity-props-row">
          <label>Z Height</label>
          <input
            type="number"
            step="0.1"
            value={currentZ}
            onChange={(e) => updateZ(e.target.value)}
          />
        </div>

        {(entity.entity_type === 'Circle' || entity.entity_type === 'Arc') && (
          <div className="entity-props-row">
            <label>Radius</label>
            <input
              type="number"
              step="0.1"
              value={entity.radius}
              onChange={(e) => updateRadius(e.target.value)}
            />
          </div>
        )}

        {entity.entity_type === 'Polygon' && (
          <div className="entity-props-row">
            <label>Sides</label>
            <input
              type="number"
              min="3"
              step="1"
              value={entity.sides}
              onChange={(e) => updateSides(e.target.value)}
            />
          </div>
        )}

        {entity.entity_type === 'Ellipse' && entity.points.length >= 3 && (
          <>
            <div className="entity-props-section">Points</div>
            <div className="entity-props-row">
              <label>Center</label>
              <div className="entity-props-inputs">
                <input type="number" step="0.1" value={entity.points[0].x}
                  onChange={(e) => updatePoint(0, 'x', e.target.value)} />
                <input type="number" step="0.1" value={entity.points[0].y}
                  onChange={(e) => updatePoint(0, 'y', e.target.value)} />
              </div>
            </div>
            <div className="entity-props-row">
              <label>Width pt</label>
              <div className="entity-props-inputs">
                <input type="number" step="0.1" value={entity.points[1].x}
                  onChange={(e) => updatePoint(1, 'x', e.target.value)} />
                <input type="number" step="0.1" value={entity.points[1].y}
                  onChange={(e) => updatePoint(1, 'y', e.target.value)} />
              </div>
            </div>
            <div className="entity-props-row">
              <label>Height pt</label>
              <div className="entity-props-inputs">
                <input type="number" step="0.1" value={entity.points[2].x}
                  onChange={(e) => updatePoint(2, 'x', e.target.value)} />
                <input type="number" step="0.1" value={entity.points[2].y}
                  onChange={(e) => updatePoint(2, 'y', e.target.value)} />
              </div>
            </div>
          </>
        )}

        {(entity.entity_type === 'Line' || entity.entity_type === 'Rectangle' ||
          entity.entity_type === 'Polygon' || entity.entity_type === 'Spline' ||
          entity.entity_type === 'Polyline') && entity.points.length > 0 && (
          <>
            <div className="entity-props-section">Points</div>
            {entity.points.map((pt, i) => (
              <div key={i} className="entity-props-row">
                <label>P{i + 1}</label>
                <div className="entity-props-inputs">
                  <input type="number" step="0.1" value={pt.x}
                    onChange={(e) => updatePoint(i, 'x', e.target.value)} />
                  <input type="number" step="0.1" value={pt.y}
                    onChange={(e) => updatePoint(i, 'y', e.target.value)} />
                </div>
              </div>
            ))}
          </>
        )}

        {entity.entity_type === 'Circle' && entity.points.length >= 1 && (
          <>
            <div className="entity-props-section">Center</div>
            <div className="entity-props-row">
              <label>P1</label>
              <div className="entity-props-inputs">
                <input type="number" step="0.1" value={entity.points[0].x}
                  onChange={(e) => updatePoint(0, 'x', e.target.value)} />
                <input type="number" step="0.1" value={entity.points[0].y}
                  onChange={(e) => updatePoint(0, 'y', e.target.value)} />
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
