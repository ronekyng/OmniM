import * as THREE from 'three';
import type { Point3D } from '../../../../shared/types/creation';
import type { LayerDefInput } from '../../../../shared/types';

export function clearMarkerGroup(group: THREE.Group): void {
  while (group.children.length) {
    const child = group.children[0];
    group.remove(child);
    if (child instanceof THREE.Line) {
      child.geometry.dispose();
      (child.material as THREE.Material).dispose();
    } else if (child instanceof THREE.Mesh) {
      child.geometry.dispose();
      (child.material as THREE.Material).dispose();
    }
  }
}

export function addPointMarker(group: THREE.Group, point: Point3D, color = 0x44ff88): void {
  const dot = new THREE.Mesh(
    new THREE.SphereGeometry(0.15, 12, 12),
    new THREE.MeshBasicMaterial({ color })
  );
  dot.position.set(point.x, point.y, point.z);
  group.add(dot);
}

export function addPreviewLine(group: THREE.Group, points: Point3D[], color = 0x44ff88): void {
  const linePoints = points.map((p) => new THREE.Vector3(p.x, p.y, p.z));
  const lineGeo = new THREE.BufferGeometry().setFromPoints(linePoints);
  const line = new THREE.Line(
    lineGeo,
    new THREE.LineBasicMaterial({ color, linewidth: 2, transparent: true, opacity: 0.6 })
  );
  group.add(line);
}

/** Add a translucent filled polygon (ground-plane fan) preview for slab/floor/
 *  roof footprint picking. Fan tessellation matches a single-origin polygon. */
export function addPolygonFill(group: THREE.Group, points: Point3D[], color: number | undefined = 0x44ff88): void {
  if (points.length < 3) return;
  const pts = points.map((p) => new THREE.Vector3(p.x, p.y, p.z));
  const positions: number[] = [];
  const origin = pts[0];
  for (let i = 1; i + 1 < pts.length; i++) {
    positions.push(origin.x, origin.y, origin.z + 0.01);
    positions.push(pts[i].x, pts[i].y, pts[i].z + 0.01);
    positions.push(pts[i + 1].x, pts[i + 1].y, pts[i + 1].z + 0.01);
  }
  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3));
  const mat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.25, side: THREE.DoubleSide });
  const mesh = new THREE.Mesh(geo, mat);
  group.add(mesh);
}

/** Add a placement cursor for single-point types (column/tank/pump/...): a
 *  small ground dot surrounded by a crosshair ring at fixed world size. */
export function addPlaceCursor(group: THREE.Group, point: Point3D, color = 0xffcc44): void {
  addPointMarker(group, point, color);
  const size = 0.8;
  const s = size / 2;
  const cross = [
    new THREE.Vector3(point.x - s, point.y, point.z),
    new THREE.Vector3(point.x + s, point.y, point.z),
    new THREE.Vector3(point.x, point.y - s, point.z),
    new THREE.Vector3(point.x, point.y + s, point.z),
  ];
  const crossGeo = new THREE.BufferGeometry().setFromPoints(cross);
  const crossMat = new THREE.LineBasicMaterial({ color, transparent: true, opacity: 0.9 });
  const crossLine = new THREE.Line(crossGeo, crossMat);
  group.add(crossLine);

  const ringGeo = new THREE.RingGeometry(s, s + 0.06, 32);
  const ringMat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.6, side: THREE.DoubleSide });
  const ring = new THREE.Mesh(ringGeo, ringMat);
  ring.position.set(point.x, point.y, point.z + 0.02);
  ring.rotation.x = -Math.PI / 2;
  group.add(ring);
}

/** Create or update a dashed preview line from the last collected point to
 *  the current ground-plane cursor position. Returns the line object so the
 *  caller can remove it on the next frame. */
export function createDashedPreviewLine(
  from: Point3D,
  to: { x: number; y: number; z: number },
): THREE.Line {
  const points = [
    new THREE.Vector3(from.x, from.y, from.z),
    new THREE.Vector3(to.x, to.y, to.z),
  ];
  const geo = new THREE.BufferGeometry().setFromPoints(points);
  const mat = new THREE.LineDashedMaterial({
    color: 0x44ff88,
    dashSize: 0.2,
    gapSize: 0.15,
    transparent: true,
    opacity: 0.5,
  });
  const line = new THREE.Line(geo, mat);
  line.computeLineDistances();
  return line;
}

const PREVIEW_THICKNESS_SCALE = 0.001; // mm → metres (matches server Layered genrule)

/** Per-vertex bend extension (miter allowance) for a polyline preview.
 *  Returns, for each interior vertex i, the amount each adjacent segment end
 *  should be extended along its own direction so layer planes meet plane-to
 *  -plane at the bend (server kernel uses e = (t/2)·tan(φ/2) per layer).
 *  `thickness` is in world units (metres). Interior vertices only; endpoints
 *  get 0. Sharp bends are capped to keep extents from exploding (matches the
 *  kernel cap: min(0.45·shorter adjacent edge, 2·t)). */
