import * as THREE from 'three';

export type SnapMode = 'vertex' | 'edge' | 'midpoint' | 'endpoint' | 'grid';

export type SnapResult = {
  point: THREE.Vector3;
  snapped: boolean;
  snapType: SnapMode | null;
  distance: number;
};

export type SnapSettings = {
  enabled: boolean;
  toVertex: boolean;
  toEdge: boolean;
  toMidpoint: boolean;
  toEndpoint: boolean;
  toGrid: boolean;
  threshold: number; // world units
  gridSize: number;
};

export const DEFAULT_SNAP_SETTINGS: SnapSettings = {
  enabled: true,
  toVertex: true,
  toEdge: false,
  toMidpoint: false,
  toEndpoint: true,
  toGrid: true,
  threshold: 0.5,
  gridSize: 1.0,
};

/**
 * Snap a 3D point using the configured snap modes.
 * Priority: grid < vertex/endpoint < midpoint < edge
 * Returns the original point unsnapped if no snap target is found within threshold.
 */
export function snapPoint(
  point: THREE.Vector3,
  meshes: Map<string, THREE.Mesh>,
  settings: SnapSettings,
): SnapResult {
  if (!settings.enabled) {
    return { point: point.clone(), snapped: false, snapType: null, distance: 0 };
  }

  let best: SnapResult = { point: point.clone(), snapped: false, snapType: null, distance: Infinity };
  const threshold = settings.threshold;

  // 1. Grid snap (lowest priority, only if no other snap found)
  if (settings.toGrid) {
    const gridSnapped = snapToGrid(point, settings.gridSize);
    const dist = point.distanceTo(gridSnapped);
    if (dist < threshold && dist < best.distance) {
      best = { point: gridSnapped, snapped: true, snapType: 'grid', distance: dist };
    }
  }

  // 2. Vertex / Endpoint snap
  if (settings.toVertex || settings.toEndpoint) {
    for (const [, mesh] of meshes) {
      const geo = mesh.geometry;
      if (!geo) continue;
      const pos = geo.getAttribute('position');
      if (!pos) continue;
      const worldPos = new THREE.Vector3();
      const localPoint = point.clone().sub(mesh.position);
      for (let i = 0; i < pos.count; i++) {
        worldPos.set(pos.getX(i), pos.getY(i), pos.getZ(i));
        worldPos.add(mesh.position);
        const dist = point.distanceTo(worldPos);
        if (dist < threshold && dist < best.distance) {
          best = { point: worldPos.clone(), snapped: true, snapType: 'vertex', distance: dist };
        }
      }
    }
  }

  // 3. Edge snap (project point onto nearest edge segment)
  if (settings.toEdge) {
    for (const [, mesh] of meshes) {
      const geo = mesh.geometry;
      if (!geo) continue;
      const pos = geo.getAttribute('position');
      if (!pos) continue;
      const idx = geo.index;
      if (idx) {
        // Indexed geometry: use triangles
        for (let i = 0; i < idx.count; i += 3) {
          const a = new THREE.Vector3(pos.getX(idx.getX(i)), pos.getY(idx.getX(i)), pos.getZ(idx.getX(i))).add(mesh.position);
          const b = new THREE.Vector3(pos.getX(idx.getX(i + 1)), pos.getY(idx.getX(i + 1)), pos.getZ(idx.getX(i + 1))).add(mesh.position);
          const c = new THREE.Vector3(pos.getX(idx.getX(i + 2)), pos.getY(idx.getX(i + 2)), pos.getZ(idx.getX(i + 2))).add(mesh.position);
          checkEdge(a, b, point, threshold, best);
          checkEdge(b, c, point, threshold, best);
          checkEdge(c, a, point, threshold, best);
        }
      } else {
        // Non-indexed: treat every 3 vertices as a triangle
        for (let i = 0; i < pos.count; i += 3) {
          const a = new THREE.Vector3(pos.getX(i), pos.getY(i), pos.getZ(i)).add(mesh.position);
          const b = new THREE.Vector3(pos.getX(i + 1), pos.getY(i + 1), pos.getZ(i + 1)).add(mesh.position);
          const c = new THREE.Vector3(pos.getX(i + 2), pos.getY(i + 2), pos.getZ(i + 2)).add(mesh.position);
          checkEdge(a, b, point, threshold, best);
          checkEdge(b, c, point, threshold, best);
          checkEdge(c, a, point, threshold, best);
        }
      }
    }
  }

  // 4. Midpoint snap
  if (settings.toMidpoint) {
    for (const [, mesh] of meshes) {
      const geo = mesh.geometry;
      if (!geo) continue;
      const pos = geo.getAttribute('position');
      if (!pos) continue;
      const idx = geo.index;
      if (idx) {
        for (let i = 0; i < idx.count; i += 3) {
          const a = new THREE.Vector3(pos.getX(idx.getX(i)), pos.getY(idx.getX(i)), pos.getZ(idx.getX(i))).add(mesh.position);
          const b = new THREE.Vector3(pos.getX(idx.getX(i + 1)), pos.getY(idx.getX(i + 1)), pos.getZ(idx.getX(i + 1))).add(mesh.position);
          const c = new THREE.Vector3(pos.getX(idx.getX(i + 2)), pos.getY(idx.getX(i + 2)), pos.getZ(idx.getX(i + 2))).add(mesh.position);
          checkMidpoint(a, b, point, threshold, best);
          checkMidpoint(b, c, point, threshold, best);
          checkMidpoint(c, a, point, threshold, best);
        }
      } else {
        for (let i = 0; i < pos.count; i += 3) {
          const a = new THREE.Vector3(pos.getX(i), pos.getY(i), pos.getZ(i)).add(mesh.position);
          const b = new THREE.Vector3(pos.getX(i + 1), pos.getY(i + 1), pos.getZ(i + 1)).add(mesh.position);
          const c = new THREE.Vector3(pos.getX(i + 2), pos.getY(i + 2), pos.getZ(i + 2)).add(mesh.position);
          checkMidpoint(a, b, point, threshold, best);
          checkMidpoint(b, c, point, threshold, best);
          checkMidpoint(c, a, point, threshold, best);
        }
      }
    }
  }

  return best;
}

