import type { SketchConstraintType } from '../shared/types';

export type FeatureType = 'extrude' | 'revolve' | 'sweep' | 'loft' | 'fusion' | 'lofted_fusion' | 'boolean' | 'chamfer' | 'fillet' | 'pattern' | 'shell';

type Props = {
  activeTool: string;
  activeConstraint: SketchConstraintType | null;
  solving: boolean;
  hasSketch: boolean;
  hasSelection: boolean;
  creatingFeature: FeatureType | null;
  creatingFeatureSketches: string[] | null;
  activeSketchIdx: number;
  snapEnabled: boolean;
  constructionMode: boolean;
  currentSketchZ: number;
  featureDepth: number;
  featureAngle: number;
  offsetDistance: number;
  canUndo: boolean;
  canRedo: boolean;
  onSwitchSketch: (idx: number) => void;
  onSelectTool: (tool: string) => void;
  onSelectConstraint: (ct: SketchConstraintType) => void;
  onSolve: () => void;
  onDeleteEntity: () => void;
  onAddFeature: (type: FeatureType) => void;
  onFinishDrawing: () => void;
  onCancelCreating: () => void;
  onToggleSnap: () => void;
  onToggleConstruction: () => void;
  onMirror: () => void;
  onOffset: () => void;
  onUpdateOffsetDistance: (d: number) => void;
  onUndo: () => void;
  onRedo: () => void;
  onUpdateSketchZ: (z: number) => void;
  onUpdateFeatureDepth: (d: number) => void;
  onUpdateFeatureAngle: (a: number) => void;
};

const CONSTRAINTS: { key: SketchConstraintType; label: string }[] = [
  { key: 'Horizontal', label: 'H' },
  { key: 'Vertical', label: 'V' },
  { key: 'Coincident', label: '⦿' },
  { key: 'Parallel', label: '∥' },
  { key: 'Perpendicular', label: '⊥' },
  { key: 'Length', label: 'L=' },
  { key: 'Distance', label: 'D=' },
  { key: 'Angle', label: '∠' },
  { key: 'Radius', label: 'R' },
  { key: 'Diameter', label: 'Ø' },
  { key: 'Fixed', label: '⚓' },
  { key: 'Tangent', label: '⌒' },
  { key: 'AtMidpoint', label: '◉' },
  { key: 'EqualLength', label: 'L≃' },
  { key: 'EqualRadius', label: 'R≃' },
  { key: 'Symmetric', label: '↔' },
  { key: 'PtOnLine', label: '⊞' },
  { key: 'PtOnCircle', label: '⊟' },
  { key: 'SameOrientation', label: '↗' },
];

const TOOLS = [
  { key: 'select' as const, label: 'Select', shortcut: 'V' },
  { key: 'line' as const, label: 'Line', shortcut: 'L' },
  { key: 'circle' as const, label: 'Circle', shortcut: 'C' },
  { key: 'arc' as const, label: 'Arc', shortcut: 'A' },
  { key: 'rect' as const, label: 'Rect', shortcut: 'R' },
  { key: 'polygon' as const, label: 'Polygon', shortcut: 'P' },
  { key: 'ellipse' as const, label: 'Ellipse', shortcut: 'E' },
  { key: 'spline' as const, label: 'Spline', shortcut: 'S' },
  { key: 'polyline' as const, label: 'Polyline', shortcut: 'N' },
  { key: 'trim' as const, label: 'Trim', shortcut: 'T' },
  { key: 'extend' as const, label: 'Extend', shortcut: 'X' },
];

const FEATURE_BUTTONS: { type: FeatureType; icon: string; label: string; tooltip: string }[] = [
  { type: 'extrude', icon: '⬍', label: '拉伸', tooltip: '拉伸 - 从轮廓创建实体' },
  { type: 'revolve', icon: '↻', label: '旋转', tooltip: '旋转 - 绕轴旋转轮廓' },
  { type: 'sweep', icon: '⟿', label: '扫掠', tooltip: '扫掠 - 沿路径移动轮廓' },
  { type: 'loft', icon: 'ℳ', label: '放样', tooltip: '放样 - 连接多个截面' },
  { type: 'fusion', icon: '⊕', label: '融合', tooltip: '融合 - 两个截面之间的过渡实体 (如大小头、天圆地方)' },
  { type: 'lofted_fusion', icon: 'ℳ⊕', label: '放样融合', tooltip: '放样融合 - 两个截面沿路径渐变过渡' },
  { type: 'boolean', icon: '⊖', label: '布尔', tooltip: '布尔 - 两个实体运算 (并集/差集/交集)' },
  { type: 'chamfer', icon: '⊿', label: '倒角', tooltip: '倒角 - 边缘倒角' },
  { type: 'fillet', icon: '⌒', label: '圆角', tooltip: '圆角 - 边缘圆角' },
  { type: 'pattern', icon: '⊞', label: '阵列', tooltip: '阵列 - 复制排列' },
  { type: 'shell', icon: '⊡', label: '抽壳', tooltip: '抽壳 - 变薄壁' },
];

