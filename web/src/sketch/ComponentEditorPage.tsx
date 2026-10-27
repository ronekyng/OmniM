import { useState, useCallback, useEffect, useMemo } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import * as THREE from 'three';
import { api } from '../shared/api';
import type { Sketch, SketchConstraintType, SketchResponse, FeatureItem } from '../shared/types';
import { SketchCanvas2D } from './SketchCanvas2D';
import { SketchCanvas3D } from './SketchCanvas3D';
import { EditorToolbar } from './EditorToolbar';
import type { FeatureType } from './EditorToolbar';
import { FeaturePanel } from './FeaturePanel';
import { AddFeatureDialog } from './AddFeatureDialog';
import { EntityPropertiesPanel } from './EntityPropertiesPanel';

type Tool = 'select' | 'line' | 'circle' | 'arc' | 'rect' | 'polygon' | 'ellipse' | 'spline' | 'polyline' | 'trim' | 'extend';
type ConstraintTool = SketchConstraintType | null;
type CreatingFeature = { featureType: FeatureType; sketchIds: string[]; currentSketchIdx: number } | null;

const MULTI_SKETCH_LABELS: Record<string, string[]> = {
  revolve: ['轮廓', '轴线'],
  sweep: ['轮廓', '路径'],
  loft: ['截面1', '截面2'],
  fusion: ['截面1', '截面2'],
  lofted_fusion: ['截面1', '截面2', '路径'],
};