function checkEdge(a: THREE.Vector3, b: THREE.Vector3, point: THREE.Vector3, threshold: number, best: SnapResult) {
  const closest = closestPointOnSegment(a, b, point);
  const dist = point.distanceTo(closest);
  if (dist < threshold && dist < best.distance) {
    best.point.copy(closest);
    best.snapped = true;
    best.snapType = 'edge';
    best.distance = dist;
  }
}

function checkMidpoint(a: THREE.Vector3, b: THREE.Vector3, point: THREE.Vector3, threshold: number, best: SnapResult) {
  const mid = new THREE.Vector3().addVectors(a, b).multiplyScalar(0.5);
  const dist = point.distanceTo(mid);
  if (dist < threshold && dist < best.distance) {
    best.point.copy(mid);
    best.snapped = true;
    best.snapType = 'midpoint';
    best.distance = dist;
  }
}

function closestPointOnSegment(a: THREE.Vector3, b: THREE.Vector3, p: THREE.Vector3): THREE.Vector3 {
  const ab = new THREE.Vector3().copy(b).sub(a);
  const ap = new THREE.Vector3().copy(p).sub(a);
  const t = Math.max(0, Math.min(1, ap.dot(ab) / ab.dot(ab)));
  return new THREE.Vector3().copy(a).add(ab.multiplyScalar(t));
}

export function snapToGrid(point: THREE.Vector3, gridSize: number): THREE.Vector3 {
  return new THREE.Vector3(
    Math.round(point.x / gridSize) * gridSize,
    Math.round(point.y / gridSize) * gridSize,
    Math.round(point.z / gridSize) * gridSize,
  );
}