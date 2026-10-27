import { useRef, useEffect, useCallback, useState } from 'react';
import type { Sketch } from '../shared/types';

type Props = {
  sketch: Sketch | null;
  sectionLines?: { x1: number; y1: number; x2: number; y2: number }[];
  activeTool: string;
  selectedEntityIdx: number | null;
  pendingLineStart: { x: number; y: number } | null;
  pendingPolyPoints: { x: number; y: number }[];
  mousePos: { x: number; y: number };
  snapEnabled: boolean;
  onCanvasClick: (x: number, y: number) => void;
  onSelectEntity: (idx: number | null) => void;
  onMouseMoveWorld: (x: number, y: number) => void;
  onEditConstraint?: (constraintIdx: number) => void;
  onDragEntityPoint?: (entityIdx: number, pointIdx: number, x: number, y: number) => void;
};

export function SketchCanvas2D({
  sketch, sectionLines = [], activeTool, selectedEntityIdx,
  pendingLineStart, pendingPolyPoints, mousePos, snapEnabled,
  onCanvasClick, onSelectEntity, onMouseMoveWorld, onEditConstraint,
  onDragEntityPoint,
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [scale, setScale] = useState(80);
  const dragRef = useRef<{ start: { x: number; y: number }; offset: { x: number; y: number } } | null>(null);
  const dimensionHitRef = useRef<{ x: number; y: number; constraintIdx: number }[]>([]);
  const gripDragRef = useRef<{
    entityIdx: number;
    pointIdx: number;
    startX: number;
    startY: number;
  } | null>(null);
  const [gripDragPos, setGripDragPos] = useState<{ x: number; y: number } | null>(null);
  const gripHoverRef = useRef<number | null>(null);

  const toCanvas = useCallback((wx: number, wy: number): { x: number; y: number } => {
    const c = canvasRef.current;
    if (!c) return { x: 0, y: 0 };
    return {
      x: c.width / 2 + wx * scale + offset.x,
      y: c.height / 2 - wy * scale + offset.y,
    };
  }, [scale, offset]);

  const toWorld = useCallback((cx: number, cy: number): { x: number; y: number } => {
    const c = canvasRef.current;
    if (!c) return { x: 0, y: 0 };
    return {
      x: (cx - c.width / 2 - offset.x) / scale,
      y: -(cy - c.height / 2 - offset.y) / scale,
    };
  }, [scale, offset]);

  const snapPoint = useCallback((wx: number, wy: number): { x: number; y: number; snapped: boolean } => {
    if (!snapEnabled || !sketch) return { x: wx, y: wy, snapped: false };
    const threshold = 8 / scale;
    let best = { x: wx, y: wy, dist: Infinity, snapped: false };

    for (const entity of sketch.entities) {
      if (entity.entity_type === 'Line' && entity.points.length >= 2) {
        for (const pt of [entity.points[0], entity.points[1]]) {
          const d = Math.hypot(pt.x - wx, pt.y - wy);
          if (d < best.dist && d < threshold) {
            best = { x: pt.x, y: pt.y, dist: d, snapped: true };
          }
        }
        const mx = (entity.points[0].x + entity.points[1].x) / 2;
        const my = (entity.points[0].y + entity.points[1].y) / 2;
        const dm = Math.hypot(mx - wx, my - wy);
        if (dm < best.dist && dm < threshold) {
          best = { x: mx, y: my, dist: dm, snapped: true };
        }
      }
      if ((entity.entity_type === 'Circle' || entity.entity_type === 'Arc') && entity.points.length >= 1) {
        const c = entity.points[0];
        const d = Math.hypot(c.x - wx, c.y - wy);
        if (d < best.dist && d < threshold) {
          best = { x: c.x, y: c.y, dist: d, snapped: true };
        }
      }
      if (entity.entity_type === 'Rectangle' || entity.entity_type === 'Polygon' || entity.entity_type === 'Spline' || entity.entity_type === 'Polyline') {
        for (const pt of entity.points) {
          const d = Math.hypot(pt.x - wx, pt.y - wy);
          if (d < best.dist && d < threshold) {
            best = { x: pt.x, y: pt.y, dist: d, snapped: true };
          }
        }
      }
    }

    if (best.snapped) return { x: best.x, y: best.y, snapped: true };
    return { x: wx, y: wy, snapped: false };
  }, [snapEnabled, sketch, scale]);

  const snapRef = useRef<{ x: number; y: number; snapped: boolean }>({ x: 0, y: 0, snapped: false });

  const draw = useCallback(() => {
    const c = canvasRef.current;
    if (!c) return;
    const ctx = c.getContext('2d');
    if (!ctx) return;

    const w = c.width;
    const h = c.height;
    ctx.clearRect(0, 0, w, h);

    // background
    ctx.fillStyle = '#1a1a2e';
    ctx.fillRect(0, 0, w, h);

    // grid
    ctx.strokeStyle = '#2a2a4e';
    ctx.lineWidth = 0.5;
    const gridStep = scale > 40 ? 1 : scale > 20 ? 2 : 5;
    const origin = toCanvas(0, 0);
    for (let gx = origin.x % (gridStep * scale); gx < w; gx += gridStep * scale) {
      ctx.beginPath(); ctx.moveTo(gx, 0); ctx.lineTo(gx, h); ctx.stroke();
    }
    for (let gy = origin.y % (gridStep * scale); gy < h; gy += gridStep * scale) {
      ctx.beginPath(); ctx.moveTo(0, gy); ctx.lineTo(w, gy); ctx.stroke();
    }

    // axes
    ctx.strokeStyle = '#ff4444';
    ctx.lineWidth = 2;
    const axEnd = toCanvas(2, 0);
    ctx.beginPath(); ctx.moveTo(origin.x, origin.y); ctx.lineTo(axEnd.x, axEnd.y); ctx.stroke();
    ctx.fillStyle = '#ff4444';
    ctx.font = '12px monospace';
    ctx.fillText('X', axEnd.x + 4, axEnd.y + 4);

    ctx.strokeStyle = '#44ff44';
    const ayEnd = toCanvas(0, 2);
    ctx.beginPath(); ctx.moveTo(origin.x, origin.y); ctx.lineTo(ayEnd.x, ayEnd.y); ctx.stroke();
    ctx.fillStyle = '#44ff44';
    ctx.fillText('Y', ayEnd.x + 4, ayEnd.y + 4);

    // section lines (3D实体剖视图轮廓)
    if (sectionLines.length > 0) {
      ctx.save();
      ctx.strokeStyle = '#6688cc';
      ctx.lineWidth = 2;
      ctx.globalAlpha = 0.7;
      for (const seg of sectionLines) {
        const p1 = toCanvas(seg.x1, seg.y1);
        const p2 = toCanvas(seg.x2, seg.y2);
        ctx.beginPath(); ctx.moveTo(p1.x, p1.y); ctx.lineTo(p2.x, p2.y); ctx.stroke();
      }
      ctx.restore();
    }

    // draw entities
    if (sketch) {
      sketch.entities.forEach((entity, idx) => {
      const isSelected = idx === selectedEntityIdx;
      const isSolved = sketch.solved;
      const isConstruction = entity.construction;

      if (isConstruction) {
        ctx.strokeStyle = isSelected ? '#ffaa00' : '#556677';
        ctx.lineWidth = isSelected ? 3 : 1.5;
        ctx.setLineDash([6, 4]);
      } else {
        ctx.strokeStyle = isSelected ? '#ffaa00' : isSolved ? '#4fc3f7' : '#ff4444';
        ctx.lineWidth = isSelected ? 3 : 2;
        if (!isSolved) ctx.setLineDash([4, 4]);
      }

      switch (entity.entity_type) {
        case 'Line': {
          if (entity.points.length >= 2) {
            const p1 = toCanvas(entity.points[0].x, entity.points[0].y);
            const p2 = toCanvas(entity.points[1].x, entity.points[1].y);
            ctx.beginPath(); ctx.moveTo(p1.x, p1.y); ctx.lineTo(p2.x, p2.y); ctx.stroke();
            drawPoint(ctx, p1.x, p1.y, isSelected);
            drawPoint(ctx, p2.x, p2.y, isSelected);
          }
          break;
        }
        case 'Circle': {
          if (entity.points.length >= 1) {
            const center = toCanvas(entity.points[0].x, entity.points[0].y);
            const r = entity.radius * scale;
            ctx.beginPath(); ctx.arc(center.x, center.y, r, 0, Math.PI * 2); ctx.stroke();
            drawPoint(ctx, center.x, center.y, isSelected);
          }
          break;
        }
        case 'Arc': {
          if (entity.points.length >= 3) {
            const center = toCanvas(entity.points[0].x, entity.points[0].y);
            const r = entity.radius * scale;
            const startAngle = Math.atan2(entity.points[1].y - entity.points[0].y, entity.points[1].x - entity.points[0].x);
            const endAngle = Math.atan2(entity.points[2].y - entity.points[0].y, entity.points[2].x - entity.points[0].x);
            ctx.beginPath(); ctx.arc(center.x, center.y, r, -startAngle, -endAngle); ctx.stroke();
            drawPoint(ctx, center.x, center.y, isSelected);
          }
          break;
        }
        case 'Rectangle': {
          if (entity.points.length >= 4) {
            ctx.beginPath();
            for (let i = 0; i < 4; i++) {
              const p = toCanvas(entity.points[i].x, entity.points[i].y);
              if (i === 0) ctx.moveTo(p.x, p.y);
              else ctx.lineTo(p.x, p.y);
            }
            ctx.closePath(); ctx.stroke();
            entity.points.forEach(p => {
              const cp = toCanvas(p.x, p.y);
              drawPoint(ctx, cp.x, cp.y, isSelected);
            });
          }
          break;
        }
        case 'Polygon': {
          if (entity.points.length >= 3) {
            ctx.beginPath();
            entity.points.forEach((p, i) => {
              const cp = toCanvas(p.x, p.y);
              if (i === 0) ctx.moveTo(cp.x, cp.y);
              else ctx.lineTo(cp.x, cp.y);
            });
            ctx.closePath(); ctx.stroke();
            entity.points.forEach(p => {
              const cp = toCanvas(p.x, p.y);
              drawPoint(ctx, cp.x, cp.y, isSelected);
            });
          }
          break;
        }
        case 'Ellipse': {
          if (entity.points.length >= 3) {
            const cx = toCanvas(entity.points[0].x, entity.points[0].y);
            const ex = toCanvas(entity.points[1].x, entity.points[1].y);
            const ey = toCanvas(entity.points[2].x, entity.points[2].y);
            const rx = Math.hypot(ex.x - cx.x, ex.y - cx.y);
            const ry = Math.hypot(ey.x - cx.x, ey.y - cx.y);
            const angle = Math.atan2(ex.y - cx.y, ex.x - cx.x);
            ctx.beginPath();
            ctx.ellipse(cx.x, cx.y, rx, ry, angle, 0, Math.PI * 2);
            ctx.stroke();
            drawPoint(ctx, cx.x, cx.y, isSelected);
          }
          break;
        }
        case 'Spline':
        case 'Polyline': {
          if (entity.points.length >= 2) {
            ctx.beginPath();
            const p0 = toCanvas(entity.points[0].x, entity.points[0].y);
            ctx.moveTo(p0.x, p0.y);
            if (entity.entity_type === 'Spline' && entity.points.length >= 3) {
              for (let i = 1; i < entity.points.length - 1; i++) {
                const p = toCanvas(entity.points[i].x, entity.points[i].y);
                const pn = toCanvas(entity.points[i + 1].x, entity.points[i + 1].y);
                const mx = (p.x + pn.x) / 2;
                const my = (p.y + pn.y) / 2;
                ctx.quadraticCurveTo(p.x, p.y, mx, my);
              }
              const last = toCanvas(entity.points[entity.points.length - 1].x, entity.points[entity.points.length - 1].y);
              ctx.lineTo(last.x, last.y);
            } else {
              for (let i = 1; i < entity.points.length; i++) {
                const p = toCanvas(entity.points[i].x, entity.points[i].y);
                ctx.lineTo(p.x, p.y);
              }
            }
            ctx.stroke();
            entity.points.forEach(p => {
              const cp = toCanvas(p.x, p.y);
              drawPoint(ctx, cp.x, cp.y, isSelected);
            });
          }
          break;
        }
      }
      ctx.setLineDash([]);
    });
    }

    // closed contour fill
    if (sketch && sketch.solved) {
      const contours = findClosedContours(sketch);
      ctx.save();
      for (const contour of contours) {
        if (contour.length < 3) continue;
        ctx.beginPath();
        for (let i = 0; i < contour.length; i++) {
          const cp = toCanvas(contour[i].x, contour[i].y);
          if (i === 0) ctx.moveTo(cp.x, cp.y);
          else ctx.lineTo(cp.x, cp.y);
        }
        ctx.closePath();
        ctx.fillStyle = 'rgba(68, 170, 255, 0.08)';
        ctx.fill();
        ctx.strokeStyle = 'rgba(68, 170, 255, 0.25)';
        ctx.lineWidth = 1;
        ctx.setLineDash([4, 4]);
        ctx.stroke();
        ctx.setLineDash([]);
      }
      ctx.restore();
    }

    // constraints
    if (sketch) {
      drawAllConstraints(ctx, sketch, toCanvas, scale);
    }

    // grip handles for selected entity
    if (sketch && selectedEntityIdx !== null && selectedEntityIdx >= 0 && selectedEntityIdx < sketch.entities.length) {
      const entity = sketch.entities[selectedEntityIdx];
      const grips = getGripPoints(entity);
      grips.forEach((gp, gi) => {
        const cp = toCanvas(gp.x, gp.y);
        drawGripHandle(ctx, cp.x, cp.y, gi === gripHoverRef.current);
      });
      // If currently dragging a grip, show the drag preview
      if (gripDragRef.current && gripDragPos) {
        const dp = toCanvas(gripDragPos.x, gripDragPos.y);
        ctx.fillStyle = '#ffaa00';
        ctx.beginPath(); ctx.arc(dp.x, dp.y, 5, 0, Math.PI * 2); ctx.fill();
        ctx.strokeStyle = '#ffaa00';
        ctx.lineWidth = 1;
        ctx.setLineDash([3, 3]);
        ctx.beginPath(); ctx.arc(dp.x, dp.y, 8, 0, Math.PI * 2); ctx.stroke();
        ctx.setLineDash([]);
      }
    }

    // preview while drawing
    if (pendingLineStart || pendingPolyPoints.length > 0) {
      ctx.strokeStyle = '#ffaa00';
      ctx.lineWidth = 2;
      ctx.setLineDash([6, 4]);

      if (pendingLineStart) {
        const p1 = toCanvas(pendingLineStart.x, pendingLineStart.y);
        drawPoint(ctx, p1.x, p1.y, true);
      }

      if (pendingPolyPoints.length > 0) {
        ctx.beginPath();
        const p0 = toCanvas(pendingPolyPoints[0].x, pendingPolyPoints[0].y);
        ctx.moveTo(p0.x, p0.y);
        for (let i = 1; i < pendingPolyPoints.length; i++) {
          const p = toCanvas(pendingPolyPoints[i].x, pendingPolyPoints[i].y);
          ctx.lineTo(p.x, p.y);
        }
        if (pendingLineStart && (activeTool === 'rect' || activeTool === 'polygon' || activeTool === 'ellipse')) {
          const pm = toCanvas(mousePos.x, mousePos.y);
          ctx.lineTo(pm.x, pm.y);
        }
        if (pendingLineStart && (activeTool === 'spline' || activeTool === 'polyline')) {
          const pm = toCanvas(mousePos.x, mousePos.y);
          ctx.lineTo(pm.x, pm.y);
        }
        ctx.stroke();
        pendingPolyPoints.forEach(p => {
          const cp = toCanvas(p.x, p.y);
          drawPoint(ctx, cp.x, cp.y, true);
        });
      }

      if (pendingLineStart && pendingPolyPoints.length === 0) {
        const p2 = toCanvas(mousePos.x, mousePos.y);
        ctx.beginPath();
        const p1 = toCanvas(pendingLineStart.x, pendingLineStart.y);
        ctx.moveTo(p1.x, p1.y);
        if (activeTool === 'rect') {
          ctx.lineTo(p2.x, p1.y);
          ctx.lineTo(p2.x, p2.y);
          ctx.lineTo(p1.x, p2.y);
          ctx.closePath();
        } else if (activeTool === 'polygon' || activeTool === 'circle') {
          const r = Math.hypot(p2.x - p1.x, p2.y - p1.y);
          ctx.arc(p1.x, p1.y, r, 0, Math.PI * 2);
        } else {
          ctx.lineTo(p2.x, p2.y);
        }
        ctx.stroke();
      }

      ctx.setLineDash([]);
    }
  }, [sketch, sectionLines, selectedEntityIdx, pendingLineStart, pendingPolyPoints, mousePos, scale, offset, toCanvas, activeTool]);

  function drawPoint(ctx: CanvasRenderingContext2D, x: number, y: number, selected: boolean) {
    ctx.fillStyle = selected ? '#ffaa00' : '#fff';
    ctx.beginPath(); ctx.arc(x, y, 4, 0, Math.PI * 2); ctx.fill();
  }

  function getEntityCenter(entity: Sketch['entities'][0]): { x: number; y: number } {
    if (entity.entity_type === 'Circle' || entity.entity_type === 'Arc') {
      return { x: entity.points[0]?.x ?? 0, y: entity.points[0]?.y ?? 0 };
    }
    if (entity.entity_type === 'Line' && entity.points.length >= 2) {
      return {
        x: (entity.points[0].x + entity.points[1].x) / 2,
        y: (entity.points[0].y + entity.points[1].y) / 2,
      };
    }
    let cx = 0, cy = 0, count = 0;
    for (const pt of entity.points) { cx += pt.x; cy += pt.y; count++; }
    if (count === 0) return { x: 0, y: 0 };
    return { x: cx / count, y: cy / count };
  }

  function getGripPoints(entity: Sketch['entities'][0]): { x: number; y: number }[] {
    switch (entity.entity_type) {
      case 'Line': {
        if (entity.points.length >= 2) return [entity.points[0], entity.points[1]];
        return [];
      }
      case 'Circle': {
        if (entity.points.length >= 1) {
          const c = entity.points[0];
          const r = entity.radius ?? 1;
          return [c, { x: c.x + r, y: c.y }];
        }
        return [];
      }
      case 'Arc': {
        if (entity.points.length >= 3) {
          const c = entity.points[0];
          const r = entity.radius ?? 1;
          return [c, entity.points[1], entity.points[2], { x: c.x + r, y: c.y }];
        }
        if (entity.points.length >= 1) {
          const c = entity.points[0];
          const r = entity.radius ?? 1;
          return [c, { x: c.x + r, y: c.y }];
        }
        return [];
      }
      case 'Rectangle':
      case 'Polygon':
      case 'Polyline':
      case 'Spline': {
        return entity.points.map((p) => ({ x: p.x, y: p.y }));
      }
      case 'Ellipse': {
        if (entity.points.length >= 3) {
          return [entity.points[0], entity.points[1], entity.points[2]];
        }
        return entity.points.map((p) => ({ x: p.x, y: p.y }));
      }
      default:
        return [];
    }
  }

  function drawGripHandle(ctx: CanvasRenderingContext2D, x: number, y: number, hovered: boolean) {
    const size = 6;
    ctx.fillStyle = hovered ? '#ffaa00' : '#ffffff';
    ctx.strokeStyle = '#222244';
    ctx.lineWidth = 1.5;
    ctx.fillRect(x - size / 2, y - size / 2, size, size);
    ctx.strokeRect(x - size / 2, y - size / 2, size, size);
  }

  function drawConstraintSymbol(
    ctx: CanvasRenderingContext2D, label: string,
    cx: number, cy: number, ox: number, oy: number,
  ) {
    ctx.save();
    ctx.font = 'bold 11px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const tw = ctx.measureText(label).width;
    const pad = 3;
    ctx.fillStyle = 'rgba(26,26,46,0.8)';
    ctx.fillRect(cx + ox - tw / 2 - pad, cy + oy - 7 - pad, tw + pad * 2, 14 + pad * 2);
    ctx.fillStyle = '#ffcc00';
    ctx.fillText(label, cx + ox, cy + oy);
    ctx.restore();
  }

  function drawArrow(ctx: CanvasRenderingContext2D, fromX: number, fromY: number, toX: number, toY: number, size: number) {
    const angle = Math.atan2(toY - fromY, toX - fromX);
    ctx.beginPath();
    ctx.moveTo(toX, toY);
    ctx.lineTo(toX - size * Math.cos(angle - Math.PI / 6), toY - size * Math.sin(angle - Math.PI / 6));
    ctx.lineTo(toX - size * Math.cos(angle + Math.PI / 6), toY - size * Math.sin(angle + Math.PI / 6));
    ctx.closePath();
    ctx.fill();
  }

  function drawAllConstraints(
    ctx: CanvasRenderingContext2D,
    sketch: Sketch,
    toCanvas: (wx: number, wy: number) => { x: number; y: number },
    scale: number,
  ) {
    ctx.save();
    ctx.strokeStyle = '#ffcc00';
    ctx.fillStyle = '#ffcc00';
    ctx.lineWidth = 1;
    ctx.font = '10px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    dimensionHitRef.current = [];

    sketch.constraints.forEach((constraint, ci) => {
      const { constraint_type, entity_refs, value } = constraint;
      const entities = entity_refs.map(r => sketch.entities[r.entity_index]).filter(Boolean);
      if (entities.length === 0) return;

      const entity = entities[0]!;
      const center = getEntityCenter(entity);
      const cp = toCanvas(center.x, center.y);
      const offsetY = -18;

      // Dimension constraints need special handling
      if (constraint_type === 'Length' && entity.entity_type === 'Line' && entity.points.length >= 2) {
        const hit = drawLengthDimension(ctx, entity, value, toCanvas, scale);
        if (hit) dimensionHitRef.current.push({ x: hit.x, y: hit.y, constraintIdx: ci });
        return;
      }
      if (constraint_type === 'Distance' && entities.length >= 2) {
        const hit = drawDistanceDimension(ctx, entities, value, toCanvas, scale);
        if (hit) dimensionHitRef.current.push({ x: hit.x, y: hit.y, constraintIdx: ci });
        return;
      }
      if (constraint_type === 'Angle' && entities.length >= 2) {
        const hit = drawAngleDimension(ctx, entities, value, toCanvas, scale);
        if (hit) dimensionHitRef.current.push({ x: hit.x, y: hit.y, constraintIdx: ci });
        return;
      }
      if (constraint_type === 'Radius' && (entity.entity_type === 'Circle' || entity.entity_type === 'Arc')) {
        const hit = drawRadiusDimension(ctx, entity, value, toCanvas, scale);
        if (hit) dimensionHitRef.current.push({ x: hit.x, y: hit.y, constraintIdx: ci });
        return;
      }
      if (constraint_type === 'Diameter' && (entity.entity_type === 'Circle' || entity.entity_type === 'Arc')) {
        const hit = drawDiameterDimension(ctx, entity, value, toCanvas, scale);
        if (hit) dimensionHitRef.current.push({ x: hit.x, y: hit.y, constraintIdx: ci });
        return;
      }

      // Non-dimension constraints: draw symbol
      const symbol = CONSTRAINT_SYMBOLS[constraint_type];
      if (symbol) {
        drawConstraintSymbol(ctx, symbol, cp.x, cp.y, 0, offsetY);
      }
    });

    ctx.restore();
  }

  function drawLengthDimension(
    ctx: CanvasRenderingContext2D, entity: Sketch['entities'][0],
    value: number, toCanvas: (wx: number, wy: number) => { x: number; y: number },
    scale: number,
  ): { x: number; y: number } | null {
    const p1 = toCanvas(entity.points[0].x, entity.points[0].y);
    const p2 = toCanvas(entity.points[1].x, entity.points[1].y);
    const dx = p2.x - p1.x, dy = p2.y - p1.y;
    const len = Math.hypot(dx, dy);
    if (len < 1) return null;
    const nx = -dy / len, ny = dx / len;
    const offset = 30;

    const ox = nx * offset, oy = ny * offset;
    const d1x = p1.x + ox, d1y = p1.y + oy;
    const d2x = p2.x + ox, d2y = p2.y + oy;

    // Extension lines
    ctx.strokeStyle = '#ffcc00';
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    ctx.beginPath(); ctx.moveTo(p1.x, p1.y); ctx.lineTo(d1x, d1y); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(p2.x, p2.y); ctx.lineTo(d2x, d2y); ctx.stroke();
    ctx.setLineDash([]);

    // Dimension line
    ctx.beginPath(); ctx.moveTo(d1x, d1y); ctx.lineTo(d2x, d2y); ctx.stroke();

    // Arrows
    const arrowSize = 6;
    drawArrow(ctx, d1x, d1y, d2x, d2y, arrowSize);
    drawArrow(ctx, d2x, d2y, d1x, d1y, arrowSize);

    // Text
    const actualLen = Math.hypot(entity.points[1].x - entity.points[0].x, entity.points[1].y - entity.points[0].y);
    const displayValue = value !== 0 ? value : actualLen;
    ctx.save();
    ctx.font = 'bold 11px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'bottom';
    const tx = (d1x + d2x) / 2;
    const ty = (d1y + d2y) / 2;
    const label = displayValue.toFixed(2);
    const tw = ctx.measureText(label).width;
    ctx.fillStyle = 'rgba(26,26,46,0.85)';
    ctx.fillRect(tx - tw / 2 - 3, ty - 18, tw + 6, 18);
    ctx.fillStyle = '#ffcc00';
    const textOffset = nx < 0 ? -8 : 8;
    ctx.fillText(label, tx, ty + (ny > 0 ? textOffset : -textOffset));
    ctx.restore();
    return { x: tx, y: ty + (ny > 0 ? textOffset : -textOffset) };
  }

  function drawDistanceDimension(
    ctx: CanvasRenderingContext2D, entities: Sketch['entities'],
    value: number, toCanvas: (wx: number, wy: number) => { x: number; y: number },
    scale: number,
  ): { x: number; y: number } | null {
    const c1 = getEntityCenter(entities[0]!);
    const c2 = getEntityCenter(entities[1]!);
    const p1 = toCanvas(c1.x, c1.y);
    const p2 = toCanvas(c2.x, c2.y);
    const dx = p2.x - p1.x, dy = p2.y - p1.y;
    const len = Math.hypot(dx, dy);
    if (len < 1) return null;
    const nx = -dy / len, ny = dx / len;
    const offset = 20;

    const ox = nx * offset, oy = ny * offset;
    const d1x = p1.x + ox, d1y = p1.y + oy;
    const d2x = p2.x + ox, d2y = p2.y + oy;

    ctx.strokeStyle = '#ffcc00';
    ctx.lineWidth = 1;
    ctx.setLineDash([3, 3]);
    ctx.beginPath(); ctx.moveTo(p1.x, p1.y); ctx.lineTo(d1x, d1y); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(p2.x, p2.y); ctx.lineTo(d2x, d2y); ctx.stroke();
    ctx.setLineDash([]);

    ctx.beginPath(); ctx.moveTo(d1x, d1y); ctx.lineTo(d2x, d2y); ctx.stroke();

    const arrowSize = 6;
    drawArrow(ctx, d1x, d1y, d2x, d2y, arrowSize);
    drawArrow(ctx, d2x, d2y, d1x, d1y, arrowSize);

    const actualDist = Math.hypot(c2.x - c1.x, c2.y - c1.y);
    const displayValue = value !== 0 ? value : actualDist;
    ctx.save();
    ctx.font = 'bold 11px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'bottom';
    const tx = (d1x + d2x) / 2;
    const ty = (d1y + d2y) / 2;
    const label = displayValue.toFixed(2);
    const tw = ctx.measureText(label).width;
    ctx.fillStyle = 'rgba(26,26,46,0.85)';
    ctx.fillRect(tx - tw / 2 - 3, ty - 18, tw + 6, 18);
    ctx.fillStyle = '#ffcc00';
    ctx.fillText(label, tx, ty + (ny > 0 ? -8 : 8));
    ctx.restore();
    return { x: tx, y: ty + (ny > 0 ? -8 : 8) };
  }

  function drawAngleDimension(
    ctx: CanvasRenderingContext2D, entities: Sketch['entities'],
    value: number, toCanvas: (wx: number, wy: number) => { x: number; y: number },
    scale: number,
  ): { x: number; y: number } | null {
    const e0 = entities[0]!, e1 = entities[1]!;
    if (e0.points.length < 2 || e1.points.length < 2) return null;

    let vx = 0, vy = 0, found = false;
    for (const p0 of e0.points) {
      for (const p1 of e1.points) {
        if (Math.hypot(p0.x - p1.x, p0.y - p1.y) < 1e-6) {
          vx = p0.x; vy = p0.y; found = true; break;
        }
      }
      if (found) break;
    }
    if (!found) {
      vx = e0.points[0].x; vy = e0.points[0].y;
    }

    const getDir = (e: Sketch['entities'][0], vx: number, vy: number) => {
      for (const p of e.points) {
        const dx = p.x - vx, dy = p.y - vy;
        if (Math.hypot(dx, dy) > 1e-6) return { x: dx, y: dy };
      }
      return { x: 1, y: 0 };
    };
    const d0 = getDir(e0, vx, vy);
    const d1 = getDir(e1, vx, vy);

    const angle0 = Math.atan2(d0.y, d0.x);
    const angle1 = Math.atan2(d1.y, d1.x);

    const vp = toCanvas(vx, vy);
    const arcRadius = 40;

    ctx.strokeStyle = '#ffcc00';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.arc(vp.x, vp.y, arcRadius, -angle0, -angle1);
    ctx.stroke();

    const midAngle = (angle0 + angle1) / 2;
    const displayAngle = value !== 0 ? value * (180 / Math.PI) : Math.abs((angle1 - angle0) * 180 / Math.PI);
    ctx.save();
    ctx.font = 'bold 10px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const label = displayAngle.toFixed(1) + '°';
    const tx = vp.x + (arcRadius + 16) * Math.cos(-midAngle);
    const ty = vp.y + (arcRadius + 16) * Math.sin(-midAngle);
    const tw = ctx.measureText(label).width;
    ctx.fillStyle = 'rgba(26,26,46,0.85)';
    ctx.fillRect(tx - tw / 2 - 3, ty - 8, tw + 6, 16);
    ctx.fillStyle = '#ffcc00';
    ctx.fillText(label, tx, ty);
    ctx.restore();
    return { x: tx, y: ty };
  }

  function drawRadiusDimension(
    ctx: CanvasRenderingContext2D, entity: Sketch['entities'][0],
    value: number, toCanvas: (wx: number, wy: number) => { x: number; y: number },
    scale: number,
  ): { x: number; y: number } | null {
    if (entity.points.length < 1) return null;
    const cp = toCanvas(entity.points[0].x, entity.points[0].y);
    const label = 'R' + (value !== 0 ? value : entity.radius).toFixed(2);

    ctx.save();
    ctx.font = 'bold 11px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const tw = ctx.measureText(label).width;
    ctx.fillStyle = 'rgba(26,26,46,0.85)';
    ctx.fillRect(cp.x + 12 - tw / 2 - 3, cp.y - 8, tw + 6, 16);
    ctx.fillStyle = '#ffcc00';
    ctx.fillText(label, cp.x + 12, cp.y);
    ctx.restore();
    return { x: cp.x + 12, y: cp.y };
  }

  function drawDiameterDimension(
    ctx: CanvasRenderingContext2D, entity: Sketch['entities'][0],
    value: number, toCanvas: (wx: number, wy: number) => { x: number; y: number },
    scale: number,
  ): { x: number; y: number } | null {
    if (entity.points.length < 1) return null;
    const cp = toCanvas(entity.points[0].x, entity.points[0].y);
    const d = (value !== 0 ? value : entity.radius * 2);
    const label = 'Ø' + d.toFixed(2);

    ctx.save();
    ctx.font = 'bold 11px monospace';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const tw = ctx.measureText(label).width;
    ctx.fillStyle = 'rgba(26,26,46,0.85)';
    ctx.fillRect(cp.x + 12 - tw / 2 - 3, cp.y - 8, tw + 6, 16);
    ctx.fillStyle = '#ffcc00';
    ctx.fillText(label, cp.x + 12, cp.y);
    ctx.restore();
    return { x: cp.x + 12, y: cp.y };
  }

  const CONSTRAINT_SYMBOLS: Record<string, string> = {
    Horizontal: '≡',
    Vertical: '↕',
    Coincident: '⦿',
    Parallel: '∥',
    Perpendicular: '⊥',
    Fixed: '⚓',
    Tangent: '⌒',
    AtMidpoint: '◉',
    EqualLength: '=',
    EqualRadius: '=',
    Symmetric: '↔',
    PtOnLine: '×',
    PtOnCircle: '×',
    SameOrientation: '↗',
  };

  useEffect(() => { draw(); }, [draw]);

  useEffect(() => {
    const c = canvasRef.current;
    if (!c) return;
    const ro = new ResizeObserver(() => {
      const parent = c.parentElement;
      if (parent) {
        c.width = parent.clientWidth;
        c.height = parent.clientHeight;
        draw();
      }
    });
    ro.observe(c.parentElement!);
    return () => ro.disconnect();
  }, [draw]);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 1 || e.button === 2) {
      dragRef.current = { start: { x: e.clientX, y: e.clientY }, offset: { ...offset } };
      return;
    }
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;
    const world = toWorld(cx, cy);

    if (activeTool !== 'select') {
      const snap = snapPoint(world.x, world.y);
      onCanvasClick(snap.x, snap.y);
    } else if (sketch) {
      // Check dimension label hits first
      if (onEditConstraint) {
        for (const area of dimensionHitRef.current) {
          if (Math.abs(cx - area.x) < 30 && Math.abs(cy - area.y) < 12) {
            onEditConstraint(area.constraintIdx);
            return;
          }
        }
      }

      // Check grip handle hit on selected entity
      if (selectedEntityIdx !== null && selectedEntityIdx >= 0 && selectedEntityIdx < sketch.entities.length) {
        const entity = sketch.entities[selectedEntityIdx];
        const grips = getGripPoints(entity);
        const gripThreshold = 12 / scale;
        for (let gi = 0; gi < grips.length; gi++) {
          const d = Math.hypot(world.x - grips[gi].x, world.y - grips[gi].y);
          if (d < gripThreshold) {
            gripDragRef.current = { entityIdx: selectedEntityIdx, pointIdx: gi, startX: world.x, startY: world.y };
            setGripDragPos({ x: grips[gi].x, y: grips[gi].y });
            return;
          }
        }
      }

      // Then check entity selection
      let nearest: number | null = null;
      let minDist = 20 / scale;
      sketch.entities.forEach((entity, idx) => {
        if (entity.entity_type === 'Line' && entity.points.length >= 2) {
          const dist = pointToSegmentDist(
            world.x, world.y,
            entity.points[0].x, entity.points[0].y,
            entity.points[1].x, entity.points[1].y
          );
          if (dist < minDist) { minDist = dist; nearest = idx; }
        }
      });
      onSelectEntity(nearest);
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (gripDragRef.current) {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (rect) {
        const world = toWorld(e.clientX - rect.left, e.clientY - rect.top);
        const snap = snapPoint(world.x, world.y);
        setGripDragPos({ x: snap.x, y: snap.y });
      }
      return;
    }
    if (dragRef.current) {
      const dx = e.clientX - dragRef.current.start.x;
      const dy = e.clientY - dragRef.current.start.y;
      setOffset({ x: dragRef.current.offset.x + dx, y: dragRef.current.offset.y + dy });
    }
    const rect = canvasRef.current?.getBoundingClientRect();
    if (rect) {
      const world = toWorld(e.clientX - rect.left, e.clientY - rect.top);
      const snap = snapPoint(world.x, world.y);
      snapRef.current = snap;
      onMouseMoveWorld(snap.x, snap.y);

      // Grip hover detection
      let hoveredGrip: number | null = null;
      if (sketch && selectedEntityIdx !== null && selectedEntityIdx >= 0 && selectedEntityIdx < sketch.entities.length) {
        const entity = sketch.entities[selectedEntityIdx];
        const grips = getGripPoints(entity);
        const gripThreshold = 12 / scale;
        for (let gi = 0; gi < grips.length; gi++) {
          const d = Math.hypot(world.x - grips[gi].x, world.y - grips[gi].y);
          if (d < gripThreshold) {
            hoveredGrip = gi;
            break;
          }
        }
      }
      if (hoveredGrip !== gripHoverRef.current) {
        gripHoverRef.current = hoveredGrip;
        draw();
      }
    }
  };

  const handleClick = (e: React.MouseEvent) => {
    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;
    const world = toWorld(e.clientX - rect.left, e.clientY - rect.top);
    const snap = snapPoint(world.x, world.y);
    onCanvasClick(snap.x, snap.y);
  };

  const handleMouseUp = () => {
    if (gripDragRef.current && gripDragPos) {
      const dx = gripDragPos.x - gripDragRef.current.startX;
      const dy = gripDragPos.y - gripDragRef.current.startY;
      if (Math.abs(dx) > 1e-6 || Math.abs(dy) > 1e-6) {
        onDragEntityPoint?.(gripDragRef.current.entityIdx, gripDragRef.current.pointIdx, gripDragPos.x, gripDragPos.y);
      }
      gripDragRef.current = null;
      setGripDragPos(null);
      return;
    }
    dragRef.current = null;
  };

  const handleWheel = (e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setScale((s) => Math.max(10, Math.min(500, s * delta)));
  };

  return (
    <canvas
      ref={canvasRef}
      className="sketch-canvas"
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onWheel={handleWheel}
      onContextMenu={(e) => e.preventDefault()}
    />
  );
}

