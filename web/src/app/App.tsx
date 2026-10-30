import { useEffect, useRef, useCallback } from 'react';
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
import { ThreeViewer, type ThreeViewerHandle } from './components/ThreeViewer';
import { ViewCube3D } from './components/ViewCube3D';
import { ToastContainer } from './components/ToastContainer';
import { IcoMove, IcoRotate, IcoShaded, IcoWireframe, IcoHiddenLine, IcoRealistic, IcoSection, IcoFit, IcoHome, IcoGrid, IcoLocal, IcoOrtho, IcoMeasureDist, IcoMeasureAngle, IcoMeasureArea, IcoXray, IcoSnap } from './components/Icons';
import { useToastApi } from '../shared/toast';
import { ComponentEditorPage } from '../sketch/ComponentEditorPage';
import { SeriesManagerPage } from '../series/SeriesManagerPage';
import { LibraryManagerPage } from '../library/LibraryManagerPage';
import { useAppStore } from './store/appStore';
import { pickModeForForm, applyPointsToForm } from '../shared/types/creation';
import { registerDefaultPanels } from './panels/defaultPanels';
import { AppLayout } from './layout/AppLayout';
import { PanelSlot } from './panels/PanelRegistry';
import { ThreeViewerContext } from './context/ThreeViewerContext';

export function App() {
  const toast = useToastApi();
  const navigate = useNavigate();
  const location = useLocation();
  const isMain = location.pathname === '/' || location.pathname === '';
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
  } = useAppStore();
  const confirm = useConfirm();

  async function deleteSelected() {
    if (selectedIds.length === 0) return;
    const msg = selectedIds.length === 1
      ? 'This removes the object from the project.'
      : `This removes ${selectedIds.length} objects from the project.`;
    const ok = await confirm({
      title: selectedIds.length === 1 ? 'Delete object?' : `Delete ${selectedIds.length} objects?`,
      message: msg,
      confirmLabel: 'Delete',
      danger: true,
    });
    if (!ok) return;
    for (const id of selectedIds) {
      const response = await api.deleteObject(id);
      if (!response.ok) {
        toast.error(`Delete failed for ${id}: ${response.status} ${response.statusText}`);
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

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) return;
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

  return (
    <ErrorBoundary scope="Workspace" resetKey={location.pathname}>
    <Routes>
      <Route path="/projects" element={<ProjectsPage />} />
      <Route path="/component-editor" element={<ComponentEditorPage />} />
      <Route path="/series-manager" element={<SeriesManagerPage />} />
      <Route path="/library-manager" element={<LibraryManagerPage />} />
      <Route path="*" element={
        <ThreeViewerContext.Provider value={viewerRef}>
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
              {section.enabled && <SectionPanel section={section} onChange={setSection} />}
              <ViewCube3D viewer={viewerRef} />
              <div className="view-toolbar-vertical">
                <div className="transform-group" ref={transformRef}>
                  <button className={transformMode ? 'active' : ''} onClick={() => setTransformOpen((v) => !v)} title={transformMode === 'translate' ? 'Move (W)' : transformMode === 'rotate' ? 'Rotate (E)' : ''}>
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
              <div className="hint">W:move E:rotate F:fit H:home G:grid O:ortho X:axis Del:delete | Click:select Ctrl+click:toggle Ctrl+drag:box-select</div>
            </div>
          </AppLayout>
        </ThreeViewerContext.Provider>
      } />
    </Routes>
    <ToastContainer />
    </ErrorBoundary>
  );
}