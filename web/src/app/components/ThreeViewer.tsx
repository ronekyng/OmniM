import { forwardRef, useEffect, useImperativeHandle, useRef } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { TransformControls } from 'three/examples/jsm/controls/TransformControls.js';
import { RoomEnvironment } from 'three/examples/jsm/environments/RoomEnvironment.js';
import { ViewCubeGizmo, ObjectPosition } from '@mlightcad/three-viewcube';
import { api } from '../../shared/api';
import type { MeshObject, Measurement } from '../../shared/types';
import type { TransformMode } from './MainToolbar';
import { useAppStore } from '../store/appStore';
import type { ObjectType, Point3D, CreationForm } from '../../shared/types/creation';
import { createInitialForm, applyPointsToForm, pickModeForForm, defaultPickModeForType } from '../../shared/types/creation';
import { LAYERED_TYPES } from '../../shared/catalog';
import { clearMarkerGroup, addPointMarker, addPreviewLine, createDashedPreviewLine, addLayerPreview, addPolygonFill, addPlaceCursor } from './viewer/three/Markers';
import { pickGroundPoint, pickingHint, requiredClicks } from './viewer/three/Picking';
import { SnapIndicator } from './viewer/three/SnapIndicator';
import { snapPoint, snapToGrid, type SnapSettings } from './viewer/three/Snapping';
import viewerStyles from './ThreeViewer.module.css';

export type VisualMode = 'shaded' | 'wireframe' | 'hidden-line' | 'realistic' | 'x-ray';

export type MeasureMode = 'none' | 'distance' | 'angle' | 'area';

export type SectionState = {
  enabled: boolean;
  axis: 'x' | 'y' | 'z';
  value: number;
};

export type ViewName = 'top' | 'bottom' | 'front' | 'back' | 'left' | 'right' | 'iso'
  | 'front-left-top' | 'front-right-top' | 'front-left-bottom' | 'front-right-bottom'
  | 'back-left-top' | 'back-right-top' | 'back-left-bottom' | 'back-right-bottom'
  | 'front-top' | 'front-bottom' | 'back-top' | 'back-bottom'
  | 'front-left' | 'front-right' | 'back-left' | 'back-right'
  | 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';

export type MeshMaterial = {
  color: string;
  roughness: number;
  metalness: number;
};

export type ThreeViewerHandle = {
  fit: () => void;
  home: () => void;
  setView: (view: ViewName) => void;
  getView: () => ViewName | undefined;
  getCameraDirection: () => THREE.Vector3 | undefined;
  rotateView: (dx: number, dy: number) => void;
  getMeshInfo: (id: string) => MeshInfo | undefined;
  getMeshMaterial: (id: string) => MeshMaterial | undefined;
  setMeshMaterial: (id: string, mat: MeshMaterial) => void;
  focusObject: (id: string) => void;
  toggleGrid: () => boolean;
  toggleOrtho: () => boolean;
  setTransformSpace: (space: 'world' | 'local') => void;
  getTransformSpace: () => 'world' | 'local';
  clearMeasurements: () => void;
  finishMeasurement: () => void;
  clearPickingMarkers: () => void;
};

export type MeshInfo = {
  id: string;
  name: string;
  type: string;
  vcount: number;
  tcount: number;
  bbox: [[number, number, number], [number, number, number]];
};

type ThreeViewerProps = {
  selectedIds: readonly string[];
  reloadToken: number;
  transformMode: TransformMode;
  visualMode: VisualMode;
  section: SectionState;
  measureMode: MeasureMode;
  onSelectionChanged?: (ids: string[]) => void;
  onChanged?: () => void;
};

type ViewerState = {
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  controls: OrbitControls;
  transformControls: TransformControls;
  group: THREE.Group;
  measureGroup: THREE.Group;
  meshes: Map<string, THREE.Mesh>;
  edgeHelpers: Map<string, THREE.LineSegments>;
  sectionPlaneMesh?: THREE.Mesh;
  currentView: ViewName;
  grid: THREE.GridHelper;
  isOrtho: boolean;
  sunLight: THREE.DirectionalLight;
  snapIndicator: SnapIndicator;
};

