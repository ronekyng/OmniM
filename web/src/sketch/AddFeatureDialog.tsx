import { useState } from 'react';
import type { FeatureItem, SketchResponse } from '../shared/types';

type Props = {
  feature: FeatureItem;
  sketches: SketchResponse[];
  onConfirm: (params: Record<string, unknown>) => void;
  onCancel: () => void;
};

const FEATURE_FIELDS: Record<string, { key: string; label: string; type: 'number' | 'text' | 'select'; default: number | string; options?: string[] }[]> = {
  extrude: [
    { key: 'depth', label: '拉伸深度 (Depth)', type: 'number', default: 1.0 },
    { key: 'direction_x', label: '方向 X', type: 'number', default: 0.0 },
    { key: 'direction_y', label: '方向 Y', type: 'number', default: 0.0 },
    { key: 'direction_z', label: '方向 Z', type: 'number', default: 1.0 },
  ],
  revolve: [
    { key: 'angle', label: '旋转角度 (弧度, 0=360°)', type: 'number', default: 0.0 },
  ],
  sweep: [
    { key: 'path_sketch_id', label: '路径 Sketch', type: 'select', default: '', options: [] },
  ],
  loft: [
    { key: 'section_sketch_ids', label: '截面 Sketch (逗号分隔)', type: 'text', default: '' },
  ],
  fusion: [],
  lofted_fusion: [],
  boolean: [
    { key: 'operation', label: '运算类型', type: 'select', default: 'subtract', options: ['union', 'subtract', 'intersect'] },
  ],
  chamfer: [
    { key: 'distance', label: '倒角距离', type: 'number', default: 0.1 },
  ],
  fillet: [
    { key: 'radius', label: '圆角半径', type: 'number', default: 0.1 },
  ],
  pattern: [
    { key: 'pattern_type', label: '阵列类型', type: 'select', default: 'linear', options: ['linear', 'circular'] },
    { key: 'count', label: '数量', type: 'number', default: 2 },
    { key: 'spacing', label: '间距', type: 'number', default: 1.0 },
  ],
  shell: [
    { key: 'wall_thickness', label: '壁厚', type: 'number', default: 0.1 },
  ],
};

