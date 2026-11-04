import { lazy, Suspense, useEffect, useMemo, useRef, useCallback, useState } from 'react';
import { Routes, Route, useNavigate, useLocation } from 'react-router-dom';
import { api } from '../shared/api';
import { MainToolbar } from './components/MainToolbar';
import { SectionPanel } from './components/SectionPanel';
import { StoreyViewSelect } from './components/StoreyViewSelect';
import { ProjectsPage } from './ProjectsPage';
import { LeftDock } from './layout/LeftDock';
import { TitleBar } from './components/TitleBar';
import { StatusBar } from './components/StatusBar';
import { ErrorBoundary } from './components/ErrorBoundary';
import { useConfirm } from './components/ConfirmDialog';
import { shouldWarnBeforeUnload } from '../shared/saveState';
import { analyzeDeleteImpact, formatDeleteImpact } from '../shared/deleteImpact';
import type { ThreeViewerHandle } from './components/ThreeViewer';
// PRF-01/06 (PERFORMANCE §3/§6): three.js dominates the initial chunk; the
// shell mounts first and the viewport loads asynchronously.
const ThreeViewer = lazy(() =>
  import('./components/ThreeViewer').then((m) => ({ default: m.ThreeViewer })),
);
import { ViewCube3D } from './components/ViewCube3D';
import { ToastContainer } from './components/ToastContainer';
import { IcoMove, IcoRotate, IcoShaded, IcoWireframe, IcoHiddenLine, IcoRealistic, IcoSection, IcoFit, IcoHome, IcoGrid, IcoLocal, IcoOrtho, IcoMeasureDist, IcoMeasureAngle, IcoMeasureArea, IcoXray, IcoSnap } from './components/Icons';
import { useToastApi } from '../shared/toast';
const ComponentEditorPage = lazy(() => import('../sketch/ComponentEditorPage').then((m) => ({ default: m.ComponentEditorPage })));
const SeriesManagerPage = lazy(() => import('../series/SeriesManagerPage').then((mod) => ({ default: mod.SeriesManagerPage })));
const LibraryManagerPage = lazy(() => import('../library/LibraryManagerPage').then((mod) => ({ default: mod.LibraryManagerPage })));
import { useAppStore } from './store/appStore';
import { pickModeForForm, applyPointsToForm, createInitialForm } from '../shared/types/creation';
import { registerDefaultPanels } from './panels/defaultPanels';
import { AppLayout } from './layout/AppLayout';
import { PanelSlot } from './panels/PanelRegistry';
import { ThreeViewerContext } from './context/ThreeViewerContext';
import type { ReactNode } from 'react';
import { applyTheme } from '../shared/theme';
import { CommandPalette, type PaletteCommand } from './components/CommandPalette';
import { ShortcutHelp } from './components/ShortcutHelp';

const FOCUS_REGIONS = [
  '.main-toolbar',
  '.dock-tabs',
  '#left-dock-panel',
  '.three-viewer',
  '.properties-panel',
  '.omni-panel',
  '.status-bar',
];

function firstFocusable(region: Element): HTMLElement | null {
  return (
    Array.from(
      region.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      ),
    ).find((el) => !el.hasAttribute('disabled') && el.getClientRects().length > 0) ?? null
  );
}

/** A11Y §4: F6 / Shift+F6 cycle the seven shell regions. */
function focusNextRegion(delta: 1 | -1) {
  const active = document.activeElement;
  let index = -1;
  FOCUS_REGIONS.forEach((selector, i) => {
    const region = document.querySelector(selector);
    if (region && active && region.contains(active)) index = i;
  });
  for (let step = 1; step <= FOCUS_REGIONS.length; step += 1) {
    const i = (index + delta * step + FOCUS_REGIONS.length * 2) % FOCUS_REGIONS.length;
    const region = document.querySelector(FOCUS_REGIONS[i]);
    const target = region ? firstFocusable(region) : null;
    if (target) {
      target.focus();
      return;
    }
  }
}

/** Dev/e2e aid (UI-FBK-06): workspace route crash for boundary tests. */
function CrashRoutePanel(): ReactNode {
  throw new Error('crash-test fixture: route threw while rendering');
}