export const ThreeViewer = forwardRef<ThreeViewerHandle, ThreeViewerProps>(function ThreeViewer({ selectedIds, reloadToken, transformMode, visualMode, section, measureMode, onSelectionChanged, onChanged }, ref) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const stateRef = useRef<ViewerState | null>(null);
  const selectedIdsRef = useRef<readonly string[]>([]);
  const draggingTransformRef = useRef(false);
  const onSelectionChangedRef = useRef(onSelectionChanged);
  const onChangedRef = useRef(onChanged);
  const cameraDirRef = useRef(new THREE.Vector3(0, 0, 1));
  const measurePointsRef = useRef<THREE.Vector3[]>([]);
  const measureModeRef = useRef<MeasureMode>('none');
  const pickingMarkerGroupRef = useRef<THREE.Group>(new THREE.Group());
  const previewLineRef = useRef<THREE.Line | null>(null);
  const layerPreviewRef = useRef<THREE.Group | null>(null);
  const creationPhaseRef = useRef<string>('idle');
  const creationObjectTypeRef = useRef<ObjectType | null>(null);
  const creationPointsRef = useRef<Point3D[]>([]);
  const creationFormRef = useRef<CreationForm | null>(null);
  const creationDispatchRef = useRef(useAppStore.getState().creationDispatch);
  onSelectionChangedRef.current = onSelectionChanged;
  onChangedRef.current = onChanged;
  measureModeRef.current = measureMode;
  creationDispatchRef.current = useAppStore.getState().creationDispatch;

  const snapSettings = useAppStore((s) => s.snapSettings);
  const snapSettingsRef = useRef<SnapSettings>(snapSettings);
  snapSettingsRef.current = snapSettings;

  /** Finish the picking session with the given collected points, advancing
   *  the state machine to 'configuring'. */
  function completePickingWith(points: Point3D[]) {
    const form = creationFormRef.current ?? createInitialForm(creationObjectTypeRef.current ?? 'wall');
    const completed = points.length >= 1 ? applyPointsToForm(form, points) : form;
    creationDispatchRef.current({ type: 'completePicking', form: completed });
  }

  const creationPhase = useAppStore((s) => s.creationPhase);
  const creationObjectType = useAppStore((s) => s.creationObjectType);
  const creationPoints = useAppStore((s) => s.creationPoints);
  const creationForm = useAppStore((s) => s.creationForm);
  creationPhaseRef.current = creationPhase;
  creationObjectTypeRef.current = creationObjectType;
  creationPointsRef.current = creationPoints;
  creationFormRef.current = creationForm;

  // When a new picking session starts (creationPhase becomes 'picking'), clear
  // any leftover preview markers from a previous session so they don't linger.
  const prevCreationPhaseRef = useRef<string>('idle');
  useEffect(() => {
    if (creationPhase === 'picking' && creationPhase !== prevCreationPhaseRef.current) {
      clearMarkerGroup(pickingMarkerGroupRef.current);
      removePreviewLine(previewLineRef, pickingMarkerGroupRef);
      layerPreviewRef.current = null;
    }
    if (creationPhase === 'idle') {
      clearMarkerGroup(pickingMarkerGroupRef.current);
      removePreviewLine(previewLineRef, pickingMarkerGroupRef);
      layerPreviewRef.current = null;
    }
    prevCreationPhaseRef.current = creationPhase;
  }, [creationPhase]);

  // When enough points are collected during picking, the preview markers
  // are drawn by the click handler. The user stays in 'picking' until they
  // press "创建" in the Properties panel.

  useImperativeHandle(ref, () => ({
    fit: () => stateRef.current && fitCamera(stateRef.current),
    home: () => stateRef.current && homeCamera(stateRef.current),
    setView: (view) => stateRef.current && setView(stateRef.current, view),
    getView: () => stateRef.current?.currentView,
    getCameraDirection: () => cameraDirRef.current.clone(),
    rotateView: (dx, dy) => stateRef.current && rotateView(stateRef.current, dx, dy),
    getMeshInfo: (id) => {
      const s = stateRef.current;
      if (!s) return undefined;
      const mesh = s.meshes.get(id);
      if (!mesh) return undefined;
      return mesh.userData as MeshInfo;
    },
focusObject: (id) => {
      const s = stateRef.current;
      if (!s) return;
      const mesh = s.meshes.get(id);
      if (!mesh) return;
      const box = new THREE.Box3().setFromObject(mesh);
      const center = box.getCenter(new THREE.Vector3());
      const size = box.getSize(new THREE.Vector3());
      const dist = size.length() * 1.5;
      const dir = s.camera.position.clone().sub(s.controls.target).normalize();
      s.controls.target.copy(center);
      s.camera.position.copy(center.clone().add(dir.multiplyScalar(Math.max(dist, 0.5))));
      s.controls.update();
    },
    getMeshMaterial: (id) => {
      const s = stateRef.current;
      if (!s) return undefined;
      const mesh = s.meshes.get(id);
      if (!mesh) return undefined;
      const mat = mesh.material as THREE.MeshStandardMaterial;
      return {
        color: '#' + mat.color.getHexString(),
        roughness: mat.roughness,
        metalness: mat.metalness,
      };
    },
    setMeshMaterial: (id, mat) => {
      const s = stateRef.current;
      if (!s) return;
      const mesh = s.meshes.get(id);
      if (!mesh) return;
      const m = mesh.material as THREE.MeshStandardMaterial;
      m.color.set(mat.color);
      m.roughness = mat.roughness;
      m.metalness = mat.metalness;
      m.needsUpdate = true;
    },
    toggleGrid: () => {
      const s = stateRef.current;
      if (!s) return false;
      s.grid.visible = !s.grid.visible;
      return s.grid.visible;
    },
    toggleOrtho: () => {
      const s = stateRef.current;
      if (!s) return false;
      s.isOrtho = !s.isOrtho;
      return s.isOrtho;
    },
    setTransformSpace: (space) => {
      const s = stateRef.current;
      if (!s) return;
      s.transformControls.setSpace(space);
    },
    getTransformSpace: () => {
      const s = stateRef.current;
      if (!s) return 'world';
      return s.transformControls.space as 'world' | 'local';
    },
    clearMeasurements: () => {
      measurePointsRef.current = [];
      clearMeasurePreviewObjects();
      const s = stateRef.current;
      if (s) clearMeasureGroup(s);
    },
    clearPickingMarkers: () => {
      clearMarkerGroup(pickingMarkerGroupRef.current);
    },
    finishMeasurement: () => {
      const s = stateRef.current;
      if (!s || measureModeRef.current === 'none') return;
      const points = measurePointsRef.current;
      if (points.length < 2) return;
      if (measureModeRef.current === 'distance' && points.length >= 2) {
        drawDistanceMeasurement(s, points[0], points[1]);
        measurePointsRef.current = [];
      } else if (measureModeRef.current === 'angle' && points.length >= 3) {
        drawAngleMeasurement(s, points[0], points[1], points[2]);
        measurePointsRef.current = [];
      } else if (measureModeRef.current === 'area' && points.length >= 3) {
        drawAreaMeasurement(s, points);
        measurePointsRef.current = [];
      }
      clearMeasurePreviewObjects();
    },
  }), []);

  useEffect(() => { selectedIdsRef.current = selectedIds; }, [selectedIds]);

  // Set crosshair cursor while in picking phase
  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    const canvas = host.querySelector('canvas');
    if (!canvas) return;
    canvas.style.cursor = creationPhase === 'picking' ? 'crosshair' : '';
  }, [creationPhase]);

  useEffect(() => {
    const hostEl = hostRef.current;
    if (!hostEl) return;
    if (stateRef.current) return;
    const host: HTMLDivElement = hostEl;

    const scene = new THREE.Scene();
    scene.background = createSkyGradient();
    scene.fog = new THREE.Fog(skyHorizon, 100, 600);

    // Add gradient sky dome
    const skyGeo = new THREE.SphereGeometry(800, 32, 32);
    const canvas = document.createElement('canvas');
    canvas.width = 1;
    canvas.height = 64;
    const ctx = canvas.getContext('2d')!;
    const grad = ctx.createLinearGradient(0, 0, 0, 64);
    grad.addColorStop(0.0, '#5b9bd5');
    grad.addColorStop(0.35, '#89c4e1');
    grad.addColorStop(0.6, '#b8d8e8');
    grad.addColorStop(0.85, '#d4e6f0');
    grad.addColorStop(1.0, '#dceef6');
    ctx.fillStyle = grad;
    ctx.fillRect(0, 0, 1, 64);
    const skyTex = new THREE.CanvasTexture(canvas);
    skyTex.magFilter = THREE.LinearFilter;
    skyTex.minFilter = THREE.LinearMipMapLinearFilter;
    const skyMat = new THREE.MeshBasicMaterial({ map: skyTex, side: THREE.BackSide });
    const sky = new THREE.Mesh(skyGeo, skyMat);
    scene.add(sky);

    const camera = new THREE.PerspectiveCamera(50, host.clientWidth / Math.max(host.clientHeight, 1), 0.1, 2000);
    camera.up.set(0, 0, 1);
    camera.position.set(15, 20, 12);

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(host.clientWidth, host.clientHeight);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.2;
    host.appendChild(renderer.domElement);

    // Environment map for realistic reflections
    const pmremGenerator = new THREE.PMREMGenerator(renderer);
    const envTexture = pmremGenerator.fromScene(new RoomEnvironment(), 0.04).texture;
    scene.environment = envTexture;
    pmremGenerator.dispose();

    // ── ViewCube Gizmo ──
    let _clickTimer: ReturnType<typeof setTimeout> | null = null;
    const viewCubeGizmo = new ViewCubeGizmo(camera, renderer, {
      pos: ObjectPosition.RIGHT_TOP,
      dimension: 156,
      faceColor: 0x1a1d23,
      hoverColor: 0x4a90d9,
      outlineColor: 0x2a2d35,
    });
    viewCubeGizmo.addEventListener('change', (event) => {
      const navigate = () => {
        const dir = new THREE.Vector3(0, 0, 1).applyQuaternion(event.quaternion).normalize();
        const target = getViewTarget(viewer);
        const distance = getViewDistance(viewer);
        viewer.camera.position.copy(target).add(dir.multiplyScalar(distance));
        viewer.controls.target.copy(target);
        viewer.controls.update();
        viewer.currentView = getOrthoView(dir);
      };
      if (_clickTimer) {
        clearTimeout(_clickTimer);
        _clickTimer = null;
        setView(viewer, 'iso');
        return;
      }
      _clickTimer = setTimeout(() => {
        _clickTimer = null;
        navigate();
      }, 250);
    });

    const _faceLabels = ['TOP', 'LEFT', 'BOTTOM', 'RIGHT', 'FRONT', 'BACK'];
    const facesGroup = (viewCubeGizmo as any).cube.children[0];
    if (facesGroup?.children) {
      for (let i = 0; i < 6; i++) {
        const mesh = facesGroup.children[i] as THREE.Mesh;
        if (!mesh.isMesh) continue;
        const mat = mesh.material as THREE.MeshBasicMaterial;
        if (mat.map) { mat.map = null; mat.needsUpdate = true; }
        const canvas = document.createElement('canvas');
        canvas.width = 128;
        canvas.height = 128;
        const ctx = canvas.getContext('2d')!;
        ctx.clearRect(0, 0, 128, 128);
        ctx.font = 'bold 40px sans-serif';
        ctx.fillStyle = '#e0e0e0';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(_faceLabels[i], 64, 64);
        const tex = new THREE.CanvasTexture(canvas);
        tex.minFilter = THREE.LinearFilter;
        const textMat = new THREE.MeshBasicMaterial({
          map: tex, transparent: true, depthWrite: false, side: THREE.DoubleSide,
        });
        const textMesh = new THREE.Mesh(new THREE.PlaneGeometry(0.85, 0.85), textMat);
        textMesh.position.set(0, 0, 0.01);
        const textRotations = [0, Math.PI / 2, Math.PI, -Math.PI / 2, Math.PI, 0];
        textMesh.rotation.z = textRotations[i];
        textMesh.renderOrder = 1;
        mesh.add(textMesh);
      }
    }

    const _faceColor = 0x1a1d23;
    const _hoverColor = 0x4a90d9;
    (viewCubeGizmo as any).checkSideOver = function (this: any, x: number, y: number) {
      const raycaster = new THREE.Raycaster();
      raycaster.setFromCamera(new THREE.Vector2(x, y), this.gizmoCamera);
      const intersects = raycaster.intersectObjects(this.cube.children, true);
      this.cube.traverse((obj: any) => {
        if (obj.name && obj.material) {
          obj.material.color.setHex(_faceColor);
        }
      });
      if (intersects.length) {
        let hoverName = '';
        for (const hit of intersects) {
          if (hit.object.name) { hoverName = hit.object.name; break; }
        }
        if (hoverName) {
          this.cube.traverse((obj: any) => {
            if (obj.name === hoverName && obj.material) {
              obj.material.color.setHex(_hoverColor);
            }
          });
        }
      }
    };

    const onDblClick = (event: MouseEvent) => {
      const dim = 156;
      const rect = renderer.domElement.getBoundingClientRect();
      const cx = event.clientX - rect.left;
      const cy = event.clientY - rect.top;
      if (cx >= rect.width - dim && cy <= dim) {
        setView(viewer, 'iso');
      }
    };
    renderer.domElement.addEventListener('dblclick', onDblClick);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.mouseButtons = { LEFT: THREE.MOUSE.PAN, MIDDLE: THREE.MOUSE.DOLLY, RIGHT: THREE.MOUSE.ROTATE };
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.target.set(0, 0, 3);
    controls.update();

    const transformControls = new TransformControls(camera, renderer.domElement);
    scene.add(transformControls.getHelper());
    const group = new THREE.Group();
    const meshes = new Map<string, THREE.Mesh>();

    scene.add(new THREE.AmbientLight(0x8899bb, 0.8));
    scene.add(new THREE.HemisphereLight(0x87ceeb, 0xd4e6f0, 1.2));
    const sun = new THREE.DirectionalLight(0xfff5e8, 3);
    sun.position.set(20, 30, 10);
    sun.castShadow = true;
    sun.shadow.mapSize.set(2048, 2048);
    sun.shadow.camera.near = 0.5;
    sun.shadow.camera.far = 200;
    sun.shadow.camera.left = -40;
    sun.shadow.camera.right = 40;
    sun.shadow.camera.top = 40;
    sun.shadow.camera.bottom = -40;
    sun.shadow.camera.updateProjectionMatrix();
    sun.shadow.bias = -0.001;
    scene.add(sun);
    scene.add(sun.target);
    const fill1 = new THREE.DirectionalLight(0x8090c0, 1.2);
    fill1.position.set(-10, 5, -5);
    scene.add(fill1);
    const fill2 = new THREE.DirectionalLight(0x6080ff, 0.8);
    fill2.position.set(0, 2, -15);
    scene.add(fill2);
    const grid = new THREE.GridHelper(80, 40, 0x8899aa, 0x667788);
    grid.rotation.x = Math.PI / 2;
    grid.position.z = -0.01;
    scene.add(grid);

    const measureGroup = new THREE.Group();
    scene.add(measureGroup);
    const pickingMarkerGroup = new THREE.Group();
    scene.add(pickingMarkerGroup);
    pickingMarkerGroupRef.current = pickingMarkerGroup;
    const snapIndicator = new SnapIndicator(scene);
    const viewer: ViewerState = { scene, camera, renderer, controls, transformControls, group, measureGroup, meshes, edgeHelpers: new Map(), currentView: 'iso', grid, isOrtho: false, sunLight: sun, snapIndicator };
    stateRef.current = viewer;

    transformControls.addEventListener('dragging-changed', (event) => {
      draggingTransformRef.current = Boolean(event.value);
      controls.enabled = !event.value;
      if (!event.value && selectedIdsRef.current.length === 1) {
        void persistSelectedTransform(viewer, selectedIdsRef.current[0]);
      }
    });

    const ground = new THREE.Mesh(new THREE.PlaneGeometry(120, 120), new THREE.MeshStandardMaterial({ color: 0xd4dce4, roughness: 0.95, metalness: 0.0 }));
    ground.position.z = -0.02;
    ground.receiveShadow = true;
    scene.add(ground);
    scene.add(group);

    const raycaster = new THREE.Raycaster();
    const pointer = new THREE.Vector2();

    function resize() {
      const w = host.clientWidth;
      const h = Math.max(host.clientHeight, 1);
      const aspect = w / h;
      camera.aspect = aspect;
      if (viewer.isOrtho) {
        const dist = Math.max(camera.position.distanceTo(controls.target), 0.01);
        const halfH = dist * Math.tan((camera.fov * Math.PI) / 360) / Math.max(camera.zoom, 0.01);
        const halfW = halfH * aspect;
        camera.projectionMatrix.makeOrthographic(-halfW, halfW, halfH, -halfH, 0.1, 2000);
        camera.projectionMatrixInverse.copy(camera.projectionMatrix).invert();
      } else {
        camera.updateProjectionMatrix();
      }
      renderer.setSize(w, h);
    }

    // ── Overlay-based box selection ──
    // On Ctrl+pointerdown, a transparent overlay div captures all subsequent events,
    // fully isolating box selection from OrbitControls.

    function getObjectsInRect(viewer: ViewerState, x1: number, y1: number, x2: number, y2: number): string[] {
      const domRect = renderer.domElement.getBoundingClientRect();
      const w = domRect.width;
      const h = domRect.height;

      const ndcLeft = (Math.min(x1, x2) - domRect.left) / w * 2 - 1;
      const ndcRight = (Math.max(x1, x2) - domRect.left) / w * 2 - 1;
      const ndcTop = -((Math.min(y1, y2) - domRect.top) / h) * 2 + 1;
      const ndcBottom = -((Math.max(y1, y2) - domRect.top) / h) * 2 + 1;

      const selectedIds: string[] = [];
      for (const [id, mesh] of viewer.meshes) {
        const aabb = new THREE.Box3().setFromObject(mesh);
        const corners = [
          new THREE.Vector3(aabb.min.x, aabb.min.y, aabb.min.z),
          new THREE.Vector3(aabb.min.x, aabb.min.y, aabb.max.z),
          new THREE.Vector3(aabb.min.x, aabb.max.y, aabb.min.z),
          new THREE.Vector3(aabb.min.x, aabb.max.y, aabb.max.z),
          new THREE.Vector3(aabb.max.x, aabb.min.y, aabb.min.z),
          new THREE.Vector3(aabb.max.x, aabb.min.y, aabb.max.z),
          new THREE.Vector3(aabb.max.x, aabb.max.y, aabb.min.z),
          new THREE.Vector3(aabb.max.x, aabb.max.y, aabb.max.z),
        ];
        for (const corner of corners) {
          const projected = corner.project(camera);
          if (projected.x >= ndcLeft && projected.x <= ndcRight &&
              projected.y >= ndcBottom && projected.y <= ndcTop &&
              projected.z >= -1 && projected.z <= 1) {
            selectedIds.push(id);
            break;
          }
        }
      }
      return selectedIds;
    }

    function startBoxSelection(event: PointerEvent) {
      // Prevent OrbitControls from getting subsequent events by placing an overlay.
      // The overlay captures the pointer, so OrbitControls receives a pointercancel
      // and properly resets its internal state.
      const startX = event.clientX;
      const startY = event.clientY;
      let isDragging = false;
      let selectionRect: HTMLDivElement | null = null;

      const overlay = document.createElement('div');
      overlay.style.cssText = 'position:absolute;inset:0;z-index:5;cursor:crosshair';
      host.appendChild(overlay);

      function onOverlayMove(e: PointerEvent) {
        if (!isDragging) {
          const dx = Math.abs(e.clientX - startX);
          const dy = Math.abs(e.clientY - startY);
          if (dx > 4 || dy > 4) {
            isDragging = true;
            selectionRect = document.createElement('div');
            selectionRect.className = 'selection-overlay';
            host.appendChild(selectionRect);
          }
        }
        if (isDragging && selectionRect) {
          const hostRect = host.getBoundingClientRect();
          selectionRect.style.left = Math.min(startX, e.clientX) - hostRect.left + 'px';
          selectionRect.style.top = Math.min(startY, e.clientY) - hostRect.top + 'px';
          selectionRect.style.width = Math.abs(e.clientX - startX) + 'px';
          selectionRect.style.height = Math.abs(e.clientY - startY) + 'px';
        }
      }

      function onOverlayUp(e: PointerEvent) {
        overlay.removeEventListener('pointermove', onOverlayMove);
        overlay.removeEventListener('pointerup', onOverlayUp);
        if (selectionRect) host.removeChild(selectionRect);
        host.removeChild(overlay);
        controls.enabled = true;

        if (isDragging) {
          // Box selection: compute and apply
          const hitIds = getObjectsInRect(viewer, startX, startY, e.clientX, e.clientY);
          const current = new Set(selectedIdsRef.current);
          for (const id of hitIds) {
            if (current.has(id)) current.delete(id);
            else current.add(id);
          }
          onSelectionChangedRef.current?.([...current]);
        } else {
          // No drag (click): dispatch synthetic click for normal processing
          renderer.domElement.dispatchEvent(new MouseEvent('click', {
            clientX: startX, clientY: startY,
            ctrlKey: true, bubbles: true,
          }));
        }
      }

      overlay.addEventListener('pointermove', onOverlayMove);
      overlay.addEventListener('pointerup', onOverlayUp);
      overlay.setPointerCapture(event.pointerId);
    }

    // ── Pointer handler for box selection ──
    function onPointerDown(event: PointerEvent) {
      if (event.button !== 0) return;
      if (draggingTransformRef.current) return;
      if (!(event.ctrlKey || event.metaKey)) return;
      // Disable OrbitControls before overlay takes capture, preventing any camera movement
      controls.enabled = false;
      startBoxSelection(event);
    }

    // ── Click handler ──
    function onClick(event: MouseEvent) {
      if (draggingTransformRef.current) return;

      scene.updateMatrixWorld(true);

      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
      raycaster.setFromCamera(pointer, viewer.camera);

      // ── Interactive 3D point-picking mode (with snap) ──
      if (creationPhaseRef.current === 'picking') {
        event.stopPropagation();
        removePreviewLine(previewLineRef, pickingMarkerGroupRef);
        if (layerPreviewRef.current) {
          clearMarkerGroup(layerPreviewRef.current);
          pickingMarkerGroupRef.current.remove(layerPreviewRef.current);
          layerPreviewRef.current = null;
        }
        const pt = pickGroundPoint(
          { camera: viewer.camera, renderer: viewer.renderer },
          event.clientX, event.clientY
        );
        if (!pt) return;

        // Convert to Vector3 and apply snapping
        const vecPt = new THREE.Vector3(pt.x, pt.y, pt.z);
        const snapResult = snapPoint(vecPt, viewer.meshes, snapSettingsRef.current);
        const finalPt = snapResult.snapped ? snapResult.point : vecPt;

        const current = creationPointsRef.current;
        const form = creationFormRef.current ?? createInitialForm(creationObjectTypeRef.current ?? 'wall');
        const mode = pickModeForForm(form);

        // Polygon loop-close: clicking near the first point closes the shape
        // and snaps the new vertex onto it (§1.3 auto-completion).
        let resolved = { x: finalPt.x, y: finalPt.y, z: finalPt.z };
        if (mode.mode === 'polygon' && current.length >= 2 && closeToPoint(current[0], resolved)) {
          resolved = current[0];
        }
        const updated = [...current, resolved];
        creationDispatchRef.current({ type: 'collectPoint', point: resolved });

        // Visual marker + mode-aware preview
        addPointMarker(pickingMarkerGroupRef.current, resolved);
        if (updated.length > 1) {
          if (mode.mode === 'polygon') {
            addPreviewLine(pickingMarkerGroupRef.current, [...updated, updated[0]]);
            if (updated.length >= 3) addPolygonFill(pickingMarkerGroupRef.current, updated);
          } else if (mode.mode === 'place') {
            addPlaceCursor(pickingMarkerGroupRef.current, resolved);
          } else {
            addPreviewLine(pickingMarkerGroupRef.current, updated);
          }
          if (LAYERED_TYPES.has(form.objectType) && mode.mode !== 'polygon') {
            const layers = form.layers;
            if (layers && layers.length > 0) {
              const group = addLayerPreview(
                pickingMarkerGroupRef.current,
                updated[updated.length - 2],
                resolved,
                layers,
                parseFloat(form.height || '3') || 3,
              );
              layerPreviewRef.current = group;
            }
          }
        } else if (mode.mode === 'place') {
          addPlaceCursor(pickingMarkerGroupRef.current, resolved);
        }

        // Auto-completion for segment / place / closed polygon (§1.3).
        if (
          (mode.mode === 'segment' && updated.length >= 2) ||
          (mode.mode === 'place' && updated.length >= 1) ||
          (mode.mode === 'polygon' && updated.length >= 3 && resolved === current[0])
        ) {
          completePickingWith(updated);
        }
        return;
      }

      if (measureModeRef.current !== 'none') {
        event.stopPropagation();
        // In measurement mode, raycast against meshes and ground plane, with snap
        const allTargets = [...viewer.meshes.values()];
        const hits = raycaster.intersectObjects(allTargets);

        let hitPoint: THREE.Vector3;
        if (hits.length > 0) {
          hitPoint = hits[0].point.clone();
        } else {
          // Raycast against ground plane (z=0)
          const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
          const pt = new THREE.Vector3();
          const ray = raycaster.ray;
          const intersect = ray.intersectPlane(plane, pt);
          if (!intersect) return;
          hitPoint = pt.clone();
        }

        // Apply snapping
        const snapResult = snapPoint(hitPoint, viewer.meshes, snapSettingsRef.current);
        const finalPt = snapResult.snapped ? snapResult.point : hitPoint;

        const mode = measureModeRef.current;
        const pts = measurePointsRef.current;
        pts.push(finalPt);

        // Visual feedback: show temporary point
        const dot = new THREE.Mesh(new THREE.SphereGeometry(0.1, 8, 8), new THREE.MeshBasicMaterial({ color: 0xffff00 }));
        dot.position.copy(finalPt);
        scene.add(dot);
        measurePreviewObjects.push(dot);

        if (mode === 'distance' && pts.length >= 2) {
          drawDistanceMeasurement(viewer, pts[0], pts[1]);
          pts.length = 0;
          clearMeasurePreviewObjects();
        } else if (mode === 'angle' && pts.length >= 3) {
          drawAngleMeasurement(viewer, pts[0], pts[1], pts[2]);
          pts.length = 0;
          clearMeasurePreviewObjects();
        } else if (mode === 'area' && pts.length >= 3) {
          // For area, keep adding points; show preview polygon
          clearMeasurePreviewObjects();
          const previewPts = [...pts];
          const previewLine = new THREE.Line(
            new THREE.BufferGeometry().setFromPoints([...previewPts, previewPts[0]]),
            new THREE.LineBasicMaterial({ color: 0x44aaff, transparent: true, opacity: 0.5 })
          );
          scene.add(previewLine);
          measurePreviewObjects.push(previewLine);
        }
        return;
      }

      const hits = raycaster.intersectObjects([...viewer.meshes.values()]);

      const ctrl = event.ctrlKey || event.metaKey;

      if (hits.length > 0) {
        const obj = hits[0].object;
        const data = obj.userData;
        if (data.id) {
          if (ctrl) {
            // Ctrl+click: toggle
            const current = new Set(selectedIdsRef.current);
            current.has(data.id) ? current.delete(data.id) : current.add(data.id);
            onSelectionChangedRef.current?.([...current]);
          } else {
            // Plain click: replace selection
            onSelectionChangedRef.current?.([data.id]);
          }
        }
      } else {
        // Click on empty: deselect all
        onSelectionChangedRef.current?.([]);
      }
    }

    // ── Double-click: complete polyline / polygon picking (§1.3) ──
    function onPickingDblClick(event: MouseEvent) {
      if (creationPhaseRef.current !== 'picking') return;
      const form = creationFormRef.current;
      const mode = form ? pickModeForForm(form) : defaultPickModeForType(creationObjectTypeRef.current ?? 'wall');
      if (mode.mode !== 'polyline' && mode.mode !== 'polygon') return;
      const points = creationPointsRef.current;
      if (points.length < 2) return;
      event.stopPropagation();
      completePickingWith(points);
    }
    renderer.domElement.addEventListener('dblclick', onPickingDblClick);

    const ORTHO_VIEWS: { name: ViewName; vec: THREE.Vector3 }[] = [
      { name: 'front', vec: new THREE.Vector3(0, 1, 0) },
      { name: 'back', vec: new THREE.Vector3(0, -1, 0) },
      { name: 'left', vec: new THREE.Vector3(1, 0, 0) },
      { name: 'right', vec: new THREE.Vector3(-1, 0, 0) },
      { name: 'top', vec: new THREE.Vector3(0, 0, 1) },
      { name: 'bottom', vec: new THREE.Vector3(0, 0, -1) },
      { name: 'front-left-top', vec: new THREE.Vector3(0.577, 0.577, 0.577) },
      { name: 'front-right-top', vec: new THREE.Vector3(-0.577, 0.577, 0.577) },
      { name: 'front-left-bottom', vec: new THREE.Vector3(0.577, 0.577, -0.577) },
      { name: 'front-right-bottom', vec: new THREE.Vector3(-0.577, 0.577, -0.577) },
      { name: 'back-left-top', vec: new THREE.Vector3(0.577, -0.577, 0.577) },
      { name: 'back-right-top', vec: new THREE.Vector3(-0.577, -0.577, 0.577) },
      { name: 'back-left-bottom', vec: new THREE.Vector3(0.577, -0.577, -0.577) },
      { name: 'back-right-bottom', vec: new THREE.Vector3(-0.577, -0.577, -0.577) },
      { name: 'front-top', vec: new THREE.Vector3(0, 0.707, 0.707) },
      { name: 'front-bottom', vec: new THREE.Vector3(0, 0.707, -0.707) },
      { name: 'back-top', vec: new THREE.Vector3(0, -0.707, 0.707) },
      { name: 'back-bottom', vec: new THREE.Vector3(0, -0.707, -0.707) },
      { name: 'front-left', vec: new THREE.Vector3(0.707, 0.707, 0) },
      { name: 'front-right', vec: new THREE.Vector3(-0.707, 0.707, 0) },
      { name: 'back-left', vec: new THREE.Vector3(0.707, -0.707, 0) },
      { name: 'back-right', vec: new THREE.Vector3(-0.707, -0.707, 0) },
      { name: 'top-left', vec: new THREE.Vector3(0.707, 0, 0.707) },
      { name: 'top-right', vec: new THREE.Vector3(-0.707, 0, 0.707) },
      { name: 'bottom-left', vec: new THREE.Vector3(0.707, 0, -0.707) },
      { name: 'bottom-right', vec: new THREE.Vector3(-0.707, 0, -0.707) },
    ];

    function getOrthoView(dir: THREE.Vector3): ViewName {
      for (const ov of ORTHO_VIEWS) {
        if (dir.distanceTo(ov.vec) < 0.01) return ov.name;
      }
      return 'iso';
    }

    let frame: number;
    function animate() {
      frame = requestAnimationFrame(animate);
      controls.update();
      transformControls.update(0.016);
      if (viewer.isOrtho) {
        const dist = Math.max(camera.position.distanceTo(controls.target), 0.01);
        const aspect = camera.aspect;
        const halfH = dist * Math.tan((camera.fov * Math.PI) / 360) / Math.max(camera.zoom, 0.01);
        const halfW = halfH * aspect;
        camera.projectionMatrix.makeOrthographic(-halfW, halfW, halfH, -halfH, 0.1, 2000);
        camera.projectionMatrixInverse.copy(camera.projectionMatrix).invert();
      } else {
        camera.updateProjectionMatrix();
      }
      renderer.render(scene, camera);
      viewCubeGizmo.update();
      const dir = camera.position.clone().sub(controls.target).normalize();
      cameraDirRef.current.copy(dir);
      viewer.currentView = getOrthoView(dir);
    }

    // ── Hover highlighting ──
    let hoveredMesh: THREE.Mesh | null = null;
    let hoveredOriginalEmissive: THREE.Color | null = null;

    function onPointerMove(event: MouseEvent) {
      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
      raycaster.setFromCamera(pointer, viewer.camera);

      // ── Live snap indicator while picking or measuring ──
      if (creationPhaseRef.current === 'picking' || measureModeRef.current !== 'none') {
        const settings = snapSettingsRef.current;
        const hits = raycaster.intersectObjects([...viewer.meshes.values()]);
        const rawHits = hits.length > 0
          ? hits[0].point
          : (() => {
              const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
              const pt = new THREE.Vector3();
              return raycaster.ray.intersectPlane(plane, pt) ? pt.clone() : undefined;
            })();
        if (rawHits) {
          const snapResult = snapPoint(rawHits, viewer.meshes, settings);
          if (snapResult.snapped && snapResult.snapType && snapResult.distance < settings.threshold) {
            viewer.snapIndicator.show(snapResult.point, snapResult.snapType);
          } else {
            viewer.snapIndicator.hide();
          }
        } else {
          viewer.snapIndicator.hide();
        }
      } else {
        viewer.snapIndicator.hide();
      }

      // ── Live dashed preview line while picking (from last point to cursor) ──
      if (creationPhaseRef.current === 'picking') {
        const currentPoints = creationPointsRef.current;
        removePreviewLine(previewLineRef, pickingMarkerGroupRef);
        // Clear previous layer-stack preview.
        if (layerPreviewRef.current) {
          clearMarkerGroup(layerPreviewRef.current);
          pickingMarkerGroupRef.current.remove(layerPreviewRef.current);
          layerPreviewRef.current = null;
        }
        const form = creationFormRef.current;
        const mode = form ? pickModeForForm(form) : defaultPickModeForType(creationObjectTypeRef.current ?? 'wall');
        if (mode.mode !== 'place' || currentPoints.length === 0) {
          const pickPt = pickGroundPoint(
            { camera: viewer.camera, renderer: viewer.renderer },
            event.clientX, event.clientY
          );
          if (pickPt) {
            // For layered surface types, preview the full layer stack as colored boxes.
            const objType = creationObjectTypeRef.current;
            if (mode.mode !== 'polygon' && objType && LAYERED_TYPES.has(objType) && currentPoints.length >= 1
              && form?.layers && form.layers.length > 0) {
              const last = currentPoints[currentPoints.length - 1];
              const group = addLayerPreview(
                pickingMarkerGroupRef.current,
                last,
                pickPt,
                form.layers,
                parseFloat(form.height || '3') || 3,
              );
              layerPreviewRef.current = group;
            } else if (mode.mode === 'polygon' && currentPoints.length >= 1) {
              const closed = [...currentPoints, pickPt];
              addPreviewLine(pickingMarkerGroupRef.current, closed);
              if (closed.length >= 3) addPolygonFill(pickingMarkerGroupRef.current, closed);
            } else if (mode.mode === 'place' && currentPoints.length === 0) {
              addPlaceCursor(pickingMarkerGroupRef.current, pickPt);
            } else if (currentPoints.length >= 1) {
              const dashed = createDashedPreviewLine(currentPoints[currentPoints.length - 1], pickPt);
              pickingMarkerGroupRef.current.add(dashed);
              previewLineRef.current = dashed;
            }
          }
        }
        return;
      }

      const hits = raycaster.intersectObjects([...viewer.meshes.values()]);

      // Restore previous hovered mesh
      if (hoveredMesh) {
        const mat = hoveredMesh.material as THREE.MeshStandardMaterial;
        if (hoveredOriginalEmissive) {
          mat.emissive.copy(hoveredOriginalEmissive);
        }
        mat.emissiveIntensity = 0;
        hoveredMesh = null;
        hoveredOriginalEmissive = null;
      }

      if (hits.length > 0) {
        const obj = hits[0].object as THREE.Mesh;
        const mat = obj.material as THREE.MeshStandardMaterial;
        hoveredOriginalEmissive = mat.emissive.clone();
        mat.emissive.setHex(0x4488ff);
        mat.emissiveIntensity = 0.3;
        hoveredMesh = obj;
        renderer.domElement.style.cursor = 'pointer';
      } else {
        renderer.domElement.style.cursor = 'default';
      }
    }

    function onContextMenu(event: MouseEvent) {
      if (measureModeRef.current === 'area' && measurePointsRef.current.length >= 3) {
        event.preventDefault();
        const s = stateRef.current;
        if (s) {
          drawAreaMeasurement(s, measurePointsRef.current);
          measurePointsRef.current = [];
          clearMeasurePreviewObjects();
        }
      }
    }

    const resizeObserver = new ResizeObserver(() => resize());
    resizeObserver.observe(host);
    renderer.domElement.addEventListener('click', onClick);
    renderer.domElement.addEventListener('pointerdown', onPointerDown);
    renderer.domElement.addEventListener('pointermove', onPointerMove);
    renderer.domElement.addEventListener('dblclick', onPickingDblClick);
    renderer.domElement.addEventListener('contextmenu', onContextMenu);

    // ── Drag-and-drop from catalog ──
    function onDragOver(event: DragEvent) {
      event.preventDefault();
      host.classList.add('drag-over');
    }
    function onDragLeave() {
      host.classList.remove('drag-over');
    }
    function onDrop(event: DragEvent) {
      event.preventDefault();
      host.classList.remove('drag-over');
      const raw = event.dataTransfer?.getData('application/x-omnim-catalog');
      if (!raw) return;
      let data: { component_def_id: string; instance_name: string; params: Record<string, number> };
      try { data = JSON.parse(raw); } catch { return; }
      const rect = renderer.domElement.getBoundingClientRect();
      pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
      raycaster.setFromCamera(pointer, viewer.camera);
      const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), 0);
      const pt = new THREE.Vector3();
      if (!raycaster.ray.intersectPlane(plane, pt)) return;
      api.instantiateFromCatalog({
        ...data,
        position: { x: pt.x, y: pt.y, z: pt.z },
        skip_series_gate: true,
      }).then(() => {
        onChangedRef.current?.();
      }).catch((err) => {
        console.error('Drop instantiation failed:', err);
      });
    }
    host.addEventListener('dragover', onDragOver);
    host.addEventListener('dragleave', onDragLeave);
    host.addEventListener('drop', onDrop);
    animate();

    void loadMeshes(viewer).then(() => {
      applyVisualMode(viewer, visualMode);
      applySection(viewer, section);
      selectMeshes(viewer, selectedIds);
      fitCamera(viewer);
    });

    return () => {
      cancelAnimationFrame(frame);
      viewCubeGizmo.dispose();
      renderer.domElement.removeEventListener('dblclick', onDblClick);
      resizeObserver.disconnect();
      renderer.domElement.removeEventListener('click', onClick);
      renderer.domElement.removeEventListener('pointerdown', onPointerDown);
      renderer.domElement.removeEventListener('pointermove', onPointerMove);
      renderer.domElement.removeEventListener('dblclick', onPickingDblClick);
      renderer.domElement.removeEventListener('contextmenu', onContextMenu);
      host.removeEventListener('dragover', onDragOver);
      host.removeEventListener('dragleave', onDragLeave);
      host.removeEventListener('drop', onDrop);
      disposeMeshes(viewer);
      clearMeasureGroup(viewer);
      clearMeasurePreviewObjects();
      viewer.snapIndicator.dispose();
      transformControls.dispose();
      controls.dispose();
      renderer.dispose();
      if (host.contains(renderer.domElement)) host.removeChild(renderer.domElement);
      stateRef.current = null;
    };
  }, []);

  useEffect(() => {
    const viewer = stateRef.current;
    if (!viewer) return;
    if (transformMode) {
      viewer.transformControls.setMode(transformMode);
      if (!viewer.transformControls.object && selectedIds.length === 1) {
        const mesh = viewer.meshes.get(selectedIds[0]);
        if (mesh) viewer.transformControls.attach(mesh);
      }
    } else {
      viewer.transformControls.detach();
    }
  }, [transformMode]);

  useEffect(() => {
    const viewer = stateRef.current;
    if (!viewer) return;
    selectMeshes(viewer, selectedIds);
  }, [selectedIds]);

  useEffect(() => {
    const viewer = stateRef.current;
    if (!viewer) return;
    applyVisualMode(viewer, visualMode);
  }, [visualMode]);

  useEffect(() => {
    const viewer = stateRef.current;
    if (!viewer) return;
    applySection(viewer, section);
  }, [section]);

  useEffect(() => {
    const viewer = stateRef.current;
    if (!viewer) return;
    void loadMeshes(viewer).then(() => {
      applyVisualMode(viewer, visualMode);
      applySection(viewer, section);
      selectMeshes(viewer, selectedIdsRef.current);
      void loadMeasurements(viewer);
    });
  }, [reloadToken]);

  return (
    <div ref={hostRef} className={`three-viewer ${viewerStyles.threeViewer}`}>
      {creationPhase === 'picking' && creationObjectType && (
        <div className="picking-hint">
          <span>📍 {pickingHint(creationObjectType, creationPoints, creationForm ? pickModeForForm(creationForm) : undefined)} ，按 Esc 取消</span>
        </div>
      )}
    </div>
  );
});

