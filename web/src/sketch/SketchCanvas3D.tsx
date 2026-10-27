import { useRef, useEffect } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import type { Sketch } from '../shared/types';

type Props = {
  sketch: Sketch | null;
  selectedEntityIdx: number | null;
  previewMesh: { positions: number[]; normals: number[]; indices: number[] } | null;
};

export function SketchCanvas3D({ sketch, selectedEntityIdx, previewMesh }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<{
    scene: THREE.Scene;
    camera: THREE.PerspectiveCamera;
    renderer: THREE.WebGLRenderer;
    controls: OrbitControls;
    group: THREE.Group;
    meshGroup: THREE.Group;
    animId: number;
  } | null>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x1a1a2e);

    const w = container.clientWidth;
    const h = container.clientHeight;
    const camera = new THREE.PerspectiveCamera(45, w / h, 0.1, 200);
    camera.position.set(8, -8, 6);
    camera.up.set(0, 0, 1);
    camera.lookAt(0, 0, 0);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(w, h);
    renderer.setPixelRatio(window.devicePixelRatio);
    container.appendChild(renderer.domElement);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    controls.target.set(0, 0, 0);
    controls.mouseButtons = {
      LEFT: THREE.MOUSE.ROTATE,
      MIDDLE: THREE.MOUSE.DOLLY,
      RIGHT: THREE.MOUSE.PAN,
    };

    scene.add(new THREE.AmbientLight(0x404060, 0.6));
    const dir = new THREE.DirectionalLight(0xffffff, 1.2);
    dir.position.set(5, 5, 10);
    scene.add(dir);
    const fill = new THREE.DirectionalLight(0xffffff, 0.4);
    fill.position.set(-5, -5, -5);
    scene.add(fill);

    const grid = new THREE.GridHelper(20, 20, 0x444466, 0x333355);
    grid.rotation.x = Math.PI / 2;
    scene.add(grid);

    const group = new THREE.Group();
    scene.add(group);

    const meshGroup = new THREE.Group();
    scene.add(meshGroup);

    const state = { scene, camera, renderer, controls, group, meshGroup, animId: 0 };
    sceneRef.current = state;

    function animate() {
      state.animId = requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    }
    animate();

    const ro = new ResizeObserver(() => {
      const cw = container.clientWidth;
      const ch = container.clientHeight;
      camera.aspect = cw / ch;
      camera.updateProjectionMatrix();
      renderer.setSize(cw, ch);
    });
    ro.observe(container);

    return () => {
      cancelAnimationFrame(state.animId);
      ro.disconnect();
      renderer.dispose();
      if (renderer.domElement.parentElement) {
        renderer.domElement.parentElement.removeChild(renderer.domElement);
      }
    };
  }, []);

  // ── Update sketch entities ──
  useEffect(() => {
    const state = sceneRef.current;
    if (!state) return;
    const { group } = state;

    while (group.children.length > 0) {
      const child = group.children[0];
      if (child instanceof THREE.BufferGeometry) child.dispose();
      group.remove(child);
    }

    const planeGeo = new THREE.PlaneGeometry(10, 10);
    const planeMat = new THREE.MeshBasicMaterial({
      color: 0x2a2a5e, transparent: true, opacity: 0.3,
      side: THREE.DoubleSide, depthWrite: false,
    });
    const plane = new THREE.Mesh(planeGeo, planeMat);
    plane.rotation.x = -Math.PI / 2;
    group.add(plane);

    const borderGeo = new THREE.EdgesGeometry(planeGeo);
    const borderMat = new THREE.LineBasicMaterial({ color: 0x444488 });
    const border = new THREE.LineSegments(borderGeo, borderMat);
    border.rotation.x = -Math.PI / 2;
    group.add(border);

    const axesMat = (color: number) => new THREE.LineBasicMaterial({ color });
    const origin = new THREE.Vector3(0, 0, 0);
    const axisLen = 2.5;

    const xGeo = new THREE.BufferGeometry().setFromPoints([origin, new THREE.Vector3(axisLen, 0, 0)]);
    group.add(new THREE.Line(xGeo, axesMat(0xff4444)));
    const yGeo = new THREE.BufferGeometry().setFromPoints([origin, new THREE.Vector3(0, axisLen, 0)]);
    group.add(new THREE.Line(yGeo, axesMat(0x44ff44)));
    const zGeo = new THREE.BufferGeometry().setFromPoints([origin, new THREE.Vector3(0, 0, axisLen)]);
    group.add(new THREE.Line(zGeo, axesMat(0x4488ff)));

    if (!sketch) return;

    const isSolved = sketch.solved;

    sketch.entities.forEach((entity, idx) => {
      const isSelected = idx === selectedEntityIdx;
      const color = isSelected ? 0xffaa00 : isSolved ? 0x4fc3f7 : 0x666666;

      switch (entity.entity_type) {
        case 'Line': {
          if (entity.points.length >= 2) {
            const pts = [
              new THREE.Vector3(entity.points[0].x, entity.points[0].y, 0),
              new THREE.Vector3(entity.points[1].x, entity.points[1].y, 0),
            ];
            const geo = new THREE.BufferGeometry().setFromPoints(pts);
            group.add(new THREE.Line(geo, new THREE.LineBasicMaterial({ color })));
            pts.forEach((p) => {
              const dot = new THREE.Mesh(
                new THREE.SphereGeometry(0.06, 8, 8),
                new THREE.MeshBasicMaterial({ color })
              );
              dot.position.copy(p);
              group.add(dot);
            });
          }
          break;
        }
        case 'Circle': {
          if (entity.points.length >= 1) {
            const curve = new THREE.EllipseCurve(entity.points[0].x, entity.points[0].y, entity.radius, entity.radius, 0, Math.PI * 2, false, 0);
            const pts = curve.getPoints(32).map((p) => new THREE.Vector3(p.x, p.y, 0));
            const geo = new THREE.BufferGeometry().setFromPoints(pts);
            group.add(new THREE.Line(geo, new THREE.LineBasicMaterial({ color })));
          }
          break;
        }
        case 'Arc': {
          if (entity.points.length >= 3) {
            const cx = entity.points[0].x, cy = entity.points[0].y;
            const r = entity.radius;
            const sa = Math.atan2(entity.points[1].y - cy, entity.points[1].x - cx);
            const ea = Math.atan2(entity.points[2].y - cy, entity.points[2].x - cx);
            const curve = new THREE.EllipseCurve(cx, cy, r, r, Math.min(sa, ea), Math.max(sa, ea), false, 0);
            const pts = curve.getPoints(16).map((p) => new THREE.Vector3(p.x, p.y, 0));
            const geo = new THREE.BufferGeometry().setFromPoints(pts);
            group.add(new THREE.Line(geo, new THREE.LineBasicMaterial({ color })));
          }
          break;
        }
      }
    });
  }, [sketch, selectedEntityIdx]);

  // ── Update preview mesh ──
  useEffect(() => {
    const state = sceneRef.current;
    if (!state) return;
    const { meshGroup } = state;

    while (meshGroup.children.length > 0) {
      const child = meshGroup.children[0];
      if (child instanceof THREE.Mesh && child.geometry) child.geometry.dispose();
      meshGroup.remove(child);
    }

    if (!previewMesh) return;

    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(previewMesh.positions, 3));
    geo.setAttribute('normal', new THREE.Float32BufferAttribute(previewMesh.normals, 3));
    geo.setIndex(previewMesh.indices);

    const mat = new THREE.MeshStandardMaterial({
      color: 0x4fc3f7,
      roughness: 0.4,
      metalness: 0.2,
      transparent: true,
      opacity: 0.85,
      side: THREE.DoubleSide,
    });
    const mesh = new THREE.Mesh(geo, mat);
    meshGroup.add(mesh);

    // Edge wireframe
    const edges = new THREE.EdgesGeometry(geo);
    const edgeMat = new THREE.LineBasicMaterial({ color: 0x88ddff, transparent: true, opacity: 0.3 });
    meshGroup.add(new THREE.LineSegments(edges, edgeMat));
  }, [previewMesh]);

  return <div ref={containerRef} className="sketch-canvas-3d" />;
}