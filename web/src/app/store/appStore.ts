import { create } from 'zustand';
import { api } from '../../shared/api';
import type { StatsResponse, TreeNode } from '../../shared/types';
import type { TransformMode } from '../components/MainToolbar';
import type { VisualMode, MeasureMode, SectionState, MeshInfo } from '../components/ThreeViewer';
import type { CreationForm, CreationPhase, ObjectType, Point3D } from '../../shared/types/creation';
import type { SnapSettings } from '../components/viewer/three/Snapping';
import { DEFAULT_SNAP_SETTINGS } from '../components/viewer/three/Snapping';
import { creationReducer, initialCreationState, type CreationAction, type CreationState } from '../../shared/creation/stateMachine';

export type AppState = {
  selectedIds: string[];
  primaryMeshInfo: MeshInfo | null;
  transformMode: TransformMode;
  visualMode: VisualMode;
  section: SectionState;
  measureMode: MeasureMode;
  reloadToken: number;
  gridVisible: boolean;
  isOrtho: boolean;
  transformSpace: 'world' | 'local';
  snapSettings: SnapSettings;
  panelMode: 'floating' | 'docked';
  transformOpen: boolean;
  visualOpen: boolean;
  tree: TreeNode[];
  stats: StatsResponse | null;
  error: string | null;
  canUndo: boolean;
  canRedo: boolean;
  catalogPanelOpen: boolean;
  domainPanelOpen: Record<string, boolean>;
  creatingExtra: { ifcClass: string; desc: string };
  creationPhase: CreationPhase;
  creationObjectType: ObjectType | null;
  creationPoints: Point3D[];
  creationForm: CreationForm | null;
  creationError: string | null;
};

type AppActions = {
  setSelectedIds: (ids: string[] | ((prev: string[]) => string[])) => void;
  setPrimaryMeshInfo: (info: MeshInfo | null) => void;
  setTransformMode: (mode: TransformMode | ((prev: TransformMode) => TransformMode)) => void;
  setVisualMode: (mode: VisualMode) => void;
  setSection: (section: SectionState | ((prev: SectionState) => SectionState)) => void;
  setMeasureMode: (mode: MeasureMode | ((prev: MeasureMode) => MeasureMode)) => void;
  reloadScene: () => void;
  setGridVisible: (v: boolean | ((prev: boolean) => boolean)) => void;
  setIsOrtho: (v: boolean | ((prev: boolean) => boolean)) => void;
  setTransformSpace: (space: 'world' | 'local') => void;
  setSnapSettings: (settings: SnapSettings | ((prev: SnapSettings) => SnapSettings)) => void;
  setPanelMode: (mode: 'floating' | 'docked') => void;
  setTransformOpen: (v: boolean | ((prev: boolean) => boolean)) => void;
  setVisualOpen: (v: boolean | ((prev: boolean) => boolean)) => void;
  setCatalogPanelOpen: (v: boolean | ((prev: boolean) => boolean)) => void;
  setDomainPanelOpen: (domain: string, v: boolean | ((prev: boolean) => boolean)) => void;
  setCreatingExtra: (extra: { ifcClass: string; desc: string }) => void;
  creationDispatch: (action: CreationAction) => void;
  refresh: () => Promise<void>;
};

const creationStateToApp = (cs: CreationState) => ({
  creationPhase: cs.phase,
  creationObjectType: cs.objectType,
  creationPoints: cs.points,
  creationForm: cs.form,
  creationError: cs.error,
});