let measurePreviewObjects: THREE.Object3D[] = [];

let loadGen = 0;
async function loadMeshes(viewer: ViewerState) {
  const gen = ++loadGen;
  disposeMeshes(viewer);
  const mesh = await api.mesh();
  if (gen !== loadGen) return;
  for (const object of mesh.objects) {
    const viewMesh = createMesh(object);
    viewer.group.add(viewMesh);
    viewer.meshes.set(object.id, viewMesh);
  }
}

function clearMeasureGroup(viewer: ViewerState) {
  while (viewer.measureGroup.children.length) {
    const child = viewer.measureGroup.children[0];
    viewer.measureGroup.remove(child);
    if (child instanceof THREE.Line) {
      child.geometry.dispose();
      (child.material as THREE.Material).dispose();
    } else if (child instanceof THREE.Mesh) {
      child.geometry.dispose();
      (child.material as THREE.Material).dispose();
    } else if (child instanceof THREE.Sprite) {
      child.material.map?.dispose();
      child.material.dispose();
    }
  }
}

function clearMeasurePreviewObjects() {
  for (const obj of measurePreviewObjects) {
    if (obj.parent) obj.removeFromParent();
    if (obj instanceof THREE.Line) {
      obj.geometry.dispose();
      (obj.material as THREE.Material).dispose();
    } else if (obj instanceof THREE.Mesh) {
      obj.geometry.dispose();
      (obj.material as THREE.Material).dispose();
    } else if (obj instanceof THREE.Sprite) {
      obj.material.map?.dispose();
      obj.material.dispose();
    }
  }
  measurePreviewObjects = [];
}