export function AddFeatureDialog({ feature, sketches, onConfirm, onCancel }: Props) {
  const isEdit = !!feature.id;
  const fields = FEATURE_FIELDS[feature.feature_type] || [];

  const CUTTABLE_TYPES = ['extrude', 'revolve', 'sweep', 'loft', 'fusion', 'lofted_fusion'];

  const [vals, setVals] = useState<Record<string, string>>(() => {
    const init: Record<string, string> = {};
    // 预填当前 feature 的参数
    if (feature.feature_type === 'extrude') {
      init.depth = String(feature.depth);
      init.direction_x = '0';
      init.direction_y = '0';
      init.direction_z = '1';
    } else if (feature.feature_type === 'revolve') {
      init.angle = String(feature.angle);
    } else if (feature.feature_type === 'sweep') {
      init.path_sketch_id = '';
    } else if (feature.feature_type === 'loft') {
      init.section_sketch_ids = '';
    } else if (feature.feature_type === 'boolean') {
      init.operation = 'subtract';
    } else {
      fields.forEach(f => { init[f.key] = String(f.default); });
    }
    return init;
  });

  const [profileSketchId, setProfileSketchId] = useState(feature.sketch_id || '');
  const [isCut, setIsCut] = useState(feature.is_cut || false);
  const sketchPreset = !!feature.sketch_id;

  const handleConfirm = () => {
    const params: Record<string, unknown> = {
      feature_type: feature.feature_type,
      sketch_id: profileSketchId,
      is_cut: isCut,
    };

    // 合并表单参数
    for (const f of fields) {
      if (f.type === 'number') {
        params[f.key] = parseFloat(vals[f.key]) || f.default;
      } else {
        params[f.key] = vals[f.key] || f.default;
      }
    }

    // 特殊类型参数映射: chamfer/fillet/pattern/shell 使用 depth 字段
    if (feature.feature_type === 'chamfer') {
      params.depth = parseFloat(vals['distance']) || 0.1;
    }
    if (feature.feature_type === 'fillet') {
      params.depth = parseFloat(vals['radius']) || 0.1;
    }
    if (feature.feature_type === 'pattern') {
      const count = parseInt(vals['count']) || 2;
      const spacing = parseFloat(vals['spacing']) || 1.0;
      params.count_x = count;
      params.count_y = 1;
      params.count_z = 1;
      params.spacing_x = spacing;
      params.spacing_y = spacing;
      params.spacing_z = 0;
      params.depth = spacing;
      params.angle = count as number;
      // 将 dependency_ids 设为空（将在服务器端通过顺序确定）
      params.dependency_ids = feature.id ? [feature.id] : [];
    }
    if (feature.feature_type === 'shell') {
      params.depth = parseFloat(vals['wall_thickness']) || 0.1;
    }

    // revolve 的 axis_sketch_id 从 feature 传入
    if (feature.feature_type === 'revolve' && feature.axis_sketch_id) {
      params.axis_sketch_id = feature.axis_sketch_id;
    }

    // loft 和 lofted_fusion 的 input_sketch_ids 从 feature 传入
    if ((feature.feature_type === 'loft' || feature.feature_type === 'lofted_fusion') && feature.input_sketch_ids.length > 0) {
      params.input_sketch_ids = feature.input_sketch_ids;
    }

    // fusion 的第二个截面从 input_sketch_ids[0] 传入
    if (feature.feature_type === 'fusion' && feature.input_sketch_ids.length > 0) {
      params.input_sketch_ids = feature.input_sketch_ids;
    }

    onConfirm(params);
  };

  // 过滤出可用的 sketch（排除当前选中的 profile sketch 用于 sweep path）
  const availableSketches = feature.feature_type === 'sweep'
    ? sketches.filter(s => s.id !== profileSketchId)
    : sketches;

  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h3>{isEdit ? '编辑' : '添加'} {feature.feature_type}</h3>

        {/* Profile Sketch — 只在无预设时显示选择器 */}
        {sketchPreset ? (
          <div className="dialog-row">
            <label>轮廓 Sketch</label>
            <span className="dialog-readonly">
              {sketches.find(s => s.id === profileSketchId)?.name ?? profileSketchId}
            </span>
          </div>
        ) : (
          <div className="dialog-row">
            <label>轮廓 Sketch (Profile)</label>
            <select
              value={profileSketchId}
              onChange={(e) => setProfileSketchId(e.target.value)}
            >
              <option value="">-- 选择 Sketch --</option>
              {sketches.map(s => (
                <option key={s.id} value={s.id}>{s.name} ({s.entity_count} entities)</option>
              ))}
            </select>
          </div>
        )}

        {/* Axis Sketch for revolve */}
        {feature.feature_type === 'revolve' && feature.axis_sketch_id && (
          <div className="dialog-row">
            <label>轴线 Sketch (Axis)</label>
            <span className="dialog-readonly">
              {sketches.find(s => s.id === feature.axis_sketch_id)?.name ?? feature.axis_sketch_id}
            </span>
          </div>
        )}

        {/* 动态字段 */}
        {fields.map(f => (
          <div key={f.key} className="dialog-row">
            <label>{f.label}</label>
            {f.type === 'select' ? (
              <select
                value={vals[f.key] || ''}
                onChange={(e) => setVals(v => ({ ...v, [f.key]: e.target.value }))}
              >
                {(f.key === 'path_sketch_id' ? availableSketches : []).map(s => (
                  <option key={s.id} value={s.id}>{s.name}</option>
                ))}
                {f.options && f.options.map(opt => (
                  <option key={opt} value={opt}>{opt}</option>
                ))}
              </select>
            ) : (
              <input
                type={f.type}
                value={vals[f.key] || ''}
                onChange={(e) => setVals(v => ({ ...v, [f.key]: e.target.value }))}
              />
            )}
          </div>
        ))}

        {/* 空心 (Cut) toggle for applicable feature types */}
        {CUTTABLE_TYPES.includes(feature.feature_type) && (
          <div className="dialog-row">
            <label>
              <input
                type="checkbox"
                checked={isCut}
                onChange={(e) => setIsCut(e.target.checked)}
                style={{ marginRight: 6 }}
              />
              空心 (Cut) — 移除材料而非添加
            </label>
          </div>
        )}

        <div className="dialog-actions">
          <button onClick={onCancel}>取消</button>
          <button className="accent" onClick={handleConfirm} disabled={!profileSketchId && !sketchPreset}>
            {isEdit ? '保存' : '添加'}
          </button>
        </div>
      </div>
    </div>
  );
}