export const useAppStore = create<AppState & AppActions>((set, get) => {
  let creationState = initialCreationState;

  return {
    selectedIds: [],
    primaryMeshInfo: null,
    transformMode: 'translate' as TransformMode,
    visualMode: 'shaded' as VisualMode,
    section: { enabled: false, axis: 'x', value: 0 },
    measureMode: 'none' as MeasureMode,
    reloadToken: 0,
    gridVisible: true,
    isOrtho: false,
    transformSpace: 'world',
    panelMode: 'floating',
    transformOpen: false,
    visualOpen: false,
    snapSettings: DEFAULT_SNAP_SETTINGS,
    tree: [],
    stats: null,
    error: null,
    canUndo: false,
    canRedo: false,
    catalogPanelOpen: false,
    domainPanelOpen: {},
    creatingExtra: { ifcClass: '', desc: '' },
    creationPhase: 'idle' as CreationPhase,
    creationObjectType: null,
    creationPoints: [],
    creationForm: null,
    creationError: null,

    setSelectedIds: (ids) => {
      if (typeof ids === 'function') set((s) => ({ selectedIds: (ids as (prev: string[]) => string[])(s.selectedIds) }));
      else set({ selectedIds: ids });
    },
    setPrimaryMeshInfo: (info) => set({ primaryMeshInfo: info }),
    setTransformMode: (mode) => {
      if (typeof mode === 'function') set((s) => ({ transformMode: (mode as (prev: TransformMode) => TransformMode)(s.transformMode) }));
      else set({ transformMode: mode });
    },
    setVisualMode: (mode) => set({ visualMode: mode }),
    setSection: (section) => {
      if (typeof section === 'function') {
        set((s) => ({ section: (section as (prev: SectionState) => SectionState)(s.section) }));
      } else {
        set({ section });
      }
    },
    setMeasureMode: (mode) => {
      if (typeof mode === 'function') set((s) => ({ measureMode: (mode as (prev: MeasureMode) => MeasureMode)(s.measureMode) }));
      else set({ measureMode: mode });
    },
    reloadScene: () => set((s) => ({ reloadToken: s.reloadToken + 1 })),
    setGridVisible: (v) => {
      if (typeof v === 'function') set((s) => ({ gridVisible: (v as (prev: boolean) => boolean)(s.gridVisible) }));
      else set({ gridVisible: v });
    },
    setIsOrtho: (v) => {
      if (typeof v === 'function') set((s) => ({ isOrtho: (v as (prev: boolean) => boolean)(s.isOrtho) }));
      else set({ isOrtho: v });
    },
    setTransformSpace: (space) => set({ transformSpace: space }),
    setSnapSettings: (settings) => {
      if (typeof settings === 'function') set((s) => ({ snapSettings: (settings as (prev: SnapSettings) => SnapSettings)(s.snapSettings) }));
      else set({ snapSettings: settings });
    },
    setPanelMode: (mode) => set({ panelMode: mode }),
    setTransformOpen: (v) => {
      if (typeof v === 'function') set((s) => ({ transformOpen: (v as (prev: boolean) => boolean)(s.transformOpen) }));
      else set({ transformOpen: v });
    },
    setVisualOpen: (v) => {
      if (typeof v === 'function') set((s) => ({ visualOpen: (v as (prev: boolean) => boolean)(s.visualOpen) }));
      else set({ visualOpen: v });
    },
    setCatalogPanelOpen: (v) => {
      if (typeof v === 'function') set((s) => ({ catalogPanelOpen: (v as (prev: boolean) => boolean)(s.catalogPanelOpen) }));
      else set({ catalogPanelOpen: v });
    },
    setDomainPanelOpen: (domain, v) => {
      set((s) => ({
        domainPanelOpen: {
          ...s.domainPanelOpen,
          [domain]: typeof v === 'function' ? (v as (prev: boolean) => boolean)(s.domainPanelOpen[domain] ?? false) : v,
        },
      }));
    },
    setCreatingExtra: (extra) => set({ creatingExtra: extra }),

    creationDispatch: (action) => {
      creationState = creationReducer(creationState, action);
      set(creationStateToApp(creationState));
    },

    refresh: async () => {
      set({ error: null });
      try {
        const [nextStats, nextTree, undoStatus] = await Promise.all([
          api.stats(),
          api.tree(),
          api.undoStatus(),
        ]);
        set({
          stats: nextStats,
          tree: nextTree,
          canUndo: undoStatus.can_undo,
          canRedo: undoStatus.can_redo,
        });
      } catch (err) {
        set({ error: err instanceof Error ? err.message : String(err) });
      }
    },
  };
});