export function App() {
  const toast = useToastApi();
  const navigate = useNavigate();
  const location = useLocation();
  const isMain = location.pathname === '/' || location.pathname === '';
  const crashRoute =
    isMain &&
    typeof window !== 'undefined' &&
    window.sessionStorage.getItem('omnim.crashTest') === 'route';
  const viewerRef = useRef<ThreeViewerHandle>(null);

  const {
    selectedIds, tree, stats, error, canUndo, canRedo,
    transformMode, visualMode, section, measureMode,
    transformOpen, visualOpen, panelMode,
    gridVisible, isOrtho, transformSpace, reloadToken,
    creationPhase, creationForm, creationPoints,
    setSelectedIds, setTransformMode, setVisualMode,
    setSection, setMeasureMode, setPanelMode,
    setTransformOpen, setVisualOpen,
    setGridVisible, setIsOrtho, setTransformSpace,
    snapSettings, setSnapSettings,
    creationDispatch,
    refresh, reloadScene, markDirty, saveProject, saveState,
    setCursorWorld, setFps,
    theme,
  } = useAppStore();
  const confirm = useConfirm();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [helpOpen, setHelpOpen] = useState(false);

  async function deleteSelected() {
    if (selectedIds.length === 0) return;
    // INTERACTION §7.3: state scope, child impact and undo before deleting.
    const impact = analyzeDeleteImpact(tree, selectedIds);
    const ok = await confirm({
      title: selectedIds.length === 1 ? 'Delete object?' : `Delete ${selectedIds.length} objects?`,
      message: formatDeleteImpact(impact),
      confirmLabel: 'Delete',
      danger: true,
    });
    if (!ok) return;
    for (const id of selectedIds) {
      try {
        await api.deleteObject(id);
      } catch (err) {
        toast.error(`Delete failed for ${id}: ${err instanceof Error ? err.message : String(err)}`);
        return;
      }
    }
    setSelectedIds([]);
    markDirty();
    toast.success(selectedIds.length === 1 ? 'Object deleted' : `${selectedIds.length} objects deleted`);
    refresh();
    reloadScene();
  }

  useEffect(() => {
    if (!shouldWarnBeforeUnload(saveState)) return;
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = '';
    };
    window.addEventListener('beforeunload', onBeforeUnload);
    return () => window.removeEventListener('beforeunload', onBeforeUnload);
  }, [saveState.status]);

  useEffect(() => { registerDefaultPanels(); }, []);
  useEffect(() => { applyTheme(theme); }, [theme]);
  useEffect(() => {
    // UI-DAT-01: the workspace is only reachable after an explicit open/new.
    // The server tracks whether this session has one; a restart lands here.
    if (!isMain) return;
    let cancelled = false;
    api.currentProject()
      .then((session) => {
        if (!cancelled && !session.open) navigate('/projects', { replace: true });
      })
      .catch(() => { /* the workspace will surface the connection error */ });
    return () => { cancelled = true; };
  }, [isMain, navigate]);
  useEffect(() => { refresh(); reloadScene(); }, []);
  useEffect(() => { if (isMain) { refresh(); reloadScene(); } }, [location.pathname]);

  const previousPathRef = useRef(location.pathname);
  useEffect(() => {
    if (previousPathRef.current === location.pathname) return;
    previousPathRef.current = location.pathname;
    // A11Y §4: a route change moves focus to the page heading.
    requestAnimationFrame(() => {
      // Never steal focus from an error fallback that is already announcing.
      if (document.querySelector('.error-boundary')) return;
      const target = document.querySelector<HTMLElement>('[data-page-heading], h1');
      if (!target) return;
      if (!target.hasAttribute('tabindex')) target.setAttribute('tabindex', '-1');
      target.focus();
    });
  }, [location.pathname]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setPaletteOpen(true);
        return;
      }
      if (event.key === 'F1') {
        event.preventDefault();
        setHelpOpen(true);
        return;
      }
      if (event.key === 'F6') {
        event.preventDefault();
        focusNextRegion(event.shiftKey ? -1 : 1);
        return;
      }
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z') {
        event.preventDefault();
        if (event.shiftKey) void redoCommand();
        else void undoCommand();
        return;
      }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'y') {
        event.preventDefault();
        void redoCommand();
        return;
      }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
        event.preventDefault();
        void saveProject();
        return;
      }
      switch (event.key.toLowerCase()) {
        case 'w': setTransformMode((prev) => prev === 'translate' ? null : 'translate'); break;
        case 'e': setTransformMode((prev) => prev === 'rotate' ? null : 'rotate'); break;
        case 'f': viewerRef.current?.fit(); break;
        case 'h': viewerRef.current?.home(); break;
        case 'g':
          viewerRef.current?.toggleGrid();
          setGridVisible((v) => !v);
          break;
        case 'o':
          viewerRef.current?.toggleOrtho();
          setIsOrtho((v) => !v);
          break;
        case 'x':
          viewerRef.current?.setTransformSpace(transformSpace === 'world' ? 'local' : 'world');
          setTransformSpace(transformSpace === 'world' ? 'local' : 'world');
          break;
        case 'delete':
        case 'backspace':
          // While picking, backspace is an undo of the last picked point,
          // not a delete of scene objects.
          if (event.key.toLowerCase() === 'backspace' && creationPhase === 'picking') {
            creationDispatch({ type: 'removeLastPoint' });
            break;
          }
          void deleteSelected().catch((err) => toast.error(err instanceof Error ? err.message : String(err)));
          break;
        case 'escape':
          if (creationPhase === 'picking' || creationPhase === 'configuring' || creationPhase === 'error') {
            creationDispatch({ type: 'cancelCreation' });
          } else {
            setSelectedIds([]);
          }
          break;
        case 'enter':
          if (creationPhase === 'picking' && creationForm) {
            const mode = pickModeForForm(creationForm);
            if ((mode.mode === 'polyline' || mode.mode === 'polygon') && creationPoints.length >= (mode.mode === 'polygon' ? 3 : 2)) {
              const form = applyPointsToForm(creationForm, creationPoints);
              creationDispatch({ type: 'completePicking', form });
            }
          }
          break;
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedIds, transformSpace, creationPhase, creationPoints, creationForm]);

  const transformRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!transformOpen) return;
    const handler = (e: MouseEvent) => {
      if (transformRef.current && !transformRef.current.contains(e.target as Node)) {
        setTransformOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [transformOpen]);

  const visualRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!visualOpen) return;
    const handler = (e: MouseEvent) => {
      if (visualRef.current && !visualRef.current.contains(e.target as Node)) {
        setVisualOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [visualOpen]);

  const undoCommand = async () => {
    await api.undo();
    markDirty();
    refresh();
    reloadScene();
  };
  const redoCommand = async () => {
    await api.redo();
    markDirty();
    refresh();
    reloadScene();
  };

  const commands: PaletteCommand[] = [
    { id: 'file.save', title: 'Save project', shortcut: 'Ctrl/Cmd+S', scope: 'Global', run: () => void saveProject() },
    { id: 'edit.undo', title: 'Undo', shortcut: 'Ctrl/Cmd+Z', scope: 'Global', disabled: !canUndo, run: () => void undoCommand() },
    { id: 'edit.redo', title: 'Redo', shortcut: 'Ctrl/Cmd+Shift+Z', scope: 'Global', disabled: !canRedo, run: () => void redoCommand() },
    { id: 'edit.delete', title: 'Delete selected objects', shortcut: 'Delete', scope: 'Global', disabled: selectedIds.length === 0, run: () => void deleteSelected() },
    { id: 'view.fit', title: 'Fit view', shortcut: 'F', scope: 'Viewport', run: () => viewerRef.current?.fit() },
    { id: 'view.home', title: 'Home view', shortcut: 'H', scope: 'Viewport', run: () => viewerRef.current?.home() },
    { id: 'view.grid', title: 'Toggle grid', shortcut: 'G', scope: 'Viewport', run: () => { setGridVisible((v) => !v); viewerRef.current?.toggleGrid(); } },
    { id: 'view.ortho', title: 'Toggle orthographic / perspective', shortcut: 'O', scope: 'Viewport', run: () => { setIsOrtho((v) => !v); viewerRef.current?.toggleOrtho(); } },
    { id: 'view.space', title: 'Toggle local / world transform space', shortcut: 'X', scope: 'Viewport', run: () => { const next = transformSpace === 'world' ? 'local' : 'world'; setTransformSpace(next); viewerRef.current?.setTransformSpace(next); } },
    { id: 'transform.move', title: 'Move mode', shortcut: 'W', scope: 'Viewport', run: () => setTransformMode((prev) => prev === 'translate' ? null : 'translate') },
    { id: 'transform.rotate', title: 'Rotate mode', shortcut: 'E', scope: 'Viewport', run: () => setTransformMode((prev) => prev === 'rotate' ? null : 'rotate') },
    { id: 'create.storey', title: 'Create storey', scope: 'Create', run: () => creationDispatch({ type: 'completePicking', form: createInitialForm('storey', []) }) },
    { id: 'create.door', title: 'Create door', scope: 'Create', run: () => creationDispatch({ type: 'completePicking', form: createInitialForm('door', []) }) },
    { id: 'nav.projects', title: 'Go to projects', scope: 'Navigation', run: () => navigate('/projects') },
    { id: 'nav.series', title: 'Open Series Manager', scope: 'Navigation', run: () => navigate('/series-manager') },
    { id: 'nav.library', title: 'Open Library Manager', scope: 'Navigation', run: () => navigate('/library-manager') },
    { id: 'help.shortcuts', title: 'Keyboard shortcuts', shortcut: 'F1', scope: 'Help', run: () => setHelpOpen(true) },
  ];

  return (
    <ErrorBoundary scope="Workspace" resetKey={location.pathname}>
    <Routes>
      <Route path="/projects" element={<ProjectsPage />} />
      <Route path="/component-editor" element={<Suspense fallback={null}><ComponentEditorPage /></Suspense>} />
      <Route path="/series-manager" element={<Suspense fallback={null}><SeriesManagerPage /></Suspense>} />
      <Route path="/library-manager" element={<Suspense fallback={null}><LibraryManagerPage /></Suspense>} />
      <Route path="*" element={
        <ThreeViewerContext.Provider value={viewerRef}>
          {/* Dev/e2e aid (UI-FBK-06): throw inside the workspace route to
              exercise its error boundary without touching AppLayout. */}
          {crashRoute && <CrashRoutePanel />}
          <AppLayout
            titleBar={<TitleBar />}
            statusBar={<StatusBar />}
            toolbar={<div className="modeling-bar">
              <MainToolbar
                selectedIds={selectedIds}
                canUndo={canUndo}
                canRedo={canRedo}
                onRefresh={refresh}
                onDelete={() => void deleteSelected().catch((err) => toast.error(err instanceof Error ? err.message : String(err)))}
                onCreated={() => { markDirty(); refresh(); reloadScene(); }}
                onImported={() => { markDirty(); refresh(); reloadScene(); }}
                onEditComponent={(componentId) => navigate(componentId ? `/component-editor?component=${componentId}` : '/component-editor')}
                selectedParentId={selectedIds.length === 1 ? selectedIds[0] : undefined}
                tree={tree}
              />
            </div>}
            leftPanel={<LeftDock />}
            rightPanel={<PanelSlot position="right" />}
            bottomPanel={<PanelSlot position="bottom" />}
            panelMode={panelMode}
            onTogglePanelMode={() => setPanelMode(panelMode === 'floating' ? 'docked' : 'floating')}
          >
            <div className="viewport-area">
              <div className="viewport-storey-control" title="Storey view">
                <StoreyViewSelect />
              </div>
              {/* Fallback mirrors the viewer's application landmass so the
                  a11y shell contract (named regions) holds during the async
                  chunk load; PRF-01/06. */}
              <Suspense
                fallback={
                  <div
                    role="application"
                    aria-label="3D viewport"
                    className="viewport-loading"
                  >
                    <span role="status" aria-live="polite">Loading viewport…</span>
                  </div>
                }
              >
              <ThreeViewer
                ref={viewerRef}
                selectedIds={selectedIds}
                reloadToken={reloadToken}
                transformMode={transformMode}
                visualMode={visualMode}
                section={section}
                measureMode={measureMode}
                onSelectionChanged={(ids) => {
                  setSelectedIds(ids);
                }}
                onChanged={() => { markDirty(); refresh(); reloadScene(); }}
                onPointerWorld={setCursorWorld}
                onStats={setFps}
              />
              </Suspense>
              {section.enabled && <SectionPanel section={section} onChange={setSection} />}
              <ViewCube3D viewer={viewerRef} />
              <div className="view-toolbar-vertical">
                <div className="transform-group" ref={transformRef}>
                  <button className={transformMode ? 'active' : ''} onClick={() => setTransformOpen((v) => !v)} title={transformMode === 'translate' ? 'Move (W)' : transformMode === 'rotate' ? 'Rotate (E)' : 'Transform'} aria-label={transformMode === 'translate' ? 'Move (W)' : transformMode === 'rotate' ? 'Rotate (E)' : 'Transform'}>
                    {transformMode === 'translate' ? <IcoMove /> : transformMode === 'rotate' ? <IcoRotate /> : <IcoMove />}
                  </button>
                  {transformOpen && (
                    <div className="transform-popover">
                      <button className={transformMode === 'translate' ? 'active' : ''} onClick={() => { setTransformMode(transformMode === 'translate' ? null : 'translate'); setTransformOpen(false); }} title="Move (W)"><IcoMove /></button>
                      <button className={transformMode === 'rotate' ? 'active' : ''} onClick={() => { setTransformMode(transformMode === 'rotate' ? null : 'rotate'); setTransformOpen(false); }} title="Rotate (E)"><IcoRotate /></button>
                    </div>
                  )}
                </div>
                <span className="separator" />
                <div className="transform-group" ref={visualRef}>
                  <button className={visualMode !== 'shaded' ? 'active' : ''} onClick={() => setVisualOpen((v) => !v)} title={{ shaded: 'Shaded', wireframe: 'Wireframe', 'hidden-line': 'Hidden Line', realistic: 'Realistic', 'x-ray': 'X-Ray' }[visualMode]}>
                    {{ shaded: <IcoShaded />, wireframe: <IcoWireframe />, 'hidden-line': <IcoHiddenLine />, realistic: <IcoRealistic />, 'x-ray': <IcoXray /> }[visualMode]}
                  </button>
                  {visualOpen && (
                    <div className="transform-popover">
                      <button className={visualMode === 'shaded' ? 'active' : ''} onClick={() => { setVisualMode('shaded'); setVisualOpen(false); }} title="Shaded"><IcoShaded /></button>
                      <button className={visualMode === 'wireframe' ? 'active' : ''} onClick={() => { setVisualMode('wireframe'); setVisualOpen(false); }} title="Wireframe"><IcoWireframe /></button>
                      <button className={visualMode === 'hidden-line' ? 'active' : ''} onClick={() => { setVisualMode('hidden-line'); setVisualOpen(false); }} title="Hidden Line"><IcoHiddenLine /></button>
                      <button className={visualMode === 'realistic' ? 'active' : ''} onClick={() => { setVisualMode('realistic'); setVisualOpen(false); }} title="Realistic"><IcoRealistic /></button>
                      <button className={visualMode === 'x-ray' ? 'active' : ''} onClick={() => { setVisualMode('x-ray'); setVisualOpen(false); }} title="X-Ray"><IcoXray /></button>
                    </div>
                  )}
                </div>
                <span className="separator" />
                <button className={measureMode === 'distance' ? 'active' : ''} onClick={() => { setMeasureMode((prev) => prev === 'distance' ? 'none' : 'distance'); viewerRef.current?.clearMeasurements(); }} title="Measure distance"><IcoMeasureDist /></button>
                <button className={measureMode === 'angle' ? 'active' : ''} onClick={() => { setMeasureMode((prev) => prev === 'angle' ? 'none' : 'angle'); viewerRef.current?.clearMeasurements(); }} title="Measure angle"><IcoMeasureAngle /></button>
                <button className={measureMode === 'area' ? 'active' : ''} onClick={() => { setMeasureMode((prev) => prev === 'area' ? 'none' : 'area'); viewerRef.current?.clearMeasurements(); }} title="Measure area (right-click to finish)"><IcoMeasureArea /></button>
                <span className="separator" />
                <button className={section.enabled ? 'active' : ''} onClick={() => setSection((v) => ({ ...v, enabled: !v.enabled }))} title="Section cut"><IcoSection /></button>
                <button onClick={() => viewerRef.current?.fit()} title="Fit view (F)"><IcoFit /></button>
                <button onClick={() => { viewerRef.current?.home(); }} title="Home view (H)"><IcoHome /></button>
                <span className="separator" />
                <button className={!gridVisible ? 'active' : ''} onClick={() => { setGridVisible((v) => !v); viewerRef.current?.toggleGrid(); }} title="Toggle grid (G)"><IcoGrid /></button>
                <button className={snapSettings.enabled ? 'active' : ''} onClick={() => setSnapSettings((prev) => ({ ...prev, enabled: !prev.enabled }))} title="Toggle snap"><IcoSnap /></button>
                <button className={isOrtho ? 'active' : ''} onClick={() => { setIsOrtho((v) => !v); viewerRef.current?.toggleOrtho(); }} title="Toggle ortho/perspective (O)"><IcoOrtho /></button>
                <button className={transformSpace === 'local' ? 'active' : ''} onClick={() => { const next = transformSpace === 'world' ? 'local' : 'world'; setTransformSpace(next); viewerRef.current?.setTransformSpace(next); }} title={`Transform space: ${transformSpace} (X)`}><IcoLocal /></button>
              </div>
              <div className="hint" title="W move · E rotate · F fit · H home · G grid · O ortho · X axis · Del delete · Click select · Ctrl+click toggle · Ctrl+drag box-select">W/E transform · F fit · H home · G grid · O ortho · Del delete · Ctrl+drag box select</div>
            </div>
          </AppLayout>
        </ThreeViewerContext.Provider>
      } />
    </Routes>
    <ToastContainer />
    <CommandPalette open={paletteOpen} commands={commands} onClose={() => setPaletteOpen(false)} />
    <ShortcutHelp open={helpOpen} commands={commands} onClose={() => setHelpOpen(false)} />
    </ErrorBoundary>
  );
}