function findClosedContours(sketch: Sketch): { x: number; y: number }[][] {
  const lines: { a: { x: number; y: number }; b: { x: number; y: number } }[] = [];
  for (const entity of sketch.entities) {
    if (entity.entity_type === 'Line' && entity.points.length >= 2) {
      lines.push({ a: entity.points[0], b: entity.points[1] });
    }
    if (entity.entity_type === 'Polygon' && entity.points.length >= 3) {
      for (let i = 0; i < entity.points.length; i++) {
        lines.push({ a: entity.points[i], b: entity.points[(i + 1) % entity.points.length] });
      }
    }
    if (entity.entity_type === 'Rectangle' && entity.points.length >= 4) {
      for (let i = 0; i < 4; i++) {
        lines.push({ a: entity.points[i], b: entity.points[(i + 1) % 4] });
      }
    }
  }

  const eps = 1e-6;
  function pointKey(p: { x: number; y: number }): string {
    return `${p.x.toFixed(4)},${p.y.toFixed(4)}`;
  }

  function pointEqual(a: { x: number; y: number }, b: { x: number; y: number }): boolean {
    return Math.abs(a.x - b.x) < eps && Math.abs(a.y - b.y) < eps;
  }

  // Build adjacency: point -> list of connected points
  const adj = new Map<string, { x: number; y: number }[]>();
  function addEdge(a: { x: number; y: number }, b: { x: number; y: number }) {
    const ka = pointKey(a);
    if (!adj.has(ka)) adj.set(ka, []);
    const kb = pointKey(b);
    if (!adj.has(kb)) adj.set(kb, []);
    const arrA = adj.get(ka)!;
    const arrB = adj.get(kb)!;
    if (!arrA.some((p) => pointEqual(p, b))) arrA.push(b);
    if (!arrB.some((p) => pointEqual(p, a))) arrB.push(a);
  }

  for (const line of lines) {
    addEdge(line.a, line.b);
  }

  // Find cycles using DFS
  const visited = new Set<string>();
  const contours: { x: number; y: number }[][] = [];

  function dfs(start: string, current: string, path: { x: number; y: number }[], depth: number) {
    if (depth > 0 && current === start && path.length >= 3) {
      contours.push([...path]);
      return;
    }
    if (depth > 0 && current === start) return; // 2-point cycle not valid
    if (depth > 0 && visited.has(current)) return;
    if (depth > 20) return; // limit depth

    visited.add(current);
    const neighbors = adj.get(current) || [];
    for (const next of neighbors) {
      const nk = pointKey(next);
      // Skip going back to the immediate previous point
      if (depth > 0 && path.length > 0 && pointEqual(next, path[path.length - 1])) continue;
      path.push(next);
      dfs(start, nk, path, depth + 1);
      path.pop();
    }
    if (depth > 0) visited.delete(current);
  }

  for (const [key] of adj) {
    visited.clear();
    dfs(key, key, [], 0);
  }

  // Remove duplicate contours (same set of points in different order)
  const unique: { x: number; y: number }[][] = [];
  for (const c of contours) {
    const cKeys = c.map((p) => pointKey(p)).sort().join('|');
    let isDup = false;
    for (const u of unique) {
      const uKeys = u.map((p) => pointKey(p)).sort().join('|');
      if (cKeys === uKeys) { isDup = true; break; }
    }
    if (!isDup) unique.push(c);
  }

  return unique;
}

function pointToSegmentDist(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const dx = bx - ax;
  const dy = by - ay;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-10) return Math.hypot(px - ax, py - ay);
  let t = ((px - ax) * dx + (py - ay) * dy) / len2;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(px - (ax + t * dx), py - (ay + t * dy));
}