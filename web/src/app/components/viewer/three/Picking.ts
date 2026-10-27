import * as THREE from 'three';
import type { ObjectType, PickMode, Point3D } from '../../../../shared/types/creation';
import { defaultPickModeForType } from '../../../../shared/types/creation';
import { PICK_TEXT } from '../../../../shared/creation/pickText';

export type PickingContext = {
  camera: THREE.Camera;
  renderer: THREE.WebGLRenderer;
};

export function requiredClicks(mode: PickMode): number {
  return mode.mode === 'polygon' ? 3 : mode.mode === 'place' ? 1 : 2;
}

export function canComplete(mode: PickMode, count: number): boolean {
  if (mode.mode === 'polygon') return count >= 3;
  if (mode.mode === 'place') return count >= 1;
  return count >= 2;
}

export function pickingHint(objectType: ObjectType, points: Point3D[], mode?: PickMode): string {
  const pm = mode ?? defaultPickModeForType(objectType);
  const count = points.length;
  if (points.length > 0 && points.length >= requiredClicks(pm)) {
    // Polygon: closing the loop is the last click; polyline can keep going.
    return pm.mode === 'polygon'
      ? '单击封闭轮廓完成，双击/回车完成'
      : '继续选点，双击/回车完成';
  }
  const steps = PICK_TEXT[pm.mode] ?? [];
  const label = steps[count] ?? steps[steps.length - 1];
  return label(optionsFor(pm));
}

type HintOptions = { clicks: number };
function optionsFor(mode: PickMode): HintOptions {
  return { clicks: requiredClicks(mode) };
}

export function pickGroundPoint(
  context: PickingContext,
  clientX: number,
  clientY: number
): Point3D | null {
  const rect = context.renderer.domElement.getBoundingClientRect();
  const pointer = new THREE.Vector2();
  pointer.x = ((clientX - rect.left) / rect.width) * 2 - 1;
  pointer.y = -((clientY - rect.top) / rect.height) * 2 + 1;
  const raycaster = new THREE.Raycaster();
  raycaster.setFromCamera(pointer, context.camera);
  const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
  const pt = new THREE.Vector3();
  if (!raycaster.ray.intersectPlane(plane, pt)) return null;
  return { x: pt.x, y: pt.y, z: pt.z };
}