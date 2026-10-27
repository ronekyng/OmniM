import type { Point3D } from '../types/creation';

/**
 * Plan-view polygon validation for N-point picking previews (slab/roof/floor).
 * Mirrors the server's `validate_boundary` (crates/model/src/gen_rule/
 * layer_extrusion.rs): ≥3 points, no zero-length edges, non-collinear total
 * area, no self-intersection (bowtie) via O(n²) proper segment intersection.
 * Returns null when the polygon is builable, else a short zh description
 * usable as a hint/red-preview reason.
 */
export function polygonPreviewError(points: Point3D[]): string | null {
  const n = points.length;
  if (n < 3) return null; // below minimum is handled by the pick mode, not an error
  // Zero-length edges
  for (let i = 0; i < n; i++) {
    const a = points[i];
    const b = points[(i + 1) % n];
    if (Math.hypot(b.x - a.x, b.y - a.y) < 1e-6) return '顶点重合：起点/终点不能再点击同一位置';
  }
  // Shoelace area (ignore z: footprint is plan-view)
  let area2 = 0;
  for (let i = 0; i < n; i++) {
    const a = points[i];
    const b = points[(i + 1) % n];
    area2 += a.x * b.y - b.x * a.y;
  }
  if (Math.abs(area2) / 2 < 1e-3) return '轮廓面积为零：顶点共线或退化';
  // Self-intersection (proper crossings only, adjacent edges share endpoints)
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      if (Math.abs(i - j) <= 0 || (i === 0 && j === n - 1)) continue;
      if (segmentsProperlyIntersect(points[i], points[(i + 1) % n], points[j], points[(j + 1) % n])) {
        return '轮廓自相交：形如"8 字形"的路径无法生成楼板/屋顶';
      }
    }
  }
  return null;
}

function segmentsProperlyIntersect(
  p1: { x: number; y: number }, p2: { x: number; y: number },
  p3: { x: number; y: number }, p4: { x: number; y: number },
): boolean {
  const d1 = cross(p3, p4, p1);
  const d2 = cross(p3, p4, p2);
  const d3 = cross(p1, p2, p3);
  const d4 = cross(p1, p2, p4);
  return ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0));
}

function cross(o: { x: number; y: number }, a: { x: number; y: number }, b: { x: number; y: number }): number {
  return (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
}
