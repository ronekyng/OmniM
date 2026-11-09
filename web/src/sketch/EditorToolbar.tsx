import type { SketchConstraintType } from '../shared/types';
import { Glyph, IcoCheck, IcoClose, IcoMagnet } from '../app/components/Icons';
import { useI18n } from '../shared/i18n';

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

const CONSTRAINTS: { key: SketchConstraintType; label: string; titleKey: string }[] = [
  { key: 'Horizontal', label: 'H', titleKey: 'sketch.c_horizontal' },
  { key: 'Vertical', label: 'V', titleKey: 'sketch.c_vertical' },
  { key: 'Coincident', label: '⦿', titleKey: 'sketch.c_coincident' },
  { key: 'Parallel', label: '∥', titleKey: 'sketch.c_parallel' },
  { key: 'Perpendicular', label: '⊥', titleKey: 'sketch.c_perpendicular' },
  { key: 'Length', label: 'L=', titleKey: 'sketch.c_length' },
  { key: 'Distance', label: 'D=', titleKey: 'sketch.c_distance' },
  { key: 'Angle', label: '∠', titleKey: 'sketch.c_angle' },
  { key: 'Radius', label: 'R', titleKey: 'sketch.c_radius' },
  { key: 'Diameter', label: 'Ø', titleKey: 'sketch.c_diameter' },
  { key: 'Fixed', label: 'FIX', titleKey: 'sketch.c_fixed' },
  { key: 'Tangent', label: '⌒', titleKey: 'sketch.c_tangent' },
  { key: 'AtMidpoint', label: 'MID', titleKey: 'sketch.c_atmidpoint' },
  { key: 'EqualLength', label: 'L≃', titleKey: 'sketch.c_equallength' },
  { key: 'EqualRadius', label: 'R≃', titleKey: 'sketch.c_equalradius' },
  { key: 'Symmetric', label: '↔', titleKey: 'sketch.c_symmetric' },
  { key: 'PtOnLine', label: '⊞', titleKey: 'sketch.c_ptonline' },
  { key: 'PtOnCircle', label: '⊟', titleKey: 'sketch.c_ptoncircle' },
  { key: 'SameOrientation', label: '↗', titleKey: 'sketch.c_sameorientation' },
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

const FEATURE_BUTTONS: { type: FeatureType; labelKey: string; tooltipKey: string }[] = [
  { type: 'extrude', labelKey: 'sketch.f_extrude', tooltipKey: 'sketch.tt_extrude' },
  { type: 'revolve', labelKey: 'sketch.f_revolve', tooltipKey: 'sketch.tt_revolve' },
  { type: 'sweep', labelKey: 'sketch.f_sweep', tooltipKey: 'sketch.tt_sweep' },
  { type: 'loft', labelKey: 'sketch.f_loft', tooltipKey: 'sketch.tt_loft' },
  { type: 'fusion', labelKey: 'sketch.f_fusion', tooltipKey: 'sketch.tt_fusion' },
  { type: 'lofted_fusion', labelKey: 'sketch.f_lofted_fusion', tooltipKey: 'sketch.tt_lofted_fusion' },
  { type: 'boolean', labelKey: 'sketch.f_boolean', tooltipKey: 'sketch.tt_boolean' },
  { type: 'chamfer', labelKey: 'sketch.f_chamfer', tooltipKey: 'sketch.tt_chamfer' },
  { type: 'fillet', labelKey: 'sketch.f_fillet', tooltipKey: 'sketch.tt_fillet' },
  { type: 'pattern', labelKey: 'sketch.f_pattern', tooltipKey: 'sketch.tt_pattern' },
  { type: 'shell', labelKey: 'sketch.f_shell', tooltipKey: 'sketch.tt_shell' },
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
  const { t } = useI18n();
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
              title={t(f.tooltipKey as never)}
            >
              <span className="feature-ribbon-icon"><Glyph name={f.type} /></span>
              <span className="feature-ribbon-label">{t(f.labelKey as never)}</span>
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
              title={`${t(c.titleKey as never)} (${c.key})`}
              aria-label={`${t(c.titleKey as never)}${t('sketch.constraint_suffix')}`}
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
          <IcoMagnet />
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
                title={t('sketch.delete_entities')}
                aria-label={t('sketch.delete_entities_a')}
              >
                Del
              </button>
              <button
                className="sketch-tool-btn sketch-solve-btn"
                onClick={onFinishDrawing}
                style={{ background: '#2e7d32' }}
              >
                <IcoCheck /> Done
              </button>
              <button
                className="sketch-tool-btn danger"
                onClick={onCancelCreating}
              >
                <IcoClose /> Cancel
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
              <label style={{ color: '#888', fontSize: 11 }}>{t('sketch.offset_m')}</label>
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
                title={t('sketch.delete_entities')}
                aria-label={t('sketch.delete_entities_a')}
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
            <label style={{ color: '#888', fontSize: 11 }}>{t('sketch.z_elev')}</label>
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
              <label style={{ color: '#888', fontSize: 11 }}>{t('sketch.extrude_depth')}</label>
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
              <label style={{ color: '#888', fontSize: 11 }}>{t('sketch.revolve_angle')}</label>
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
            <span style={{ color: '#ffa726', fontSize: 11 }}>{t('sketch.hint_sweep')}</span>
          )}
          {(creatingFeature === 'loft' || creatingFeature === 'fusion') && (
            <span style={{ color: '#ffa726', fontSize: 11 }}>{t('sketch.hint_loft')}</span>
          )}
          {creatingFeature === 'lofted_fusion' && (
            <span style={{ color: '#ffa726', fontSize: 11 }}>{t('sketch.hint_lofted_fusion')}</span>
          )}
        </div>
      )}
    </div>
  );
}