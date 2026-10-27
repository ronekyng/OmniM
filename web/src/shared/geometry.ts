export type Point2D = { x: number; y: number };

export function lineLineIntersection(
  p1: Point2D, p2: Point2D,
  p3: Point2D, p4: Point2D,
): Point2D | null {
  const d1x = p2.x - p1.x, d1y = p2.y - p1.y;
  const d2x = p4.x - p3.x, d2y = p4.y - p3.y;
  const denom = d1x * d2y - d1y * d2x;
  if (Math.abs(denom) < 1e-10) return null;
  const t = ((p3.x - p1.x) * d2y - (p3.y - p1.y) * d2x) / denom;
  const u = ((p3.x - p1.x) * d1y - (p3.y - p1.y) * d1x) / denom;
  if (t < 0 || t > 1 || u < 0 || u > 1) return null;
  return { x: p1.x + t * d1x, y: p1.y + t * d1y };
}

export function pointOnSegmentParameter(pt: Point2D, a: Point2D, b: Point2D): number {
  const dx = b.x - a.x, dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-10) return 0;
  return ((pt.x - a.x) * dx + (pt.y - a.y) * dy) / len2;
}

export function projectPointOnLine(pt: Point2D, a: Point2D, b: Point2D): Point2D {
  const t = pointOnSegmentParameter(pt, a, b);
  const clamped = Math.max(0, Math.min(1, t));
  return { x: a.x + clamped * (b.x - a.x), y: a.y + clamped * (b.y - a.y) };
}

export function mirrorPoint(pt: Point2D, a: Point2D, b: Point2D): Point2D {
  const dx = b.x - a.x, dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-10) return pt;
  const t = ((pt.x - a.x) * dx + (pt.y - a.y) * dy) / len2;
  const projX = a.x + t * dx;
  const projY = a.y + t * dy;
  return { x: 2 * projX - pt.x, y: 2 * projY - pt.y };
}

export function offsetLine(a: Point2D, b: Point2D, dist: number): { a: Point2D; b: Point2D } {
  const dx = b.x - a.x, dy = b.y - a.y;
  const len = Math.hypot(dx, dy);
  if (len < 1e-10) return { a, b };
  const nx = -dy / len, ny = dx / len;
  return {
    a: { x: a.x + nx * dist, y: a.y + ny * dist },
    b: { x: b.x + nx * dist, y: b.y + ny * dist },
  };
}

export function distPointToSegment(pt: Point2D, a: Point2D, b: Point2D): number {
  const dx = b.x - a.x, dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-10) return Math.hypot(pt.x - a.x, pt.y - a.y);
  let t = ((pt.x - a.x) * dx + (pt.y - a.y) * dy) / len2;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(pt.x - (a.x + t * dx), pt.y - (a.y + t * dy));
}

export function findEntityIntersections(
  entityIdx: number,
  allEntities: { entity_type: string; points: Point2D[] }[],
): { t: number; point: Point2D }[] {
  const entity = allEntities[entityIdx];
  if (!entity || entity.entity_type !== 'Line' || entity.points.length < 2) return [];

  const a = entity.points[0], b = entity.points[1];
  const results: { t: number; point: Point2D }[] = [];

  for (let i = 0; i < allEntities.length; i++) {
    if (i === entityIdx) continue;
    const other = allEntities[i];
    if (other.entity_type !== 'Line' || other.points.length < 2) continue;
    const pt = lineLineIntersection(a, b, other.points[0], other.points[1]);
    if (pt) {
      const t = pointOnSegmentParameter(pt, a, b);
      if (t > 0.01 && t < 0.99) {
        results.push({ t, point: pt });
      }
    }
  }

  results.sort((a, b) => a.t - b.t);
  return results;
}

export function findNearestLineEndpoint(
  clickPt: Point2D,
  entityIdx: number,
  allEntities: { entity_type: string; points: Point2D[] }[],
  threshold: number,
): { endpoint: Point2D; otherEndpoint: Point2D; isStart: boolean } | null {
  const entity = allEntities[entityIdx];
  if (!entity || entity.entity_type !== 'Line' || entity.points.length < 2) return null;

  const start = entity.points[0], end = entity.points[1];
  const dStart = Math.hypot(clickPt.x - start.x, clickPt.y - start.y);
  const dEnd = Math.hypot(clickPt.x - end.x, clickPt.y - end.y);

  if (dStart < threshold) return { endpoint: start, otherEndpoint: end, isStart: true };
  if (dEnd < threshold) return { endpoint: end, otherEndpoint: start, isStart: false };
  return null;
}

export function pointToSegmentParam(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const dx = bx - ax, dy = by - ay;
  const len2 = dx * dx + dy * dy;
  if (len2 < 1e-10) return 0;
  return ((px - ax) * dx + (py - ay) * dy) / len2;
}

export function pointToSegmentDist(px: number, py: number, ax: number, ay: number, bx: number, by: number): number {
  const t = pointToSegmentParam(px, py, ax, ay, bx, by);
  const clamped = Math.max(0, Math.min(1, t));
  return Math.hypot(px - (ax + clamped * (bx - ax)), py - (ay + clamped * (by - ay)));
}

export function extendLineToIntersection(
  endpoint: Point2D,
  otherEndpoint: Point2D,
  entityIdx: number,
  allEntities: { entity_type: string; points: Point2D[] }[],
): Point2D | null {
  let bestPt: Point2D | null = null;
  let bestDist = Infinity;

  for (let i = 0; i < allEntities.length; i++) {
    if (i === entityIdx) continue;
    const other = allEntities[i];
    if (other.entity_type !== 'Line' || other.points.length < 2) continue;

    const pt = lineLineIntersection(endpoint, otherEndpoint, other.points[0], other.points[1]);
    if (!pt) continue;

    const dist = Math.hypot(pt.x - endpoint.x, pt.y - endpoint.y);
    if (dist < bestDist && dist > 1e-6) {
      const t = pointOnSegmentParameter(pt, endpoint, otherEndpoint);
      if (t > 0) {
        bestDist = dist;
        bestPt = pt;
      }
    }
  }

  return bestPt;
}