export function EditorToolbar({
  activeTool, solving, hasSketch, hasSelection, creatingFeature,
  creatingFeatureSketches, activeSketchIdx, snapEnabled, constructionMode,
  currentSketchZ, featureDepth, featureAngle, offsetDistance, canUndo, canRedo,
  onSwitchSketch,
  onSelectTool, onSelectConstraint, onSolve,
  onDeleteEntity, onAddFeature, onFinishDrawing, onCancelCreating, onToggleSnap, onToggleConstruction,
  onMirror, onOffset, onUpdateOffsetDistance,
  onUndo, onRedo,
  onUpdateSketchZ, onUpdateFeatureDepth, onUpdateFeatureAngle,
}: Props) {
  return (
    <div className="sketch-toolbar-wrapper">
      {/* Row 1: Feature type buttons (创建实体) */}
      <div className="sketch-toolbar feature-ribbon">
        <div className="sketch-tool-group">
          {FEATURE_BUTTONS.map((f) => (
            <button
              key={f.type}
              className="sketch-tool-btn feature-ribbon-btn"
              onClick={() => onAddFeature(f.type)}
              disabled={!!creatingFeature}
              title={f.tooltip}
            >
              <span className="feature-ribbon-icon">{f.icon}</span>
              <span className="feature-ribbon-label">{f.label}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Row 2: Entity tools + Constraints + Actions (草图) */}
      <div className="sketch-toolbar">
        {/* Entity tools */}
        <div className="sketch-tool-group">
          {TOOLS.map((t) => (
            <button
              key={t.key}
              className={`sketch-tool-btn ${activeTool === t.key ? 'active' : ''}`}
              onClick={() => onSelectTool(t.key)}
              title={t.label}
            >
              {t.label}
            </button>
          ))}
        </div>

        <span className="sketch-tool-sep" />

        {/* Constraint tools */}
        <div className="sketch-tool-group">
          {CONSTRAINTS.map((c) => (
            <button
              key={c.key}
              className="sketch-tool-btn sketch-constraint-btn"
              onClick={() => onSelectConstraint(c.key)}
              disabled={!hasSelection || !!creatingFeature}
              title={c.key}
            >
              {c.label}
            </button>
          ))}
        </div>

        <span className="sketch-tool-sep" />

        {/* Snap toggle */}
        <button
          className={`sketch-tool-btn ${snapEnabled ? 'active' : ''}`}
          onClick={onToggleSnap}
          title="Toggle snap (S)"
        >
          {snapEnabled ? '🧲' : '🧲'}
          <span style={{ fontSize: 10, marginLeft: 2 }}>Snap</span>
        </button>

        {/* Construction toggle */}
        <button
          className={`sketch-tool-btn ${constructionMode ? 'active' : ''}`}
          onClick={onToggleConstruction}
          title="Toggle construction mode - draw reference geometry (dashed lines)"
          style={{ color: constructionMode ? '#88bbff' : '#888' }}
        >
          <span style={{ fontSize: 10, marginRight: 2 }}>‒‒</span>
          <span style={{ fontSize: 10 }}>Const</span>
        </button>

        <span className="sketch-tool-sep" />

        {/* Undo/Redo */}
        <div className="sketch-tool-group">
          <button
            className="sketch-tool-btn"
            onClick={onUndo}
            disabled={!canUndo}
            title="Undo (Ctrl+Z)"
          >↩</button>
          <button
            className="sketch-tool-btn"
            onClick={onRedo}
            disabled={!canRedo}
            title="Redo"
          >↪</button>
        </div>

        <span className="sketch-tool-sep" />

        {/* Actions */}
        <div className="sketch-tool-group">
          {creatingFeature ? (
            <>
              {creatingFeatureSketches && creatingFeatureSketches.length > 1 && (
                <div className="sketch-tool-group" style={{ marginRight: 6 }}>
                  {creatingFeatureSketches.map((label, idx) => (
                    <button
                      key={idx}
                      className={`sketch-tool-btn ${idx === activeSketchIdx ? 'active' : ''}`}
                      onClick={() => onSwitchSketch(idx)}
                      title={`切换到${label}`}
                    >
                      {label}
                    </button>
                  ))}
                </div>
              )}
              <button
                className="sketch-tool-btn danger"
                onClick={onDeleteEntity}
                disabled={!hasSelection}
              >
                Del
              </button>
              <button
                className="sketch-tool-btn sketch-solve-btn"
                onClick={onFinishDrawing}
                style={{ background: '#2e7d32' }}
              >
                ✓ Done
              </button>
              <button
                className="sketch-tool-btn danger"
                onClick={onCancelCreating}
              >
                ✗ Cancel
              </button>
            </>
          ) : (
            <>
              <button
                className="sketch-tool-btn sketch-solve-btn"
                onClick={onSolve}
                disabled={!hasSketch || solving}
              >
                {solving ? 'Solving...' : 'Solve'}
              </button>
              <button
                className="sketch-tool-btn"
                onClick={onMirror}
                disabled={!hasSelection}
                title="Mirror selected entity across first axis line"
              >↔</button>
              <button
                className="sketch-tool-btn"
                onClick={onOffset}
                disabled={!hasSelection}
                title="Offset selected line"
              >∥</button>
              <input
                type="number"
                step="0.1"
                value={offsetDistance}
                onChange={(e) => onUpdateOffsetDistance(parseFloat(e.target.value) || 1)}
                style={{ width: 50, background: '#0d0f14', color: '#e0e0e0', border: '1px solid #3a3d4a', borderRadius: 3, padding: '2px 4px', fontSize: 11, fontFamily: 'monospace' }}
                title="Offset distance"
              />
              <button
                className="sketch-tool-btn danger"
                onClick={onDeleteEntity}
                disabled={!hasSelection}
              >
                Del
              </button>
            </>
          )}
        </div>
      </div>

      {/* Row 3: Feature params (Z-height, depth, angle) */}
      {creatingFeature && (
        <div className="sketch-toolbar feature-ribbon" style={{ padding: '4px 16px', display: 'flex', alignItems: 'center', gap: 12 }}>
          <div className="sketch-tool-group" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <label style={{ color: '#888', fontSize: 11 }}>Z标高:</label>
            <input
              type="number"
              step="0.1"
              value={currentSketchZ}
              onChange={(e) => onUpdateSketchZ(parseFloat(e.target.value) || 0)}
              style={{ width: 60, background: '#0d0f14', color: '#e0e0e0', border: '1px solid #3a3d4a', borderRadius: 3, padding: '2px 4px', fontSize: 11, fontFamily: 'monospace' }}
            />
          </div>
          {creatingFeature === 'extrude' && (
            <div className="sketch-tool-group" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <label style={{ color: '#888', fontSize: 11 }}>拉伸深度:</label>
              <input
                type="number"
                step="0.1"
                value={featureDepth}
                onChange={(e) => onUpdateFeatureDepth(parseFloat(e.target.value) || 1)}
                style={{ width: 60, background: '#0d0f14', color: '#e0e0e0', border: '1px solid #3a3d4a', borderRadius: 3, padding: '2px 4px', fontSize: 11, fontFamily: 'monospace' }}
              />
            </div>
          )}
          {creatingFeature === 'revolve' && (
            <div className="sketch-tool-group" style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <label style={{ color: '#888', fontSize: 11 }}>旋转角度(°):</label>
              <input
                type="number"
                step="1"
                value={featureAngle}
                onChange={(e) => onUpdateFeatureAngle(parseFloat(e.target.value) || 0)}
                style={{ width: 60, background: '#0d0f14', color: '#e0e0e0', border: '1px solid #3a3d4a', borderRadius: 3, padding: '2px 4px', fontSize: 11, fontFamily: 'monospace' }}
              />
            </div>
          )}
          {creatingFeature === 'sweep' && (
            <span style={{ color: '#ffa726', fontSize: 11 }}>扫掠: 绘制轮廓 → 切换到路径</span>
          )}
          {(creatingFeature === 'loft' || creatingFeature === 'fusion') && (
            <span style={{ color: '#ffa726', fontSize: 11 }}>放样/融合: 绘制截面 → 切换到截面2</span>
          )}
          {creatingFeature === 'lofted_fusion' && (
            <span style={{ color: '#ffa726', fontSize: 11 }}>放样融合: 绘制截面1 → 截面2 → 路径</span>
          )}
        </div>
      )}
    </div>
  );
}