function createLabelSprite(text: string, color = '#ffffff', bgColor = 'rgba(0,0,0,0.7)'): THREE.Sprite {
  const canvas = document.createElement('canvas');
  canvas.width = 256;
  canvas.height = 64;
  const ctx = canvas.getContext('2d')!;
  ctx.clearRect(0, 0, 256, 64);
  ctx.fillStyle = bgColor;
  ctx.roundRect(4, 4, 248, 56, 8);
  ctx.fill();
  ctx.font = 'bold 24px monospace';
  ctx.fillStyle = color;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, 128, 34);
  const tex = new THREE.CanvasTexture(canvas);
  tex.minFilter = THREE.LinearFilter;
  const mat = new THREE.SpriteMaterial({ map: tex, transparent: true, depthTest: false, sizeAttenuation: true });
  const sprite = new THREE.Sprite(mat);
  sprite.scale.set(2, 0.5, 1);
  return sprite;
}

function drawDistanceMeasurement(viewer: ViewerState, p1: THREE.Vector3, p2: THREE.Vector3) {
  const mid = p1.clone().add(p2).multiplyScalar(0.5);
  const dist = p1.distanceTo(p2);
  const points = [p1, p2];
  const geo = new THREE.BufferGeometry().setFromPoints(points);
  const line = new THREE.Line(geo, new THREE.LineBasicMaterial({ color: 0x44ff88, linewidth: 2 }));
  viewer.measureGroup.add(line);

  // Endpoint dots
  for (const p of [p1, p2]) {
    const dotGeo = new THREE.SphereGeometry(0.08, 8, 8);
    const dotMat = new THREE.MeshBasicMaterial({ color: 0x44ff88 });
    const dot = new THREE.Mesh(dotGeo, dotMat);
    dot.position.copy(p);
    viewer.measureGroup.add(dot);
  }

  const label = createLabelSprite(`${dist.toFixed(2)}`);
  label.position.copy(mid);
  viewer.measureGroup.add(label);

  // Persist to backend
  api.createMeasurement({
    type: 'distance',
    points: [{ x: p1.x, y: p1.y, z: p1.z }, { x: p2.x, y: p2.y, z: p2.z }],
    value: dist,
    color: '#44ff88',
  }).catch(() => {});
}

