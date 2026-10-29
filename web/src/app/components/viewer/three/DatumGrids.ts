import * as THREE from 'three';
import type { GridLine } from '../../../../shared/types';

/** Datum grid rendering (P8-S3): axis lines + tag sprites at the extents.
 *  Grids are datum annotations (no mesh), so they live in their own group and
 *  never take part in picking/mesh sync. */

/** Default half-extent when a line carries no explicit extent (0/0). */
export const DEFAULT_GRID_HALF_EXTENT = 20;

export type GridEndpoints = {
  /** World-space segment endpoints (metres). */
  from: { x: number; y: number; z: number };
  to: { x: number; y: number; z: number };
};

/** Pure endpoint maths for one grid line (unit-testable).
 *  `AlongX` runs parallel to X at `y = coordinate` (numbered family);
 *  `AlongY` runs parallel to Y at `x = coordinate` (lettered family). */
export function gridLineEndpoints(line: GridLine, z = 0): GridEndpoints {
  const hasExtent = Math.abs(line.extent_max - line.extent_min) > 1e-9;
  const min = hasExtent
    ? Math.min(line.extent_min, line.extent_max)
    : -DEFAULT_GRID_HALF_EXTENT;
  const max = hasExtent
    ? Math.max(line.extent_min, line.extent_max)
    : DEFAULT_GRID_HALF_EXTENT;
  if (line.axis === 'AlongX') {
    return {
      from: { x: min, y: line.coordinate, z },
      to: { x: max, y: line.coordinate, z },
    };
  }
  return {
    from: { x: line.coordinate, y: min, z },
    to: { x: line.coordinate, y: max, z },
  };
}

const COLORS: Record<GridLine['axis'], number> = {
  AlongX: 0xff9f43, // numbered family (1, 2, 3…)
  AlongY: 0x54a0ff, // lettered family (A, B, C…)
};

/** Canvas-texture sprite for the axis tag. */
function makeTagSprite(text: string, color: number): THREE.Sprite {
  const canvas = document.createElement('canvas');
  canvas.width = 128;
  canvas.height = 64;
  const ctx = canvas.getContext('2d')!;
  ctx.fillStyle = 'rgba(20,26,34,0.82)';
  ctx.beginPath();
  ctx.roundRect(0, 0, 128, 64, 12);
  ctx.fill();
  ctx.fillStyle = `#${color.toString(16).padStart(6, '0')}`;
  ctx.font = 'bold 40px sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 64, 34);
  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  const material = new THREE.SpriteMaterial({ map: texture, depthWrite: false });
  const sprite = new THREE.Sprite(material);
  sprite.scale.set(0.9, 0.45, 1);
  return sprite;
}

export function clearDatumGridGroup(group: THREE.Group): void {
  while (group.children.length) {
    const child = group.children[0];
    group.remove(child);
    if (child instanceof THREE.Line) {
      child.geometry.dispose();
      (child.material as THREE.Material).dispose();
    } else if (child instanceof THREE.Sprite) {
      (child.material as THREE.SpriteMaterial).map?.dispose();
      (child.material as THREE.Material).dispose();
    }
  }
}

/** Rebuild the datum grid group from API data. */
export function renderDatumGrids(group: THREE.Group, lines: GridLine[]): void {
  clearDatumGridGroup(group);
  for (const line of lines) {
    const { from, to } = gridLineEndpoints(line);
    const geometry = new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(from.x, from.y, from.z),
      new THREE.Vector3(to.x, to.y, to.z),
    ]);
    const material = new THREE.LineBasicMaterial({
      color: COLORS[line.axis],
      transparent: true,
      opacity: 0.75,
    });
    group.add(new THREE.Line(geometry, material));

    const tag = makeTagSprite(line.label, COLORS[line.axis]);
    tag.position.set(to.x, to.y, to.z + 0.9);
    group.add(tag);
  }
}
