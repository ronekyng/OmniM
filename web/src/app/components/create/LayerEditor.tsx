import { useEffect, useState } from 'react';
import type { LayerDefInput } from '../../../shared/types';
import { MATERIAL_CATEGORY_NAMES } from '../../../shared/types';
import { api } from '../../../shared/api';
import type { Material } from '../../../shared/types';
import { DEFAULT_LAYERS_WALL, DEFAULT_LAYERS_SLAB } from '../../../shared/types/creation';
import { MaterialBrowser } from './MaterialBrowser';

type Props = {
  value: LayerDefInput[];
  /** Whether the layers stack along the wall normal (wall/roof) or vertically (slab/floor). */
  mode: 'wall' | 'slab';
  onChange: (layers: LayerDefInput[]) => void;
};

const FUNCTION_LABELS: Record<string, string> = {
  structural: '承重',
  insulation: '保温',
  finish: '饰面',
  membrane: '防水',
  vapor: '隔汽',
  cladding: '覆面',
  waterproof: '防水',
  finish_floor: '面层',
  finish_ceiling: '吊顶',
};

const WALL_TEMPLATES: { name: string; layers: () => LayerDefInput[] }[] = [
  {
    name: '外墙（保温）',
    layers: () => [
      { name: '内饰面', thickness: 20, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
      { name: '结构层', thickness: 200, material_id: 'C30/37', material_name: 'C30/37 混凝土', function: 'structural', color: '#b0aaa0' },
      { name: '保温层', thickness: 60, material_id: 'XPS-100', material_name: '挤塑聚苯板 XPS', function: 'insulation', color: '#f0d060' },
      { name: '外饰面', thickness: 20, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#c9c2b4' },
    ],
  },
  {
    name: '内墙',
    layers: () => DEFAULT_LAYERS_WALL.map((l) => ({ ...l })),
  },
  {
    name: '隔墙',
    layers: () => [
      { name: '面层', thickness: 15, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
      { name: '砌块', thickness: 100, material_id: 'Block', material_name: '加气混凝土砌块', function: 'structural', color: '#b8b8b4' },
      { name: '面层', thickness: 15, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
    ],
  },
];

const SLAB_TEMPLATES: { name: string; layers: () => LayerDefInput[] }[] = [
  {
    name: '标准楼板',
    layers: () => DEFAULT_LAYERS_SLAB.map((l) => ({ ...l })),
  },
  {
    name: '保温屋面',
    layers: () => [
      { name: '结构层', thickness: 200, material_id: 'C30/37', material_name: 'C30/37 混凝土', function: 'structural', color: '#b0aaa0' },
      { name: '找坡层', thickness: 50, material_id: 'Mortar', material_name: '水泥砂浆', function: 'finish', color: '#cfc9bd' },
      { name: '保温层', thickness: 80, material_id: 'XPS-100', material_name: '挤塑聚苯板 XPS', function: 'insulation', color: '#f0d060' },
      { name: '防水层', thickness: 4, material_id: 'XPS-100', material_name: 'SBS 防水卷材', function: 'waterproof', color: '#5a5a5e' },
    ],
  },
];

/** Layer structure editor for multi-layer composite members (dev plan §2.1).
 *  Supports: add/remove layers, thickness sliders, material selection,
 *  function tagging, templates, and a visual section preview. */
export function LayerEditor({ value, mode, onChange }: Props) {
  const [materials, setMaterials] = useState<Material[]>([]);
  const [expanded, setExpanded] = useState(true);
  const [pickerIndex, setPickerIndex] = useState<number | null>(null);

  useEffect(() => {
    api.listMaterials()
      .then(setMaterials)
      .catch(() => setMaterials([]));
  }, []);

  const materialById = (id: string): Material | undefined => materials.find((m) => m.id === id);

  const updateLayer = (index: number, patch: Partial<LayerDefInput>) => {
    onChange(value.map((l, i) => (i === index ? { ...l, ...patch } : l)));
  };

  const removeLayer = (index: number) => {
    onChange(value.filter((_, i) => i !== index));
  };

  const moveLayer = (index: number, dir: -1 | 1) => {
    const target = index + dir;
    if (target < 0 || target >= value.length) return;
    const next = [...value];
    [next[index], next[target]] = [next[target], next[index]];
    onChange(next);
  };

  const addLayer = () => {
    onChange([...value, {
      name: '新层',
      thickness: 20,
      material_id: 'C30/37',
      material_name: 'C30/37 混凝土',
      function: 'structural',
      color: '#9d968a',
    }]);
  };

  const applyTemplate = (layers: LayerDefInput[]) => {
    onChange(layers.map((l) => ({ ...l })));
  };

  const totalThickness = value.reduce((sum, l) => sum + (l.thickness || 0), 0);
  const templates = mode === 'wall' ? WALL_TEMPLATES : SLAB_TEMPLATES;

  return (
    <div className="layer-editor">
      <div className="layer-editor-header" onClick={() => setExpanded((v) => !v)}>
        <span className="layer-editor-title">层结构</span>
        <span className="layer-editor-meta">{value.length} 层 · {totalThickness} mm</span>
        <span className="layer-editor-toggle">{expanded ? '▾' : '▸'}</span>
      </div>

      {expanded && (
        <>
          <div className="layer-editor-templates">
            {templates.map((t) => (
              <button
                key={t.name}
                className="layer-template-btn"
                type="button"
                onClick={() => applyTemplate(t.layers())}
                title={`应用模板: ${t.name}`}
              >
                {t.name}
              </button>
            ))}
          </div>

          <div className="layer-section-preview">
            {value.length === 0 && <div className="layer-preview-empty">无层</div>}
            {value.map((l, i) => (
              <div
                key={i}
                className="layer-preview-slice"
                style={{
                  flex: l.thickness || 1,
                  background: l.color || '#999',
                  height: '100%',
                }}
                title={`${l.name} (${l.thickness}mm)`}
              />
            ))}
          </div>

          <div className="layer-list">
            {value.map((l, i) => (
              <div key={i} className="layer-card">
                <div className="layer-card-head">
                  <span className="layer-swatch" style={{ background: l.color || '#999' }} />
                  <span className="layer-card-title">{l.name || `层 ${i + 1}`}</span>
                  <div className="layer-card-actions">
                    <button type="button" className="layer-move-btn" disabled={i === 0} onClick={() => moveLayer(i, -1)} title="上移">▲</button>
                    <button type="button" className="layer-move-btn" disabled={i === value.length - 1} onClick={() => moveLayer(i, 1)} title="下移">▼</button>
                    <button type="button" className="layer-del-btn" onClick={() => removeLayer(i)} title="删除层">×</button>
                  </div>
                </div>

                <div className="layer-card-field">
                  <span className="layer-field-label">材料</span>
                  <select
                    className="layer-material-select"
                    value={l.material_id}
                    onChange={(e) => {
                      const mat = materialById(e.target.value);
                      updateLayer(i, { material_id: e.target.value, material_name: mat?.name || e.target.value, color: mat?.color ? `rgb(${Math.round(mat.color.r * 255)},${Math.round(mat.color.g * 255)},${Math.round(mat.color.b * 255)})` : l.color });
                    }}
                  >
                    <option value="">— 无 —</option>
                    {materials.map((m) => (
                      <option key={m.id} value={m.id}>
                        {m.name}（{MATERIAL_CATEGORY_NAMES[m.category] || m.category}）
                      </option>
                    ))}
                  </select>
                  <button type="button" className="layer-browse-btn" onClick={() => setPickerIndex(i)} title="浏览材料库">⋯</button>
                </div>

                <div className="layer-card-field">
                  <span className="layer-field-label">厚度</span>
                  <input
                    type="range"
                    className="layer-thickness-slider"
                    min={1}
                    max={400}
                    step={1}
                    value={l.thickness}
                    onChange={(e) => updateLayer(i, { thickness: parseInt(e.target.value, 10) })}
                  />
                  <input
                    type="number"
                    className="layer-thickness-input"
                    min={1}
                    max={1000}
                    value={l.thickness}
                    onChange={(e) => updateLayer(i, { thickness: parseInt(e.target.value || '0', 10) })}
                  />
                  <span className="layer-unit">mm</span>
                </div>

                <div className="layer-card-field">
                  <span className="layer-field-label">功能</span>
                  <select
                    className="layer-function-select"
                    value={l.function}
                    onChange={(e) => updateLayer(i, { function: e.target.value })}
                  >
                    <option value="structural">承重</option>
                    <option value="insulation">保温</option>
                    <option value="finish">饰面</option>
                    <option value="membrane">防水</option>
                    <option value="vapor">隔汽</option>
                    <option value="cladding">覆面</option>
                  </select>
                  <input
                    type="text"
                    className="layer-name-input"
                    placeholder="层名"
                    value={l.name}
                    onChange={(e) => updateLayer(i, { name: e.target.value })}
                  />
                </div>
              </div>
            ))}
          </div>

          <button type="button" className="layer-add-btn" onClick={addLayer}>+ 添加层</button>
        </>
      )}

      {pickerIndex !== null && (
        <div className="layer-picker-overlay" onClick={() => setPickerIndex(null)}>
          <div className="layer-picker-panel" onClick={(e) => e.stopPropagation()}>
            <MaterialBrowser
              selectedId={value[pickerIndex]?.material_id}
              onSelect={(mat) => {
                updateLayer(pickerIndex, {
                  material_id: mat.id,
                  material_name: mat.name,
                  color: mat.color ? `rgb(${Math.round(mat.color.r * 255)},${Math.round(mat.color.g * 255)},${Math.round(mat.color.b * 255)})` : value[pickerIndex]?.color,
                });
                setPickerIndex(null);
              }}
              onClose={() => setPickerIndex(null)}
            />
          </div>
        </div>
      )}
    </div>
  );
}
