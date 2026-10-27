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
export function addPolygonFill(group: THREE.Group, points: Point3D[], color = 0x44ff88): void {
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

/** Build a per-layer extruded box preview between two picked ground points.
 *  Each layer is a distinct colored box offset along the wall normal so the
 *  section structure is visible before creation (dev plan §2.3). */
export function addLayerPreview(
  group: THREE.Group,
  from: Point3D,
  to: Point3D,
  layers: LayerDefInput[],
  height = 3,
): THREE.Group {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const length = Math.max(Math.sqrt(dx * dx + dy * dy), 0.1);
  const angle = Math.atan2(dy, dx);
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const cx = (from.x + to.x) / 2;
  const cy = (from.y + to.y) / 2;

  const total = layers.reduce((s, l) => s + (l.thickness || 0), 0) || 1;
  const unit = 0.02; // scale factor mm → world units for preview (20mm per unit)
  const half = (total * unit) / 2;
  let offset = -half;

  const merged = new THREE.Group();
  for (const layer of layers) {
    const t = (layer.thickness || 0) * unit;
    const geo = new THREE.BoxGeometry(length, t, height);
    const color = layer.color ? new THREE.Color(layer.color) : new THREE.Color(0x999999);
    const mat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.85, side: THREE.DoubleSide });
    const mesh = new THREE.Mesh(geo, mat);
    const localY = offset + t / 2;
    // Rotate so the layer stack is perpendicular to the wall run.
    const worldX = cx + localY * sin;
    const worldY = cy - localY * cos;
    mesh.position.set(worldX, worldY, height / 2);
    mesh.rotation.z = -angle;
    merged.add(mesh);
    offset += t;
  }
  group.add(merged);
  return merged;
}