export function ComponentEditorPage() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [sketches, setSketches] = useState<SketchResponse[]>([]);
  const [currentSketch, setCurrentSketch] = useState<Sketch | null>(null);
  const [currentComponentId] = useState<string>(searchParams.get('component') || '');
  const [componentName, setComponentName] = useState<string>('');
  const [editingName, setEditingName] = useState(false);
  const [activeTool, setActiveTool] = useState<Tool>('select');
  const [activeConstraint, setActiveConstraint] = useState<ConstraintTool>(null);
  const [selectedEntityIdx, setSelectedEntityIdx] = useState<number | null>(null);
  const [solving, setSolving] = useState(false);
  const [pendingLineStart, setPendingLineStart] = useState<{ x: number; y: number } | null>(null);
  const [pendingPolyPoints, setPendingPolyPoints] = useState<{ x: number; y: number }[]>([]);
  const [pendingPolygonSides, setPendingPolygonSides] = useState(6);
  const [snapEnabled, setSnapEnabled] = useState(true);
  const [constructionMode, setConstructionMode] = useState(false);
  const [mousePos, setMousePos] = useState<{ x: number; y: number }>({ x: 0, y: 0 });
  const [tab, setTab] = useState<'component' | '3d'>('component');
  const [previewMesh, setPreviewMesh] = useState<{ positions: number[]; normals: number[]; indices: number[] } | null>(null);
  const [sectionLines, setSectionLines] = useState<{ x1: number; y1: number; x2: number; y2: number }[]>([]);
  const [features, setFeatures] = useState<FeatureItem[]>([]);
  const [editingFeature, setEditingFeature] = useState<FeatureItem | null>(null);
  const [creatingFeature, setCreatingFeature] = useState<CreatingFeature>(null);
  const [loadedSketches, setLoadedSketches] = useState<Record<string, Sketch>>({});
  const [featureDepth, setFeatureDepth] = useState(1.0);
  const [featureAngle, setFeatureAngle] = useState(360);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);
  const [editingConstraint, setEditingConstraint] = useState<{ sketchId: string; constraintIdx: number; value: number; type: string } | null>(null);
  const [offsetDistance, setOffsetDistance] = useState(1.0);
  const [sketchDof, setSketchDof] = useState<number | null>(null);

  const refreshUndoStatus = useCallback(async () => {
    try {
      const status = await api.undoStatus();
      setCanUndo(status.can_undo);
      setCanRedo(status.can_redo);
    } catch { /* ignore */ }
  }, []);

  const loadSketches = useCallback(async (componentId?: string) => {
    try {
      const list = await api.listSketches(componentId);
      setSketches(list);
    } catch { /* ignore */ }
  }, []);

  const loadFeatures = useCallback(async (componentId: string) => {
    try {
      const list = await api.listFeatures(componentId);
      setFeatures(list);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    if (currentComponentId) {
      loadSketches(currentComponentId);
      loadFeatures(currentComponentId);
      api.getComponent(currentComponentId).then(c => setComponentName(c.name)).catch(() => {});
    }
  }, [currentComponentId, loadSketches, loadFeatures]);

  // Auto-generate 3D preview when features change
  useEffect(() => {
    if (features.length > 0) {
      rebuildPreview();
    } else {
      setPreviewMesh(null);
    }
  }, [features]);

  // Clear sketch when its feature is deleted
  useEffect(() => {
    if (!currentSketch || creatingFeature) return;
    if (features.some(f => f.sketch_id === currentSketch.id)) return;
    setCurrentSketch(null);
    setSelectedEntityIdx(null);
  }, [features, currentSketch, creatingFeature]);

  // Fetch section lines for 2D background
  useEffect(() => {
    if (!currentComponentId || features.length === 0) { setSectionLines([]); return; }
    let cancelled = false;
    api.getSectionLines(currentComponentId, 0).then(lines => {
      if (!cancelled) {
        setSectionLines(lines);
      }
    }).catch(e => console.error('[section] error', e));
    return () => { cancelled = true; };
  }, [currentComponentId, features, api]);

  const loadSketch = useCallback(async (id: string) => {
    try {
      const s = await api.getSketch(id);
      setCurrentSketch(s);
      setSelectedEntityIdx(null);
      refreshUndoStatus();
    } catch { /* ignore */ }
  }, []);

  const solveAndLoad = useCallback(async (id: string) => {
    try {
      const result = await api.solveSketch(id);
      setSketchDof(result.dof);
      await loadSketch(id);
    } catch { /* ignore */ }
  }, [loadSketch]);

  const handleUndo = useCallback(async () => {
    try {
      await api.undo();
      refreshUndoStatus();
      if (currentSketch) loadSketch(currentSketch.id);
    } catch (e) {
      alert('撤销失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  }, [currentSketch]);

  const handleRedo = useCallback(async () => {
    try {
      await api.redo();
      refreshUndoStatus();
      if (currentSketch) loadSketch(currentSketch.id);
    } catch (e) {
      alert('重做失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  }, [currentSketch]);

  const handleEditConstraint = useCallback(async (constraintIdx: number) => {
    if (!currentSketch) return;
    const constraint = currentSketch.constraints[constraintIdx];
    if (!constraint) return;
    const dimTypes = ['Length', 'Distance', 'Angle', 'Radius', 'Diameter'];
    if (!dimTypes.includes(constraint.constraint_type)) return;
    setEditingConstraint({
      sketchId: currentSketch.id,
      constraintIdx,
      value: constraint.value,
      type: constraint.constraint_type,
    });
  }, [currentSketch]);

  const handleConstraintValueChange = async (newValue: number) => {
    if (!editingConstraint) return;
    try {
      const ct = editingConstraint.type;
      const lower = ct.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
      const constraint = currentSketch?.constraints[editingConstraint.constraintIdx];
      if (!constraint) return;
      const entityRefs: [number, number][] = constraint.entity_refs.map(r => [r.entity_index, r.point_index]);
      // Remove old constraint and re-add with new value
      await api.removeSketchConstraint(editingConstraint.sketchId, editingConstraint.constraintIdx);
      await api.addSketchConstraint(editingConstraint.sketchId, {
        constraint_type: lower,
        entity_refs: entityRefs,
        value: newValue,
      });
      setEditingConstraint(null);
      if (currentSketch) {
        await loadSketch(currentSketch.id);
        await solveAndLoad(currentSketch.id);
      }
    } catch (e) {
      alert('更新约束失败: ' + (e instanceof Error ? e.message : String(e)));
      setEditingConstraint(null);
    }
  };

  // Keyboard handler for Enter (finish poly/spline), Escape (cancel), Delete (delete selected entity), Ctrl+Z/Y (undo/redo)
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
        e.preventDefault();
        if (e.shiftKey) { handleRedo(); } else { handleUndo(); }
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key === 'y') {
        e.preventDefault();
        handleRedo();
        return;
      }
      if (e.key === 'Enter' && (activeTool === 'spline' || activeTool === 'polyline') && pendingPolyPoints.length >= 2) {
        handleFinishPoly();
      }
      if (e.key === 'Escape') {
        setPendingLineStart(null);
        setPendingPolyPoints([]);
      }
      if ((e.key === 'Delete' || e.key === 'Backspace') && selectedEntityIdx !== null && currentSketch) {
        if ((e.target as HTMLElement).tagName === 'INPUT' || (e.target as HTMLElement).tagName === 'TEXTAREA') return;
        handleDeleteEntity();
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [activeTool, pendingPolyPoints, selectedEntityIdx, currentSketch, handleUndo, handleRedo]);

  // Auto-load a sketch when features exist but no sketch is selected
  useEffect(() => {
    if (features.length > 0 && !currentSketch) {
      const lastFeature = features[features.length - 1];
      if (lastFeature.sketch_id) {
        loadSketch(lastFeature.sketch_id);
      }
    }
  }, [features, currentSketch, loadSketch]);

  // Load all sketches for multi-sketch feature creation
  useEffect(() => {
    if (!creatingFeature || creatingFeature.sketchIds.length <= 1) {
      setLoadedSketches({});
      return;
    }
    let cancelled = false;
    const loadAll = async () => {
      const map: Record<string, Sketch> = {};
      for (const id of creatingFeature.sketchIds) {
        try { map[id] = await api.getSketch(id); } catch { /* ignore */ }
      }
      if (!cancelled) setLoadedSketches(map);
    };
    loadAll();
    return () => { cancelled = true; };
  }, [creatingFeature]);

  // Sync currentSketch into loadedSketches when it changes in multi-sketch mode
  useEffect(() => {
    if (!creatingFeature || creatingFeature.sketchIds.length <= 1 || !currentSketch) return;
    setLoadedSketches(prev => ({ ...prev, [currentSketch.id]: currentSketch }));
  }, [currentSketch?.entities?.length]);

  const displaySketch = useMemo(() => {
    if (!creatingFeature || creatingFeature.sketchIds.length <= 1) return currentSketch;
    const allEntities = creatingFeature.sketchIds.flatMap(id => loadedSketches[id]?.entities ?? []);
    if (allEntities.length === 0) return currentSketch;
    return { ...currentSketch!, entities: allEntities };
  }, [creatingFeature, loadedSketches, currentSketch]);

  const handleCanvasClick = async (x: number, y: number) => {
    if (!currentSketch) return;
    if (activeTool === 'select') return;

    if (activeTool === 'trim') {
      handleTrim(x, y);
      return;
    }
    if (activeTool === 'extend') {
      handleExtend(x, y);
      return;
    }

    if (activeTool === 'line') {
      if (!pendingLineStart) {
        setPendingLineStart({ x, y });
        return;
      }
      try {
        await api.addSketchEntity(currentSketch.id, {
          entity_type: 'line',
          points: [{ x: pendingLineStart.x, y: pendingLineStart.y, z: 0 }, { x, y, z: 0 }],
        construction: constructionMode,
        });
        await loadSketch(currentSketch.id);
        setPendingLineStart({ x, y });
      } catch (e) {
        alert('添加线段失败: ' + (e instanceof Error ? e.message : String(e)));
        setPendingLineStart(null);
      }
      return;
    }

    if (activeTool === 'circle') {
      if (!pendingLineStart) {
        setPendingLineStart({ x, y });
        return;
      }
      const radius = Math.hypot(x - pendingLineStart.x, y - pendingLineStart.y);
      try {
        await api.addSketchEntity(currentSketch.id, {
          entity_type: 'circle',
          points: [{ x: pendingLineStart.x, y: pendingLineStart.y, z: 0 }],
          radius,
          construction: constructionMode,
        });
        await loadSketch(currentSketch.id);
        setPendingLineStart(null);
      } catch (e) {
        alert('添加圆形失败: ' + (e instanceof Error ? e.message : String(e)));
        setPendingLineStart(null);
      }
      return;
    }

    if (activeTool === 'arc') {
      if (!pendingLineStart) {
        setPendingLineStart({ x, y });
        return;
      }
      if (pendingPolyPoints.length === 0) {
        setPendingPolyPoints([{ x, y }]);
        return;
      }
      try {
        await api.addSketchEntity(currentSketch.id, {
          entity_type: 'arc',
          points: [
            { x: pendingLineStart.x, y: pendingLineStart.y, z: 0 },
            { x: pendingPolyPoints[0].x, y: pendingPolyPoints[0].y, z: 0 },
            { x, y, z: 0 },
          ],
          radius: Math.hypot(pendingPolyPoints[0].x - pendingLineStart.x, pendingPolyPoints[0].y - pendingLineStart.y),
          construction: constructionMode,
        });
        await loadSketch(currentSketch.id);
        setPendingLineStart(null);
        setPendingPolyPoints([]);
      } catch (e) {
        alert('添加弧线失败: ' + (e instanceof Error ? e.message : String(e)));
        setPendingLineStart(null);
        setPendingPolyPoints([]);
      }
      return;
    }

    if (activeTool === 'rect') {
      if (!pendingLineStart) {
        setPendingLineStart({ x, y });
        return;
      }
      try {
        await api.addSketchEntity(currentSketch.id, {
          entity_type: 'rectangle',
          points: [
            { x: pendingLineStart.x, y: pendingLineStart.y, z: 0 },
            { x, y: pendingLineStart.y, z: 0 },
            { x, y, z: 0 },
            { x: pendingLineStart.x, y, z: 0 },
          ],
          construction: constructionMode,
        });
        await loadSketch(currentSketch.id);
        setPendingLineStart(null);
      } catch (e) {
        alert('添加矩形失败: ' + (e instanceof Error ? e.message : String(e)));
        setPendingLineStart(null);
      }
      return;
    }

    if (activeTool === 'polygon') {
      if (!pendingLineStart) {
        setPendingLineStart({ x, y });
        return;
      }
      const r = Math.hypot(x - pendingLineStart.x, y - pendingLineStart.y);
      const pts = [];
      for (let i = 0; i < pendingPolygonSides; i++) {
        const angle = (Math.PI * 2 * i) / pendingPolygonSides - Math.PI / 2;
        pts.push({ x: pendingLineStart.x + r * Math.cos(angle), y: pendingLineStart.y + r * Math.sin(angle), z: 0 });
      }
      try {
        await api.addSketchEntity(currentSketch.id, {
          entity_type: 'polygon',
          points: pts,
          sides: pendingPolygonSides,
          construction: constructionMode,
        });
        await loadSketch(currentSketch.id);
        setPendingLineStart(null);
      } catch (e) {
        alert('添加正多边形失败: ' + (e instanceof Error ? e.message : String(e)));
        setPendingLineStart(null);
      }
      return;
    }

    if (activeTool === 'ellipse') {
      if (!pendingLineStart) {
        setPendingLineStart({ x, y });
        return;
      }
      if (pendingPolyPoints.length === 0) {
        setPendingPolyPoints([{ x, y }]);
        return;
      }
      const cx = pendingLineStart.x;
      const cy = pendingLineStart.y;
      const majorX = pendingPolyPoints[0].x - cx;
      const majorY = pendingPolyPoints[0].y - cy;
      const minorX = x - cx;
      const minorY = y - cy;
      const majorR = Math.hypot(majorX, majorY);
      const minorR = Math.hypot(minorX, minorY);
      try {
        await api.addSketchEntity(currentSketch.id, {
          entity_type: 'ellipse',
          points: [
            { x: cx, y: cy, z: 0 },
            { x: pendingPolyPoints[0].x, y: pendingPolyPoints[0].y, z: 0 },
            { x, y, z: 0 },
          ],
          radius: Math.max(majorR, minorR),
          construction: constructionMode,
        });
        await loadSketch(currentSketch.id);
        setPendingLineStart(null);
        setPendingPolyPoints([]);
      } catch (e) {
        alert('添加椭圆失败: ' + (e instanceof Error ? e.message : String(e)));
        setPendingLineStart(null);
        setPendingPolyPoints([]);
      }
      return;
    }

    if (activeTool === 'spline' || activeTool === 'polyline') {
      const newPoints = [...pendingPolyPoints, { x, y }];
      setPendingPolyPoints(newPoints);
      return;
    }

    const entityType = activeTool === 'circle' ? 'circle' : 'arc';
    try {
      const res = await api.addSketchEntity(currentSketch.id, {
        entity_type: entityType,
        points: [{ x, y, z: 0 }],
        radius: 1.0,
          construction: constructionMode,
      });
      await loadSketch(currentSketch.id);
      setSelectedEntityIdx((res as any).index);
    } catch (e) {
      alert('添加实体失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleFinishPoly = async () => {
    if (!currentSketch || pendingPolyPoints.length < 2) return;
    const type = activeTool === 'spline' ? 'spline' : 'polyline';
    try {
      await api.addSketchEntity(currentSketch.id, {
        entity_type: type,
        points: pendingPolyPoints.map(p => ({ x: p.x, y: p.y, z: 0 })),
          construction: constructionMode,
      });
      await loadSketch(currentSketch.id);
      setPendingPolyPoints([]);
      setPendingLineStart(null);
    } catch (e) {
      alert('添加' + type + '失败: ' + (e instanceof Error ? e.message : String(e)));
      setPendingPolyPoints([]);
      setPendingLineStart(null);
    }
  };

  const handleSolve = async () => {
    if (!currentSketch) return;
    setSolving(true);
    try {
      await solveAndLoad(currentSketch.id);
    } catch (e) {
      alert('求解失败: ' + (e instanceof Error ? e.message : String(e)));
    }
    setSolving(false);
  };

  const handleAddConstraint = async (ct: SketchConstraintType, entityIndices: number[], value?: number) => {
    if (!currentSketch) return;
    try {
      const lower = ct.replace(/([A-Z])/g, '_$1').toLowerCase().replace(/^_/, '');
      const entityRefs: [number, number][] = entityIndices.map((ei) => [ei, 0]);
      await api.addSketchConstraint(currentSketch.id, {
        constraint_type: lower,
        entity_refs: entityRefs,
        value,
      });
      await loadSketch(currentSketch.id);
    } catch (e) {
      alert('添加约束失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleDeleteEntity = async () => {
    if (!currentSketch || selectedEntityIdx === null) return;
    await api.deleteSketchEntity(currentSketch.id, selectedEntityIdx);
    setSelectedEntityIdx(null);
    loadSketch(currentSketch.id);
  };

  const handleTrim = async (clickX: number, clickY: number) => {
    if (!currentSketch) return;
    const entities = currentSketch.entities;
    const { pointToSegmentDist, pointToSegmentParam, findEntityIntersections } = await import('../shared/geometry');
    let bestIdx = -1;
    let bestDist = 20 / 80;
    for (let i = 0; i < entities.length; i++) {
      const e = entities[i];
      if (e.entity_type === 'Line' && e.points.length >= 2) {
        const d = pointToSegmentDist(clickX, clickY, e.points[0].x, e.points[0].y, e.points[1].x, e.points[1].y);
        if (d < bestDist) { bestDist = d; bestIdx = i; }
      }
    }
    if (bestIdx < 0) return;

    const ints = findEntityIntersections(bestIdx, entities);
    if (ints.length === 0) return;

    const entity = entities[bestIdx];
    const a = entity.points[0], b = entity.points[1];
    const clickT = pointToSegmentParam(clickX, clickY, a.x, a.y, b.x, b.y);

    let segIdx = 0;
    for (let i = 0; i < ints.length; i++) {
      if (clickT < ints[i].t) { segIdx = i; break; }
    }
    if (segIdx >= ints.length) segIdx = ints.length - 1;

    await api.deleteSketchEntity(currentSketch.id, bestIdx);
    await loadSketch(currentSketch.id);

    const segments: { a: { x: number; y: number; z: number }; b: { x: number; y: number; z: number } }[] = [];
    let prev = a;
    for (let i = 0; i < ints.length; i++) {
      if (i !== segIdx) {
        segments.push({ a: { x: prev.x, y: prev.y, z: 0 }, b: { x: ints[i].point.x, y: ints[i].point.y, z: 0 } });
      }
      prev = { x: ints[i].point.x, y: ints[i].point.y, z: 0 };
    }
    if (segIdx !== ints.length - 1) {
      segments.push({ a: { x: prev.x, y: prev.y, z: 0 }, b: { x: b.x, y: b.y, z: 0 } });
    }

    for (const seg of segments) {
      if (Math.hypot(seg.b.x - seg.a.x, seg.b.y - seg.a.y) > 1e-4) {
        await api.addSketchEntity(currentSketch.id, { entity_type: 'line', points: [seg.a, seg.b], construction: !!entity.construction });
      }
    }
    await loadSketch(currentSketch.id);
  };

  const handleExtend = async (clickX: number, clickY: number) => {
    if (!currentSketch) return;
    const entities = currentSketch.entities;
    const { extendLineToIntersection, findNearestLineEndpoint } = await import('../shared/geometry');
    let bestIdx = -1;
    let bestDist = 15 / 80;
    for (let i = 0; i < entities.length; i++) {
      const e = entities[i];
      if (e.entity_type === 'Line' && e.points.length >= 2) {
        const d = Math.min(
          Math.hypot(clickX - e.points[0].x, clickY - e.points[0].y),
          Math.hypot(clickX - e.points[1].x, clickY - e.points[1].y),
        );
        if (d < bestDist) { bestDist = d; bestIdx = i; }
      }
    }
    if (bestIdx < 0) return;
    const entity = entities[bestIdx];
    const nearest = findNearestLineEndpoint({ x: clickX, y: clickY }, bestIdx, entities, 15 / 80);
    if (!nearest) return;

    const newEnd = extendLineToIntersection(nearest.endpoint, nearest.otherEndpoint, bestIdx, entities);
    if (!newEnd) return;

    const newPoints = nearest.isStart
      ? [{ x: newEnd.x, y: newEnd.y, z: 0 }, { x: nearest.otherEndpoint.x, y: nearest.otherEndpoint.y, z: 0 }]
      : [{ x: nearest.otherEndpoint.x, y: nearest.otherEndpoint.y, z: 0 }, { x: newEnd.x, y: newEnd.y, z: 0 }];

    try {
      await api.updateSketchEntity(currentSketch.id, bestIdx, { points: newPoints });
      await loadSketch(currentSketch.id);
    } catch (e) {
      alert('延伸失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleMirror = async () => {
    if (!currentSketch || selectedEntityIdx === null) return;
    const entities = currentSketch.entities;
    const srcEntity = entities[selectedEntityIdx];
    if (!srcEntity || srcEntity.entity_type !== 'Line' || srcEntity.points.length < 2) return;

    let axisIdx = -1;
    for (let i = 0; i < entities.length; i++) {
      if (i !== selectedEntityIdx && entities[i].entity_type === 'Line' && entities[i].points.length >= 2) {
        axisIdx = i; break;
      }
    }
    if (axisIdx < 0) { alert('请先绘制一条镜像轴（参考线）'); return; }

    const { mirrorPoint } = await import('../shared/geometry');
    const axis = entities[axisIdx];
    const axisA = { x: axis.points[0].x, y: axis.points[0].y };
    const axisB = { x: axis.points[1].x, y: axis.points[1].y };

    const mirroredPoints = srcEntity.points.map(p => {
      const m = mirrorPoint({ x: p.x, y: p.y }, axisA, axisB);
      return { x: m.x, y: m.y, z: p.z };
    });

    await api.addSketchEntity(currentSketch.id, {
      entity_type: srcEntity.entity_type.toLowerCase(),
      points: mirroredPoints,
      radius: srcEntity.radius,
      sides: srcEntity.sides,
      construction: !!srcEntity.construction,
    });
    await loadSketch(currentSketch.id);
  };

  const handleOffset = async () => {
    if (!currentSketch || selectedEntityIdx === null) return;
    const entity = currentSketch.entities[selectedEntityIdx];
    if (!entity || entity.entity_type !== 'Line' || entity.points.length < 2) return;

    const { offsetLine } = await import('../shared/geometry');
    const a = { x: entity.points[0].x, y: entity.points[0].y };
    const b = { x: entity.points[1].x, y: entity.points[1].y };
    const offset = offsetLine(a, b, offsetDistance);

    await api.addSketchEntity(currentSketch.id, {
      entity_type: 'line',
      points: [
        { x: offset.a.x, y: offset.a.y, z: 0 },
        { x: offset.b.x, y: offset.b.y, z: 0 },
      ],
      construction: !!entity.construction,
    });
    await loadSketch(currentSketch.id);
  };

  // ── Feature workflow ──

  const handleEditFeature = (feature: FeatureItem) => {
    setEditingFeature(feature);
  };

  const handleAddFeature = async (type: FeatureType) => {
    if (!currentComponentId) return;
    try {
      setFeatureDepth(1.0);
      setFeatureAngle(360);
      const sketchCount = type === 'fusion' ? 2
        : type === 'lofted_fusion' ? 3
        : type === 'revolve' || type === 'sweep' || type === 'loft' ? 2
        : 1;

      const sketchIds: string[] = [];
      for (let i = 0; i < sketchCount; i++) {
        const label = type === 'revolve'
          ? (i === 0 ? 'profile' : 'axis')
          : type === 'sweep'
          ? (i === 0 ? 'profile' : 'path')
          : type === 'fusion'
          ? (i === 0 ? 'profile1' : 'profile2')
          : type === 'lofted_fusion'
          ? (i === 0 ? 'profile1' : i === 1 ? 'profile2' : 'path')
          : (i === 0 ? 'profile' : 'section');
        const sketch = await api.createSketch({
          name: `${type}-${label}-${Date.now()}`,
          component_id: currentComponentId,
        });
        sketchIds.push(sketch.id);
      }
      await loadSketches(currentComponentId);
      await loadSketch(sketchIds[0]);
      setTab('component');
      setActiveTool('line');
      setPendingLineStart(null);
      setCreatingFeature({ featureType: type, sketchIds, currentSketchIdx: 0 });
    } catch (e) {
      alert('创建 Sketch 失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleSwitchSketch = (idx: number) => {
    if (!creatingFeature) return;
    loadSketch(creatingFeature.sketchIds[idx]);
    setCreatingFeature({ ...creatingFeature, currentSketchIdx: idx });
  };

  const handleFinishDrawing = () => {
    if (!creatingFeature) return;
    const isMulti = creatingFeature.sketchIds.length > 1;
    const angleRad = creatingFeature.featureType === 'revolve' ? (featureAngle * Math.PI) / 180 : 0;
    setEditingFeature({
      id: '',
      name: `${creatingFeature.featureType}-new`,
      feature_type: creatingFeature.featureType,
      sketch_id: creatingFeature.sketchIds[0],
      input_sketch_ids: isMulti ? creatingFeature.sketchIds.slice(1) : [],
      axis_sketch_id: creatingFeature.featureType === 'revolve' && creatingFeature.sketchIds.length > 1 ? creatingFeature.sketchIds[1] : '',
      depth: featureDepth,
      angle: angleRad,
      is_cut: false,
      order: features.length,
      suppressed: false,
      dirty: false,
      dependency_ids: [],
    });
  };

  const handleCancelCreating = () => {
    setCreatingFeature(null);
    setActiveTool('select');
    setPendingLineStart(null);
    setPendingPolyPoints([]);
  };

  const handleAddFeatureConfirm = async (params: Record<string, unknown>) => {
    if (!currentComponentId || !editingFeature) return;
    try {
      if (editingFeature.id) {
        await api.updateFeature(currentComponentId, editingFeature.id, {
          sketch_id: params.sketch_id as string,
          axis_sketch_id: params.axis_sketch_id as string | undefined,
          depth: params.depth as number,
          angle: params.angle as number,
          is_cut: params.is_cut as boolean,
        });
      } else {
        await api.addFeature(currentComponentId, params as any);
      }
      setCreatingFeature(null);
      setEditingFeature(null);
      await loadFeatures(currentComponentId);
    } catch (e) {
      alert((editingFeature.id ? '编辑' : '添加') + ' Feature 失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  const handleSelectSketch = (sketchId: string) => {
    loadSketch(sketchId);
    setTab('component');
  };

  // ── 3D preview ──

  async function rebuildPreview() {
    if (features.length === 0) { setPreviewMesh(null); return; }

    // Load all sketches and compute meshes for each feature
    const allPositions: number[] = [];
    const allNormals: number[] = [];
    const allIndices: number[] = [];
    let vertexOffset = 0;

    for (const feature of features) {
      if (feature.suppressed) continue;
      if (!['extrude', 'revolve', 'sweep', 'loft', 'fusion', 'lofted_fusion', 'boolean'].includes(feature.feature_type)) continue;

      try {
        let mesh: { positions: number[]; normals: number[]; indices: number[] } | null = null;
        const sketch = await api.getSketch(feature.sketch_id);
        if (!sketch) continue;

        switch (feature.feature_type) {
          case 'extrude':
            mesh = computeExtrudeMesh(sketch, feature.depth, 0, 0, 1, sketch.origin?.z ?? 0);
            break;
          case 'revolve':
            mesh = computeRevolveMesh(sketch, feature.angle, 0, 0, 1, sketch.origin?.z ?? 0);
            break;
          case 'sweep': {
            const pathSketchId = feature.input_sketch_ids?.[0];
            if (pathSketchId) {
              const pathSketch = await api.getSketch(pathSketchId);
              if (pathSketch) mesh = computeSweepMesh(sketch, pathSketch);
            }
            break;
          }
          case 'loft': {
            const sectionIds = feature.input_sketch_ids || [];
            if (sectionIds.length >= 2) {
              const s0 = await api.getSketch(sectionIds[0]);
              const s1 = await api.getSketch(sectionIds[1]);
              if (s0 && s1) mesh = computeLoftMesh(s0, s1);
            }
            break;
          }
          case 'fusion':
          case 'lofted_fusion':
            mesh = computeExtrudeMesh(sketch, feature.depth || 1.0, 0, 0, 1, sketch.origin?.z ?? 0);
            break;
          case 'boolean':
            mesh = computeExtrudeMesh(sketch, feature.depth || 1.0, 0, 0, 1, sketch.origin?.z ?? 0);
            break;
        }
        if (!mesh) continue;

        // Merge: offset indices by current vertex count
        allPositions.push(...mesh.positions);
        allNormals.push(...mesh.normals);
        for (const idx of mesh.indices) {
          allIndices.push(idx + vertexOffset);
        }
        vertexOffset += mesh.positions.length / 3;
      } catch { /* skip failed sketches */ }
    }

    if (allPositions.length > 0) {
      setPreviewMesh({ positions: allPositions, normals: allNormals, indices: allIndices });
    } else {
      setPreviewMesh(null);
    }
  }

  function extractProfile(sketch: Sketch): { x: number; y: number }[] {
    const pts: { x: number; y: number }[] = [];
    for (const e of sketch.entities) {
      if ((e.entity_type === 'Line' || e.entity_type === 'Polyline' || e.entity_type === 'Spline') && e.points.length >= 2) {
        for (const p of e.points) pts.push({ x: p.x, y: p.y });
      } else if (e.entity_type === 'Rectangle' || e.entity_type === 'Polygon') {
        for (const p of e.points) pts.push({ x: p.x, y: p.y });
      } else if (e.entity_type === 'Ellipse' && e.points.length >= 3) {
        const cx = e.points[0].x, cy = e.points[0].y;
        const rx = Math.hypot(e.points[1].x - cx, e.points[1].y - cy);
        const ry = Math.hypot(e.points[2].x - cx, e.points[2].y - cy);
        const segs = 32;
        for (let i = 0; i < segs; i++) {
          const a = (Math.PI * 2 * i) / segs;
          pts.push({ x: cx + rx * Math.cos(a), y: cy + ry * Math.sin(a) });
        }
      } else if (e.entity_type === 'Circle' && e.points.length >= 1) {
        const cx = e.points[0].x, cy = e.points[0].y, r = e.radius;
        const segs = 32;
        for (let i = 0; i < segs; i++) {
          const a = (Math.PI * 2 * i) / segs;
          pts.push({ x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) });
        }
      }
    }
    const out: typeof pts = [];
    for (const p of pts) {
      if (out.length === 0 || Math.hypot(p.x - out[out.length - 1].x, p.y - out[out.length - 1].y) > 1e-6) {
        out.push(p);
      }
    }
    return out;
  }

  function computeExtrudeMesh(sketch: Sketch, depth: number, _dx: number, _dy: number, _dz: number, zOffset = 0) {
    const profile = extractProfile(sketch);
    if (profile.length < 3) return null;

    const shape = new THREE.Shape(profile.map((p) => new THREE.Vector2(p.x, p.y)));
    const extrudeSettings = { depth, bevelEnabled: false };
    const geo = new THREE.ExtrudeGeometry(shape, extrudeSettings);
    geo.computeVertexNormals();

    const pos = geo.getAttribute('position');
    const norm = geo.getAttribute('normal');
    const idx = geo.getIndex();

    if (!pos || !norm) return null;

    const positions: number[] = [];
    const normals: number[] = [];
    for (let i = 0; i < pos.count; i++) {
      positions.push(pos.getX(i), pos.getY(i), pos.getZ(i) + zOffset);
      normals.push(norm.getX(i), norm.getY(i), norm.getZ(i));
    }

    const indices: number[] = [];
    if (idx) {
      for (let i = 0; i < idx.count; i++) indices.push(idx.getX(i));
    } else {
      for (let i = 0; i < pos.count; i++) indices.push(i);
    }

    geo.dispose();
    return { positions, normals, indices };
  }

  function computeRevolveMesh(sketch: Sketch, angle: number, _dx: number, _dy: number, _dz: number, zOffset = 0) {
    const profile = extractProfile(sketch);
    if (profile.length < 2) return null;

    const segments = 32;
    const totalAngle = Math.abs(angle) < 0.001 ? Math.PI * 2 : Math.abs(angle);

    const points = profile.map((p) => new THREE.Vector2(p.x, p.y));
    const geo = new THREE.LatheGeometry(points, segments, 0, totalAngle);
    geo.computeVertexNormals();

    const pos = geo.getAttribute('position');
    const norm = geo.getAttribute('normal');
    const idx = geo.getIndex();
    if (!pos || !norm) return null;

    const positions: number[] = [];
    const normals: number[] = [];
    for (let i = 0; i < pos.count; i++) {
      positions.push(pos.getX(i), pos.getY(i), pos.getZ(i) + zOffset);
      normals.push(norm.getX(i), norm.getY(i), norm.getZ(i));
    }

    const indices: number[] = [];
    if (idx) {
      for (let i = 0; i < idx.count; i++) indices.push(idx.getX(i));
    } else {
      for (let i = 0; i < pos.count; i++) indices.push(i);
    }

    geo.dispose();
    return { positions, normals, indices };
  }

  function computeSweepMesh(profileSketch: Sketch, pathSketch: Sketch) {
    const profile = extractProfile(profileSketch);
    if (profile.length < 3) return null;

    // Extract path points from path sketch entities
    const pathPoints: THREE.Vector3[] = [];
    for (const e of pathSketch.entities) {
      for (const p of e.points) {
        if (pathPoints.length === 0 || Math.hypot(p.x - pathPoints[pathPoints.length - 1].x, p.y - pathPoints[pathPoints.length - 1].y, (p.z ?? 0) - pathPoints[pathPoints.length - 1].z) > 1e-6) {
          pathPoints.push(new THREE.Vector3(p.x, p.y, p.z ?? 0));
        }
      }
    }
    if (pathPoints.length < 2) return null;

    // Create a THREE.CatmullRomCurve3 through the path points
    const curve = new THREE.CatmullRomCurve3(pathPoints, false, 'catmullrom', 0.5);

    // Extrude the profile along the curve path
    const shape = new THREE.Shape(profile.map((p) => new THREE.Vector2(p.x, p.y)));
    const extrudeSettings = {
      extrudePath: curve,
      steps: Math.max(pathPoints.length * 8, 32),
      bevelEnabled: false,
    };
    const geo = new THREE.ExtrudeGeometry(shape, extrudeSettings);
    geo.computeVertexNormals();

    const pos = geo.getAttribute('position');
    const norm = geo.getAttribute('normal');
    const idx = geo.getIndex();
    if (!pos || !norm) return null;

    const positions: number[] = [];
    const normals: number[] = [];
    for (let i = 0; i < pos.count; i++) {
      positions.push(pos.getX(i), pos.getY(i), pos.getZ(i));
      normals.push(norm.getX(i), norm.getY(i), norm.getZ(i));
    }

    const indices: number[] = [];
    if (idx) {
      for (let i = 0; i < idx.count; i++) indices.push(idx.getX(i));
    } else {
      for (let i = 0; i < pos.count; i++) indices.push(i);
    }

    geo.dispose();
    return { positions, normals, indices };
  }

  function computeLoftMesh(sketch0: Sketch, sketch1: Sketch) {
    const prof0 = extractProfile(sketch0);
    const prof1 = extractProfile(sketch1);
    if (prof0.length < 3 || prof1.length < 3) return null;

    // Resample both profiles to the same number of points
    const n = Math.max(prof0.length, prof1.length);
    const resample = (pts: { x: number; y: number }[], count: number) => {
      if (pts.length === count) return pts;
      const result: { x: number; y: number }[] = [];
      for (let i = 0; i < count; i++) {
        const t = i / count;
        const idx = t * pts.length;
        const i0 = Math.floor(idx) % pts.length;
        const i1 = (i0 + 1) % pts.length;
        const frac = idx - Math.floor(idx);
        result.push({
          x: pts[i0].x + (pts[i1].x - pts[i0].x) * frac,
          y: pts[i0].y + (pts[i1].y - pts[i0].y) * frac,
        });
      }
      return result;
    };

    const p0 = resample(prof0, n);
    const p1 = resample(prof1, n);

    // z offset from sketch origins
    const z0 = sketch0.origin?.z ?? 0;
    const z1 = sketch1.origin?.z ?? 0;

    const positions: number[] = [];
    const normals: number[] = [];
    const indices: number[] = [];

    // Bottom ring (z0) and top ring (z1)
    for (const p of p0) { positions.push(p.x, p.y, z0); }
    for (const p of p1) { positions.push(p.x, p.y, z1); }

    // Bottom cap (fan triangulation)
    const bottomCenter = positions.length / 3;
    positions.push(0, 0, z0);
    normals.push(0, 0, -1);
    for (let i = 0; i < n; i++) {
      indices.push(bottomCenter, i, ((i + 1) % n));
    }

    // Top cap
    const topCenter = positions.length / 3;
    const topBase = n;
    positions.push(0, 0, z1);
    normals.push(0, 0, 1);
    for (let i = 0; i < n; i++) {
      indices.push(topCenter, topBase + ((i + 1) % n), topBase + i);
    }

    // Side faces (quads → 2 triangles each)
    for (let i = 0; i < n; i++) {
      const bl = i;
      const br = (i + 1) % n;
      const tl = topBase + i;
      const tr = topBase + ((i + 1) % n);

      // Compute face normal from cross product of edge vectors
      const ax = positions[bl * 3], ay = positions[bl * 3 + 1], az = positions[bl * 3 + 2];
      const bx = positions[br * 3], by = positions[br * 3 + 1], bz = positions[br * 3 + 2];
      const dx = positions[tl * 3], dy = positions[tl * 3 + 1], dz = positions[tl * 3 + 2];
      const e1x = bx - ax, e1y = by - ay, e1z = bz - az;
      const e2x = dx - ax, e2y = dy - ay, e2z = dz - az;
      let nx = e1y * e2z - e1z * e2y;
      let ny = e1z * e2x - e1x * e2z;
      let nz = e1x * e2y - e1y * e2x;
      const len = Math.hypot(nx, ny, nz) || 1;
      nx /= len; ny /= len; nz /= len;

      // Assign face normal to all 4 vertices
      normals[bl * 3] = nx; normals[bl * 3 + 1] = ny; normals[bl * 3 + 2] = nz;
      normals[br * 3] = nx; normals[br * 3 + 1] = ny; normals[br * 3 + 2] = nz;
      normals[tl * 3] = nx; normals[tl * 3 + 1] = ny; normals[tl * 3 + 2] = nz;
      normals[tr * 3] = nx; normals[tr * 3 + 1] = ny; normals[tr * 3 + 2] = nz;

      indices.push(bl, br, tl);
      indices.push(br, tr, tl);
    }

    return { positions, normals, indices };
  }

  const handleBack = useCallback(async () => {
    if (currentComponentId) {
      try {
        await api.rebuildComponent(currentComponentId);
      } catch (e) {
        console.warn('Rebuild on exit failed:', e);
      }
    }
    navigate('/');
  }, [currentComponentId, navigate]);

  if (!currentComponentId) {
    return (
      <div className="component-editor-shell">
        <div className="component-topbar">
          <button onClick={handleBack} className="component-back-btn">&larr; Back</button>
          <span className="component-title">Component Editor</span>
        </div>
        <div className="component-canvas-area">
          <div className="component-empty">
            请从主视图创建或编辑 Component
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="component-editor-shell">
      {/* Top bar */}
      <div className="component-topbar">
        <button onClick={handleBack} className="component-back-btn">&larr; Back</button>
        {editingName ? (
          <input
            className="component-title-input"
            value={componentName}
            autoFocus
            onChange={(e) => setComponentName(e.target.value)}
            onBlur={() => {
              setEditingName(false);
              if (currentComponentId && componentName.trim()) {
                api.updateObject(currentComponentId, { name: componentName.trim() }).catch(() => {});
              }
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') (e.target as HTMLInputElement).blur();
              if (e.key === 'Escape') {
                setEditingName(false);
                api.getComponent(currentComponentId).then(c => setComponentName(c.name)).catch(() => {});
              }
            }}
          />
        ) : (
          <span
            className="component-title component-title-editable"
            onClick={() => setEditingName(true)}
            title="点击编辑名称"
          >{componentName || 'Component Editor'}</span>
        )}
      </div>

      {/* Main content: Feature Panel + Workspace */}
      <div className="component-main-content">
        {/* Left: Feature Panel */}
        <FeaturePanel
          componentId={currentComponentId}
          features={features}
          sketches={sketches}
          onRefresh={() => loadFeatures(currentComponentId)}
          onEditFeature={handleEditFeature}
          onSelectSketch={handleSelectSketch}
          onRebuilt={rebuildPreview}
        />

        {/* Right: Workspace */}
        <div className="component-workspace">
          {/* Toolbar */}
          <EditorToolbar
            activeTool={activeTool}
            activeConstraint={activeConstraint}
            solving={solving}
            hasSketch={!!currentSketch}
            hasSelection={selectedEntityIdx !== null}
            creatingFeature={creatingFeature?.featureType ?? null}
            creatingFeatureSketches={creatingFeature ? MULTI_SKETCH_LABELS[creatingFeature.featureType] ?? null : null}
            activeSketchIdx={creatingFeature?.currentSketchIdx ?? 0}
            snapEnabled={snapEnabled}
            constructionMode={constructionMode}
            offsetDistance={offsetDistance}
            canUndo={canUndo}
            canRedo={canRedo}
            onSwitchSketch={handleSwitchSketch}
            onSelectTool={(t) => { setActiveTool(t as Tool); setPendingLineStart(null); setPendingPolyPoints([]); }}
            onSelectConstraint={(ct) => {
              setActiveConstraint(ct);
              if (ct && currentSketch && selectedEntityIdx !== null) {
                handleAddConstraint(ct, [selectedEntityIdx]);
                setActiveConstraint(null);
              }
            }}
            onSolve={handleSolve}
            onDeleteEntity={handleDeleteEntity}
            onAddFeature={handleAddFeature}
            onFinishDrawing={handleFinishDrawing}
            onCancelCreating={handleCancelCreating}
            onToggleSnap={() => setSnapEnabled(s => !s)}
            onToggleConstruction={() => setConstructionMode(s => !s)}
            onMirror={handleMirror}
            onOffset={handleOffset}
            onUpdateOffsetDistance={setOffsetDistance}
            onUndo={handleUndo}
            onRedo={handleRedo}
            currentSketchZ={currentSketch?.origin?.z ?? 0}
            featureDepth={featureDepth}
            featureAngle={featureAngle}
            onUpdateSketchZ={async (z) => {
              if (!currentSketch) return;
              const updated = await api.updateSketch(currentSketch.id, { origin: { x: currentSketch.origin.x, y: currentSketch.origin.y, z } });
              setCurrentSketch(updated);
            }}
            onUpdateFeatureDepth={setFeatureDepth}
            onUpdateFeatureAngle={setFeatureAngle}
          />

          {/* Tab bar */}
          <div className="component-tabbar">
            <button
              className={`component-tab ${tab === 'component' ? 'active' : ''}`}
              onClick={() => { setTab('component'); setPendingLineStart(null); }}
            >2D-Top</button>
            <button
              className={`component-tab ${tab === '3d' ? 'active' : ''}`}
              onClick={() => { setTab('3d'); if (!previewMesh && features.length > 0) rebuildPreview(); }}
            >3D</button>
          </div>

          {/* Drawing mode banner */}
          {creatingFeature && (
            <div className="component-creating-banner">
              ⚠ 正在绘制 {creatingFeature.featureType} {MULTI_SKETCH_LABELS[creatingFeature.featureType]?.[creatingFeature.currentSketchIdx] ?? '轮廓'} — 完成后点击 Done
            </div>
          )}

          {/* Main canvas area */}
          <div className="component-canvas-area">
            {tab === 'component' ? (
              <SketchCanvas2D
                sketch={displaySketch}
                sectionLines={sectionLines}
                activeTool={activeTool}
                selectedEntityIdx={selectedEntityIdx}
                pendingLineStart={pendingLineStart}
                pendingPolyPoints={pendingPolyPoints}
                mousePos={mousePos}
                snapEnabled={snapEnabled}
                onCanvasClick={handleCanvasClick}
                onSelectEntity={setSelectedEntityIdx}
                onMouseMoveWorld={(x, y) => setMousePos({ x, y })}
                onEditConstraint={handleEditConstraint}
                onDragEntityPoint={async (entityIdx, pointIdx, x, y) => {
                  if (!currentSketch) return;
                  const entity = currentSketch.entities[entityIdx];
                  const entityType = entity.entity_type;

                  if (entityType === 'Circle') {
                    if (pointIdx === 0) {
                      const newPoints = entity.points.map((p, i) => i === 0 ? { x, y, z: p.z ?? 0 } : { x: p.x, y: p.y, z: p.z ?? 0 });
                      await api.updateSketchEntity(currentSketch.id, entityIdx, { points: newPoints });
                    } else if (pointIdx === 1) {
                      const cx = entity.points[0]?.x ?? 0;
                      const cy = entity.points[0]?.y ?? 0;
                      const newRadius = Math.max(0.01, Math.hypot(x - cx, y - cy));
                      await api.updateSketchEntity(currentSketch.id, entityIdx, { radius: newRadius });
                    }
                  } else if (entityType === 'Arc') {
                    if (pointIdx === 0) {
                      const newPoints = entity.points.map((p, i) => i === 0 ? { x, y, z: p.z ?? 0 } : { x: p.x, y: p.y, z: p.z ?? 0 });
                      await api.updateSketchEntity(currentSketch.id, entityIdx, { points: newPoints });
                    } else if (pointIdx < 3 && entity.points.length >= 3) {
                      const newPoints = entity.points.map((p, i) => i === pointIdx ? { x, y, z: p.z ?? 0 } : { x: p.x, y: p.y, z: p.z ?? 0 });
                      await api.updateSketchEntity(currentSketch.id, entityIdx, { points: newPoints });
                    } else if (pointIdx === 3) {
                      const cx = entity.points[0]?.x ?? 0;
                      const cy = entity.points[0]?.y ?? 0;
                      const newRadius = Math.max(0.01, Math.hypot(x - cx, y - cy));
                      await api.updateSketchEntity(currentSketch.id, entityIdx, { radius: newRadius });
                    }
                  } else {
                    const newPoints = entity.points.map((p, i) => i === pointIdx ? { x, y, z: p.z ?? 0 } : { x: p.x, y: p.y, z: p.z ?? 0 });
                    await api.updateSketchEntity(currentSketch.id, entityIdx, { points: newPoints });
                  }
                  await solveAndLoad(currentSketch.id);
                }}
              />
            ) : (
              <SketchCanvas3D
                sketch={displaySketch}
                selectedEntityIdx={selectedEntityIdx}
                previewMesh={previewMesh}
              />
            )}
          </div>

          {/* Properties panel */}
          {selectedEntityIdx !== null && currentSketch && currentSketch.entities[selectedEntityIdx] && (
            <EntityPropertiesPanel
              entity={currentSketch.entities[selectedEntityIdx]}
              entityIdx={selectedEntityIdx}
              onUpdate={() => loadSketch(currentSketch.id)}
            />
          )}

          {/* Status bar */}
          {currentSketch && (
            <div className="component-statusbar">
              Status: {creatingFeature ? `Drawing ${creatingFeature.featureType}` : (currentSketch.solved ? 'Solved' : 'Unsolved')} &middot;
              Entities: {currentSketch.entities.length} &middot;
              Constraints: {currentSketch.constraints.length} &middot;
              {sketchDof !== null && (
                <span>
                  DOF: {sketchDof} &middot;
                  {sketchDof < 0 ? <span style={{ color: '#ff4444' }}>Over-constrained</span> :
                   sketchDof > 0 ? <span style={{ color: '#ffaa00' }}>Under-constrained</span> :
                   <span style={{ color: '#44ff88' }}>Fully constrained</span>} &middot;
                </span>
              )}
              Mode: {currentSketch.is_3d ? '3D' : '2D'}
            </div>
          )}
        </div>
      </div>

      {/* Dimension value edit overlay */}
      {editingConstraint && (
        <div className="dimension-edit-overlay" onClick={() => setEditingConstraint(null)}>
          <div className="dimension-edit-dialog" onClick={(e) => e.stopPropagation()}>
            <div className="dimension-edit-header">
              编辑 {editingConstraint.type === 'Length' ? '长度' :
                    editingConstraint.type === 'Distance' ? '距离' :
                    editingConstraint.type === 'Angle' ? '角度' :
                    editingConstraint.type === 'Radius' ? '半径' : '直径'}
            </div>
            <input
              className="dimension-edit-input"
              type="number"
              step="0.1"
              defaultValue={editingConstraint.value || 0}
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  handleConstraintValueChange(parseFloat((e.target as HTMLInputElement).value) || 0);
                }
                if (e.key === 'Escape') {
                  setEditingConstraint(null);
                }
              }}
              onBlur={(e) => {
                handleConstraintValueChange(parseFloat(e.target.value) || 0);
              }}
            />
            <div className="dimension-edit-actions">
              <button className="dimension-edit-btn" onClick={() => setEditingConstraint(null)}>取消</button>
              <button
                className="dimension-edit-btn dimension-edit-confirm"
                onClick={() => {
                  const input = document.querySelector('.dimension-edit-input') as HTMLInputElement;
                  if (input) handleConstraintValueChange(parseFloat(input.value) || 0);
                }}
              >确认</button>
            </div>
          </div>
        </div>
      )}

      {/* Add/Edit Feature Dialog */}
      {editingFeature && (
        <AddFeatureDialog
          feature={editingFeature}
          sketches={sketches}
          onConfirm={handleAddFeatureConfirm}
          onCancel={() => { setEditingFeature(null); if (!editingFeature.id) setCreatingFeature(null); }}
        />
      )}
    </div>
  );
}