function drawAngleMeasurement(viewer: ViewerState, vertex: THREE.Vector3, p1: THREE.Vector3, p2: THREE.Vector3) {
  const v1 = p1.clone().sub(vertex).normalize();
  const v2 = p2.clone().sub(vertex).normalize();
  const angle = v1.angleTo(v2);
  const radius = Math.min(vertex.distanceTo(p1), vertex.distanceTo(p2)) * 0.4;

  const segments = 32;
  const arcPoints: THREE.Vector3[] = [];
  for (let i = 0; i <= segments; i++) {
    const t = (i / segments) * angle;
    const dir = v1.clone().multiplyScalar(Math.cos(t)).add(v2.clone().multiplyScalar(Math.sin(t))).normalize();
    arcPoints.push(vertex.clone().add(dir.multiplyScalar(radius)));
  }
  const geo = new THREE.BufferGeometry().setFromPoints(arcPoints);
  const line = new THREE.Line(geo, new THREE.LineBasicMaterial({ color: 0xffaa44 }));
  viewer.measureGroup.add(line);

  // Ray lines
  for (const p of [p1, p2]) {
    const rayGeo = new THREE.BufferGeometry().setFromPoints([vertex, p]);
    const rayLine = new THREE.Line(rayGeo, new THREE.LineBasicMaterial({ color: 0xffaa44, transparent: true, opacity: 0.5 }));
    viewer.measureGroup.add(rayLine);
  }

  const vertexDot = new THREE.Mesh(new THREE.SphereGeometry(0.08, 8, 8), new THREE.MeshBasicMaterial({ color: 0xffaa44 }));
  vertexDot.position.copy(vertex);
  viewer.measureGroup.add(vertexDot);

  const midDir = v1.clone().add(v2).normalize();
  const labelPos = vertex.clone().add(midDir.multiplyScalar(radius * 1.3));
  const label = createLabelSprite(`${(angle * 180 / Math.PI).toFixed(1)}°`);
  label.position.copy(labelPos);
  viewer.measureGroup.add(label);

  // Persist
  const deg = angle * 180 / Math.PI;
  api.createMeasurement({
    type: 'angle',
    points: [{ x: vertex.x, y: vertex.y, z: vertex.z }, { x: p1.x, y: p1.y, z: p1.z }, { x: p2.x, y: p2.y, z: p2.z }],
    value: deg,
    color: '#ffaa44',
  }).catch(() => {});
}