export function bendExtensions(points: { x: number; y: number }[], thickness: number): number[] {
  const n = points.length;
  const ext = new Array(n).fill(0);
  for (let i = 1; i + 1 < n; i++) {
    const a = Math.atan2(points[i].y - points[i - 1].y, points[i].x - points[i - 1].x);
    const b = Math.atan2(points[i + 1].y - points[i].y, points[i + 1].x - points[i].x);
    let turn = Math.abs(b - a) % Math.PI;
    if (turn > Math.PI / 2) turn = Math.PI - turn;
    const e = (thickness / 2) * Math.tan(turn / 2);
    const lenPrev = Math.hypot(points[i].x - points[i - 1].x, points[i].y - points[i - 1].y);
    const lenNext = Math.hypot(points[i + 1].x - points[i].x, points[i + 1].y - points[i].y);
    ext[i] = Math.min(e, 0.45 * Math.min(lenPrev, lenNext), 2 * thickness);
  }
  return ext;
}

function layerPreviewMaterial(color: THREE.Color): THREE.MeshStandardMaterial {
  return new THREE.MeshStandardMaterial({
    color,
    transparent: true,
    opacity: 0.55,
    depthWrite: false,
    emissive: color,
    emissiveIntensity: 0.25,
  });
}

/** Build a single layer box segment along `from → to`, extended at its ends by
 *  `extStart`/`extEnd` (bend miter allowance along the segment direction). */
function addLayerSegment(
  group: THREE.Group,
  from: Point3D,
  to: Point3D,
  thicknessWorld: number,
  offsetCenter: number,
  height: number,
  extStart: number,
  extEnd: number,
  color: THREE.Color,
): void {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const length = Math.hypot(dx, dy);
  if (length < 1e-4) return;
  const angle = Math.atan2(dy, dx);
  const run = length + extStart + extEnd;
  const geo = new THREE.BoxGeometry(run, thicknessWorld, height);
  const mat = layerPreviewMaterial(color);
  const mesh = new THREE.Mesh(geo, mat);
  // Slide the centroid along the run by (extEnd - extStart) / 2 and offset by
  // `offsetCenter` along the outward normal (nx, ny) = (-sin, cos).
  const nx = -Math.sin(angle);
  const ny = Math.cos(angle);
  const slide = (extEnd - extStart) / 2;
  const cx = (from.x + to.x) / 2 + nx * offsetCenter + Math.cos(angle) * slide;
  const cy = (from.y + to.y) / 2 + ny * offsetCenter + Math.sin(angle) * slide;
  mesh.position.set(cx, cy, height / 2);
  mesh.rotation.z = angle;
  group.add(mesh);
}

/** Build a per-layer extruded preview for a whole polyline path (multi-point
 *  wall like the server's `build_layered_path`, incl. bend miter). Points are
 *  ground-plane positions in metres; layer thickness in `layers` is mm and is
 *  scaled by 0.001 — the SAME conversion the server applies, so the preview
 *  matches the created geometry. Renders with a soft standard material
 *  (lit, translucent, no depth-write) for a clean "ghost" look. */
export function addLayeredWallPreview(
  group: THREE.Group,
  points: Point3D[],
  layers: LayerDefInput[],
  height = 3,
): THREE.Group | null {
  if (points.length < 2) return null;
  const g = new THREE.Group();
  const layerThk = layers.map((l) => Math.max((l.thickness || 0) * PREVIEW_THICKNESS_SCALE, 0));
  const total = layerThk.reduce((a, b) => a + b, 0);
  if (total <= 0) return null;
  const ext = bendExtensions(points, total);
  let offset = -total / 2;
  for (let li = 0; li < layerThk.length; li++) {
    const t = layerThk[li];
    const color = layers[li].color ? new THREE.Color(layers[li].color as string) : new THREE.Color(0x999999);
    for (let i = 0; i + 1 < points.length; i++) {
      // Extension of this layer at each bend uses the per-layer thickness:
      // e = (t/2)·tan(φ/2), scaled from the total-thickness vertex allowance.
      const startExt = i === 0 ? 0 : ext[i] * (t / total);
      const endExt = i + 2 === points.length ? 0 : ext[i + 1] * (t / total);
      addLayerSegment(g, points[i], points[i + 1], t, offset + t / 2, height, startExt, endExt, color);
    }
    offset += t;
  }
  group.add(g);
  return g;
}

/** Backward-compatible single-segment preview built on the shared layered
 *  builder (correct mm→m scale, lit material, optional end extension). */
export function addLayerPreview(
  group: THREE.Group,
  from: Point3D,
  to: Point3D,
  layers: LayerDefInput[],
  height = 3,
): THREE.Group | null {
  return addLayeredWallPreview(group, [from, to], layers, height);
}