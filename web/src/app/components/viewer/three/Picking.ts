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

import { t } from '../../../../shared/i18n';

export function pickingHint(objectType: ObjectType, points: Point3D[], mode?: PickMode): string {
  const pm = mode ?? defaultPickModeForType(objectType);
  const count = points.length;
  const min = requiredClicks(pm);
  if (count >= min) {
    // Polygon: closing the loop is the last click; polyline can keep going.
    switch (pm.mode) {
      case 'polygon':
        return t('pick.state.poly_close', { count, min });
      case 'polyline':
        return t('pick.state.poly_done', { count, min });
      case 'segment':
        return t('pick.state.seg_done');
      case 'place':
        return t('pick.state.place');
    }
  }
  if (count > 0) {
    switch (pm.mode) {
      case 'polyline': return t('pick.state.polyline', { count, min });
      case 'polygon': return t('pick.state.polygon', { count, min });
      case 'segment': return t('pick.state.segment', { count, min });
    }
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