function drawAreaMeasurement(viewer: ViewerState, points: THREE.Vector3[]) {
  const vertices: number[] = [];
  const indices: number[] = [];
  for (let i = 0; i < points.length; i++) {
    vertices.push(points[i].x, points[i].y, points[i].z);
  }
  for (let i = 1; i < points.length - 1; i++) {
    indices.push(0, i, i + 1);
  }
  if (indices.length === 0) return;

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
  geo.setIndex(indices);
  geo.computeVertexNormals();
  const mat = new THREE.MeshBasicMaterial({ color: 0x44aaff, transparent: true, opacity: 0.2, side: THREE.DoubleSide, depthWrite: false });
  const mesh = new THREE.Mesh(geo, mat);
  viewer.measureGroup.add(mesh);

  // Outline
  const outlinePoints = [...points, points[0]];
  const outlineGeo = new THREE.BufferGeometry().setFromPoints(outlinePoints);
  const outlineLine = new THREE.Line(outlineGeo, new THREE.LineBasicMaterial({ color: 0x44aaff }));
  viewer.measureGroup.add(outlineLine);

  // Endpoint dots
  for (const p of points) {
    const dot = new THREE.Mesh(new THREE.SphereGeometry(0.08, 8, 8), new THREE.MeshBasicMaterial({ color: 0x44aaff }));
    dot.position.copy(p);
    viewer.measureGroup.add(dot);
  }

  // Compute area using shoelace formula on projected 2D
  let area = 0;
  const n = points.length;
  for (let i = 0; i < n; i++) {
    const j = (i + 1) % n;
    area += points[i].x * points[j].y;
    area -= points[j].x * points[i].y;
  }
  area = Math.abs(area) / 2;

  const center = new THREE.Vector3();
  for (const p of points) center.add(p);
  center.divideScalar(points.length);

  const label = createLabelSprite(`Area: ${area.toFixed(2)}`);
  label.position.copy(center);
  viewer.measureGroup.add(label);

  // Persist
  const pts = points.map((p) => ({ x: p.x, y: p.y, z: p.z }));
  api.createMeasurement({
    type: 'area',
    points: pts,
    value: area,
    color: '#44aaff',
  }).catch(() => {});
}

