import * as THREE from 'three';
import type { SnapMode } from './Snapping';

const COLORS: Record<SnapMode, number> = {
  vertex: 0x44ff88,
  endpoint: 0x44ff88,
  edge: 0x44aaff,
  midpoint: 0xffaa44,
  grid: 0x888888,
};

/**
 * Manages a visual snap indicator ring in the scene.
 * Creates a single reusable ring mesh that is moved/visibility-toggled
 * rather than allocating new geometry each frame.
 */
export class SnapIndicator {
  private group: THREE.Group;
  private ring: THREE.Line;
  private crosshair: THREE.Line;
  private visible = false;

  constructor(scene: THREE.Scene) {
    this.group = new THREE.Group();
    this.group.visible = false;

    // Ring
    const ringPoints: THREE.Vector3[] = [];
    const segments = 24;
    const radius = 0.3;
    for (let i = 0; i <= segments; i++) {
      const theta = (i / segments) * Math.PI * 2;
      ringPoints.push(new THREE.Vector3(Math.cos(theta) * radius, Math.sin(theta) * radius, 0));
    }
    const ringGeo = new THREE.BufferGeometry().setFromPoints(ringPoints);
    const ringMat = new THREE.LineBasicMaterial({ color: 0x44ff88, linewidth: 2, depthTest: false });
    this.ring = new THREE.Line(ringGeo, ringMat);
    this.group.add(this.ring);

    // Crosshair (+)
    const crossPoints = [
      new THREE.Vector3(-radius * 0.3, 0, 0),
      new THREE.Vector3(radius * 0.3, 0, 0),
      new THREE.Vector3(0, -radius * 0.3, 0),
      new THREE.Vector3(0, radius * 0.3, 0),
    ];
    const crossGeo = new THREE.BufferGeometry().setFromPoints(crossPoints);
    const crossMat = new THREE.LineBasicMaterial({ color: 0xffffff, depthTest: false });
    this.crosshair = new THREE.LineSegments(
      new THREE.WireframeGeometry(crossGeo),
      crossMat,
    );
    this.group.add(this.crosshair);

    scene.add(this.group);
  }

  show(position: THREE.Vector3, mode: SnapMode) {
    this.group.position.copy(position);
    (this.ring.material as THREE.LineBasicMaterial).color.setHex(COLORS[mode] ?? 0x44ff88);
    this.group.visible = true;
    this.visible = true;
  }

  hide() {
    if (this.visible) {
      this.group.visible = false;
      this.visible = false;
    }
  }

  dispose() {
    this.ring.geometry.dispose();
    (this.ring.material as THREE.LineBasicMaterial).dispose();
    (this.crosshair.material as THREE.LineBasicMaterial).dispose();
    this.group.removeFromParent();
  }
}