/** Render a persisted Measurement object into the scene's measureGroup. */
function renderMeasurement(viewer: ViewerState, m: Measurement) {
  const pts = m.points.map((p) => new THREE.Vector3(p.x, p.y, p.z));
  const color = parseInt(m.color.replace('#', ''), 16);

  if (m.type === 'distance' && pts.length >= 2) {
    const mid = pts[0].clone().add(pts[1]).multiplyScalar(0.5);
    const geo = new THREE.BufferGeometry().setFromPoints([pts[0], pts[1]]);
    const line = new THREE.Line(geo, new THREE.LineBasicMaterial({ color }));
    viewer.measureGroup.add(line);
    for (const p of [pts[0], pts[1]]) {
      const dot = new THREE.Mesh(new THREE.SphereGeometry(0.08, 8, 8), new THREE.MeshBasicMaterial({ color }));
      dot.position.copy(p);
      viewer.measureGroup.add(dot);
    }
    const label = createLabelSprite(`${m.value.toFixed(2)}`);
    label.position.copy(mid);
    viewer.measureGroup.add(label);
  } else if (m.type === 'angle' && pts.length >= 3) {
    const [vertex, p1, p2] = pts;
    const v1 = p1.clone().sub(vertex).normalize();
    const v2 = p2.clone().sub(vertex).normalize();
    const angle = m.value * Math.PI / 180;
    const radius = Math.min(vertex.distanceTo(p1), vertex.distanceTo(p2)) * 0.4;
    const segments = 32;
    const arcPoints: THREE.Vector3[] = [];
    for (let i = 0; i <= segments; i++) {
      const t = (i / segments) * angle;
      const dir = v1.clone().multiplyScalar(Math.cos(t)).add(v2.clone().multiplyScalar(Math.sin(t))).normalize();
      arcPoints.push(vertex.clone().add(dir.multiplyScalar(radius)));
    }
    const arcGeo = new THREE.BufferGeometry().setFromPoints(arcPoints);
    const arcLine = new THREE.Line(arcGeo, new THREE.LineBasicMaterial({ color }));
    viewer.measureGroup.add(arcLine);
    for (const p of [p1, p2]) {
      const rayGeo = new THREE.BufferGeometry().setFromPoints([vertex, p]);
      const rayLine = new THREE.Line(rayGeo, new THREE.LineBasicMaterial({ color, transparent: true, opacity: 0.5 }));
      viewer.measureGroup.add(rayLine);
    }
    const dot = new THREE.Mesh(new THREE.SphereGeometry(0.08, 8, 8), new THREE.MeshBasicMaterial({ color }));
    dot.position.copy(vertex);
    viewer.measureGroup.add(dot);
    const midDir = v1.clone().add(v2).normalize();
    const labelPos = vertex.clone().add(midDir.multiplyScalar(radius * 1.3));
    const label = createLabelSprite(`${m.value.toFixed(1)}°`);
    label.position.copy(labelPos);
    viewer.measureGroup.add(label);
  } else if (m.type === 'area' && pts.length >= 3) {
    const vertices: number[] = [];
    pts.forEach((p) => vertices.push(p.x, p.y, p.z));
    const indices: number[] = [];
    for (let i = 1; i < pts.length - 1; i++) indices.push(0, i, i + 1);
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
    geo.setIndex(indices);
    geo.computeVertexNormals();
    const mat = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.2, side: THREE.DoubleSide, depthWrite: false });
    const mesh = new THREE.Mesh(geo, mat);
    viewer.measureGroup.add(mesh);
    const outline = [...pts, pts[0]];
    const outlineGeo = new THREE.BufferGeometry().setFromPoints(outline);
    const outlineLine = new THREE.Line(outlineGeo, new THREE.LineBasicMaterial({ color }));
    viewer.measureGroup.add(outlineLine);
    for (const p of pts) {
      const dot = new THREE.Mesh(new THREE.SphereGeometry(0.08, 8, 8), new THREE.MeshBasicMaterial({ color }));
      dot.position.copy(p);
      viewer.measureGroup.add(dot);
    }
    const center = new THREE.Vector3();
    pts.forEach((p) => center.add(p));
    center.divideScalar(pts.length);
    const label = createLabelSprite(`Area: ${m.value.toFixed(2)}`);
    label.position.copy(center);
    viewer.measureGroup.add(label);
  }
}

/** Load persisted measurements from the backend and render them. */
async function loadMeasurements(viewer: ViewerState) {
  try {
    const measurements = await api.listMeasurements();
    for (const m of measurements) {
      renderMeasurement(viewer, m);
    }
  } catch { /* ignore */ }
}

function disposeMeshes(viewer: ViewerState) {
  disposeEdgeHelpers(viewer);
  for (const mesh of viewer.meshes.values()) {
    viewer.group.remove(mesh);
    mesh.geometry.dispose();
    if (Array.isArray(mesh.material)) mesh.material.forEach((material) => material.dispose());
    else mesh.material.dispose();
  }
  viewer.meshes.clear();
}

function createMesh(object: MeshObject) {
  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.Float32BufferAttribute(object.positions, 3));
  if (object.normals.length === object.positions.length) geometry.setAttribute('normal', new THREE.Float32BufferAttribute(object.normals, 3));
  else geometry.computeVertexNormals();

  geometry.computeBoundingBox();
  const center = geometry.boundingBox!.getCenter(new THREE.Vector3());
  const posAttr = geometry.getAttribute('position');
  for (let i = 0; i < posAttr.count; i++) {
    posAttr.setXYZ(i,
      posAttr.getX(i) - center.x,
      posAttr.getY(i) - center.y,
      posAttr.getZ(i) - center.z,
    );
  }
  posAttr.needsUpdate = true;
  geometry.computeBoundingBox();
  geometry.computeBoundingSphere();

  const material = new THREE.MeshStandardMaterial({ color: colorForType(object.object_type), roughness: 0.55, metalness: 0.15, side: THREE.DoubleSide });
  const mesh = new THREE.Mesh(geometry, material);
  mesh.position.copy(center);
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  mesh.userData = {
    id: object.id,
    name: object.name,
    type: object.object_type,
    color: colorForType(object.object_type),
    vcount: object.vertex_count,
    tcount: object.triangle_count,
    bbox: [object.bbox_min, object.bbox_max],
    basePosition: center.clone(),
    baseQuaternion: new THREE.Quaternion(),
    baseScale: new THREE.Vector3(1, 1, 1),
  };
  return mesh;
}

function selectMeshes(viewer: ViewerState, ids: readonly string[]) {
  if (ids.length === 1 && viewer.meshes.get(ids[0]) === viewer.transformControls.object) return;
  viewer.transformControls.detach();
  for (const [meshId, mesh] of viewer.meshes) {
    const material = mesh.material as THREE.MeshStandardMaterial;
    const isSelected = ids.includes(meshId);
    material.emissive.set(isSelected ? 0x446688 : 0x000000);
    material.emissiveIntensity = isSelected ? 0.5 : 0;
    if (isSelected && ids.length === 1) {
      mesh.userData.basePosition = mesh.position.clone();
      mesh.userData.baseQuaternion = mesh.quaternion.clone();
      mesh.userData.baseScale = mesh.scale.clone();
      viewer.transformControls.attach(mesh);
    }
  }
}

async function persistSelectedTransform(viewer: ViewerState, selectedId?: string) {
  if (!selectedId) return;
  const mesh = viewer.meshes.get(selectedId);
  if (!mesh) return;
  const basePosition = mesh.userData.basePosition || new THREE.Vector3();
  const baseQuaternion = mesh.userData.baseQuaternion || new THREE.Quaternion();
  const baseScale = mesh.userData.baseScale || new THREE.Vector3(1, 1, 1);
  const deltaPos = mesh.position.clone().sub(basePosition);
  const scale = mesh.scale.clone().divide(baseScale);
  const deltaQuat = mesh.quaternion.clone().multiply(baseQuaternion.clone().invert());
  const hasMove = deltaPos.lengthSq() > 1e-10;
  const hasScale = Math.abs(scale.x - 1) + Math.abs(scale.y - 1) + Math.abs(scale.z - 1) > 1e-6;
  const hasRotate = Math.abs(deltaQuat.x) + Math.abs(deltaQuat.y) + Math.abs(deltaQuat.z) > 1e-6;
  if (!hasMove && !hasScale && !hasRotate) return;
  const origin = mesh.position.clone();
  const response = await api.transformObject(selectedId, {
    origin: { x: origin.x, y: origin.y, z: origin.z },
    translation: { x: deltaPos.x, y: deltaPos.y, z: deltaPos.z },
    rotation: hasRotate ? { x: deltaQuat.x, y: deltaQuat.y, z: deltaQuat.z, w: deltaQuat.w } : null,
    scale: { x: scale.x, y: scale.y, z: scale.z },
  });
  if (!response.ok) {
    alert(`Save transform failed: ${response.status} ${response.statusText}`);
    return;
  }
  mesh.userData.basePosition = mesh.position.clone();
  mesh.userData.baseQuaternion = mesh.quaternion.clone();
  mesh.userData.baseScale = mesh.scale.clone();
}

function disposeEdgeHelpers(viewer: ViewerState) {
  for (const [, helper] of viewer.edgeHelpers) {
    if (helper.parent) helper.removeFromParent();
    helper.geometry.dispose();
    (helper.material as THREE.Material).dispose();
  }
  viewer.edgeHelpers.clear();
}

function addEdgeHelper(viewer: ViewerState, id: string, mesh: THREE.Mesh, color = 0x222222, threshold = 1) {
  removeEdgeHelper(viewer, id);
  const geo = new THREE.EdgesGeometry(mesh.geometry, threshold);
  const helper = new THREE.LineSegments(geo, new THREE.LineBasicMaterial({ color }));
  helper.renderOrder = 1;
  mesh.add(helper);
  viewer.edgeHelpers.set(id, helper);
}

function removeEdgeHelper(viewer: ViewerState, id: string) {
  const helper = viewer.edgeHelpers.get(id);
  if (!helper) return;
  if (helper.parent) helper.removeFromParent();
  helper.geometry.dispose();
  (helper.material as THREE.Material).dispose();
  viewer.edgeHelpers.delete(id);
}

function applyVisualMode(viewer: ViewerState, mode: VisualMode) {
  viewer.sunLight.castShadow = mode === 'realistic';

  for (const [id, mesh] of viewer.meshes) {
    const material = mesh.material as THREE.MeshStandardMaterial;
    const origColor = (mesh.userData.color as number) ?? 0x1565c0;
    material.wireframe = false;
    material.transparent = false;
    material.opacity = 1;
    removeEdgeHelper(viewer, id);
    switch (mode) {
      case 'wireframe':
        material.color.setHex(origColor);
        material.transparent = true;
        material.opacity = 0.05;
        addEdgeHelper(viewer, id, mesh, 0x444444, 1);
        break;
      case 'hidden-line':
        material.color.setHex(0xf0f0f0);
        material.roughness = 1;
        material.metalness = 0;
        addEdgeHelper(viewer, id, mesh, 0x222222, 20);
        break;
      case 'realistic':
        material.color.setHex(origColor);
        material.roughness = 0.2;
        material.metalness = 0.6;
        break;
      case 'x-ray':
        material.color.setHex(origColor);
        material.transparent = true;
        material.opacity = 0.3;
        addEdgeHelper(viewer, id, mesh, 0x66aaff, 20);
        break;
      default:
        material.color.setHex(origColor);
        material.roughness = 0.55;
        material.metalness = 0.15;
        break;
    }
  }
}

function applySection(viewer: ViewerState, section: SectionState) {
  if (!section.enabled) {
    if (viewer.sectionPlaneMesh) {
      viewer.scene.remove(viewer.sectionPlaneMesh);
      viewer.sectionPlaneMesh.geometry.dispose();
      if (Array.isArray(viewer.sectionPlaneMesh.material)) viewer.sectionPlaneMesh.material.forEach((material) => material.dispose());
      else viewer.sectionPlaneMesh.material.dispose();
      viewer.sectionPlaneMesh = undefined;
    }
    for (const mesh of viewer.meshes.values()) {
      const material = mesh.material as THREE.MeshStandardMaterial;
      material.clippingPlanes = null;
      material.needsUpdate = true;
    }
    viewer.renderer.localClippingEnabled = false;
    return;
  }

  if (!viewer.sectionPlaneMesh) {
    viewer.sectionPlaneMesh = new THREE.Mesh(
      new THREE.PlaneGeometry(40, 40),
      new THREE.MeshBasicMaterial({ color: 0x4a90d9, transparent: true, opacity: 0.15, side: THREE.DoubleSide, depthWrite: false }),
    );
    viewer.scene.add(viewer.sectionPlaneMesh);
  }

  viewer.sectionPlaneMesh.rotation.set(0, 0, 0);
  viewer.sectionPlaneMesh.position.set(0, 0, 0);
  if (section.axis === 'x') {
    viewer.sectionPlaneMesh.rotation.y = Math.PI / 2;
    viewer.sectionPlaneMesh.position.x = section.value;
  } else if (section.axis === 'y') {
    viewer.sectionPlaneMesh.rotation.x = Math.PI / 2;
    viewer.sectionPlaneMesh.position.y = section.value;
  } else {
    viewer.sectionPlaneMesh.position.z = section.value;
  }

  const normal = section.axis === 'x' ? new THREE.Vector3(1, 0, 0) : section.axis === 'y' ? new THREE.Vector3(0, 1, 0) : new THREE.Vector3(0, 0, 1);
  const point = viewer.sectionPlaneMesh.position;
  const plane = new THREE.Plane(normal, -point.dot(normal));
  for (const mesh of viewer.meshes.values()) {
    const material = mesh.material as THREE.MeshStandardMaterial;
    material.clippingPlanes = [plane];
    material.clipShadows = true;
    material.needsUpdate = true;
  }
  viewer.renderer.localClippingEnabled = true;
}

function homeCamera(viewer: ViewerState) {
  viewer.camera.position.set(15, 20, 12);
  viewer.controls.target.set(0, 0, 0);
  viewer.controls.update();
}

function fitCamera(viewer: ViewerState) {
  if (viewer.meshes.size === 0) {
    homeCamera(viewer);
    return;
  }
  const box = new THREE.Box3();
  for (const mesh of viewer.meshes.values()) box.expandByObject(mesh);
  const center = box.getCenter(new THREE.Vector3());
  const size = box.getSize(new THREE.Vector3());
  const distance = Math.max(size.x, size.y, size.z) / (2 * Math.tan((viewer.camera.fov * Math.PI) / 360)) * 1.6;
  viewer.camera.position.copy(center).add(new THREE.Vector3(distance * 0.6, distance * 0.7, distance * 0.5));
  viewer.controls.target.copy(center);
  viewer.controls.update();
}

function setView(viewer: ViewerState, view: ViewName) {
  const directions: Record<ViewName, THREE.Vector3> = {
    top: new THREE.Vector3(0, 0, 1),
    bottom: new THREE.Vector3(0, 0, -1),
    front: new THREE.Vector3(0, 1, 0),
    back: new THREE.Vector3(0, -1, 0),
    left: new THREE.Vector3(1, 0, 0),
    right: new THREE.Vector3(-1, 0, 0),
    iso: new THREE.Vector3(0.7, 0.7, 0.6),
    'front-left-top': new THREE.Vector3(0.577, 0.577, 0.577),
    'front-right-top': new THREE.Vector3(-0.577, 0.577, 0.577),
    'front-left-bottom': new THREE.Vector3(0.577, 0.577, -0.577),
    'front-right-bottom': new THREE.Vector3(-0.577, 0.577, -0.577),
    'back-left-top': new THREE.Vector3(0.577, -0.577, 0.577),
    'back-right-top': new THREE.Vector3(-0.577, -0.577, 0.577),
    'back-left-bottom': new THREE.Vector3(0.577, -0.577, -0.577),
    'back-right-bottom': new THREE.Vector3(-0.577, -0.577, -0.577),
    'front-top': new THREE.Vector3(0, 0.707, 0.707),
    'front-bottom': new THREE.Vector3(0, 0.707, -0.707),
    'back-top': new THREE.Vector3(0, -0.707, 0.707),
    'back-bottom': new THREE.Vector3(0, -0.707, -0.707),
    'front-left': new THREE.Vector3(0.707, 0.707, 0),
    'front-right': new THREE.Vector3(-0.707, 0.707, 0),
    'back-left': new THREE.Vector3(0.707, -0.707, 0),
    'back-right': new THREE.Vector3(-0.707, -0.707, 0),
    'top-left': new THREE.Vector3(0.707, 0, 0.707),
    'top-right': new THREE.Vector3(-0.707, 0, 0.707),
    'bottom-left': new THREE.Vector3(0.707, 0, -0.707),
    'bottom-right': new THREE.Vector3(-0.707, 0, -0.707),
  };
  const target = getViewTarget(viewer);
  const distance = getViewDistance(viewer);
  viewer.camera.position.copy(target).add(directions[view].clone().normalize().multiplyScalar(distance));
  viewer.controls.target.copy(target);
  viewer.currentView = view;
  viewer.controls.update();
}

function rotateView(viewer: ViewerState, dx: number, dy: number) {
  const offset = viewer.camera.position.clone().sub(viewer.controls.target);
  const spherical = new THREE.Spherical().setFromVector3(offset);
  spherical.theta -= dx * 0.01;
  spherical.phi -= dy * 0.01;
  spherical.phi = Math.max(0.05, Math.min(Math.PI - 0.05, spherical.phi));
  offset.setFromSpherical(spherical);
  viewer.camera.position.copy(viewer.controls.target).add(offset);
  viewer.controls.update();
}

function getViewTarget(viewer: ViewerState) {
  const target = new THREE.Vector3();
  if (viewer.meshes.size > 0) {
    const box = new THREE.Box3();
    for (const mesh of viewer.meshes.values()) box.expandByObject(mesh);
    box.getCenter(target);
  }
  return target;
}

function getViewDistance(viewer: ViewerState) {
  if (viewer.meshes.size === 0) return 20;
  const box = new THREE.Box3();
  for (const mesh of viewer.meshes.values()) box.expandByObject(mesh);
  const size = box.getSize(new THREE.Vector3());
  return Math.max(20, Math.max(size.x, size.y, size.z) * 1.8);
}

// ── Sky Gradient ──

const skyTop = new THREE.Color(0x7eb8e0);
const skyHorizon = new THREE.Color(0xd4e6f0);
const skyBottom = new THREE.Color(0xc8dce6);

function createSkyGradient(): THREE.Color {
  return skyHorizon.clone();
}

// ── Preview line helpers ──

function closeToPoint(a: Point3D, b: Point3D, tol = 0.5): boolean {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  const dz = a.z - b.z;
  return dx * dx + dy * dy + dz * dz <= tol * tol;
}

function removePreviewLine(
  lineRef: React.MutableRefObject<THREE.Line | null>,
  groupRef: React.MutableRefObject<THREE.Group>,
) {
  const line = lineRef.current;
  if (line) {
    groupRef.current.remove(line);
    line.geometry.dispose();
    (line.material as THREE.Material).dispose();
    lineRef.current = null;
  }
}

// ── Color by Type ──

export function colorForType(type: string) {
  if (type.includes('Site')) return 0x4caf50;
  if (type.includes('Building')) return 0x2196f3;
  if (type.includes('Wall')) return 0xff9800;
  if (type.includes('Beam')) return 0xff5722;
  if (type.includes('Column')) return 0xf44336;
  if (type.includes('Slab')) return 0x9c27b0;
  if (type.includes('Door')) return 0x795548;
  if (type.includes('Window')) return 0x607d8b;
  if (type.includes('Roof')) return 0x8bc34a;
  if (type.includes('Tank')) return 0x00bcd4;
  if (type.includes('Pipe')) return 0x673ab7;
  if (type.includes('Part')) return 0xffeb3b;
  return 0x1565c0;
}
