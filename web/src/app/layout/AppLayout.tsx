import { useEffect, useState, useRef, useCallback, useSyncExternalStore, type ReactNode } from 'react';
import { panelRegistry } from '../panels/PanelRegistry';
import { DEFAULT_LAYOUT, loadLayout, saveLayout } from '../../shared/layout';
import { useAppStore } from '../store/appStore';

type AppLayoutProps = {
  toolbar?: ReactNode;
  titleBar?: ReactNode;
  statusBar?: ReactNode;
  leftPanel?: ReactNode;
  rightPanel?: ReactNode;
  bottomPanel?: ReactNode;
  children: ReactNode;
  panelMode?: 'floating' | 'docked';
  onTogglePanelMode?: () => void;
};

const DEFAULT_RIGHT_WIDTH = DEFAULT_LAYOUT.docks.right.width;
const MIN_RIGHT_WIDTH = 260;
const MAX_RIGHT_WIDTH = 600;
const MIN_SUPPORTED_WIDTH = 1024;

function subscribeToRegistry(fn: () => void) {
  return panelRegistry.subscribe(fn);
}

let cachedRightPanels: ReturnType<typeof panelRegistry.getForPosition> | null = null;

function getVisibleRightPanels() {
  const result = panelRegistry.getForPosition('right');
  if (cachedRightPanels && cachedRightPanels.length === result.length && cachedRightPanels.every((p, i) => p.id === result[i].id)) {
    return cachedRightPanels;
  }
  cachedRightPanels = result;
  return result;
}

export function AppLayout({ toolbar, titleBar, statusBar, leftPanel, rightPanel, bottomPanel, children, panelMode = 'docked' }: AppLayoutProps) {
  const initial = useRef(loadLayout()).current;
  const [rightWidth, setRightWidth] = useState(initial.docks.right.width);
  const [tooNarrow, setTooNarrow] = useState(
    typeof window !== 'undefined' && window.innerWidth < MIN_SUPPORTED_WIDTH,
  );
  const dragRef = useRef<{ startX: number; startWidth: number } | null>(null);
  const rightWidthRef = useRef(rightWidth);
  rightWidthRef.current = rightWidth;
  const visibleRightPanels = useSyncExternalStore(subscribeToRegistry, getVisibleRightPanels, getVisibleRightPanels);
  const layoutResetToken = useAppStore((s) => s.layoutResetToken);

  const rightPanelVisible = rightPanel !== null && panelMode !== undefined && visibleRightPanels.length > 0;

  const persistWidth = useCallback((width: number) => {
    const layout = loadLayout();
    saveLayout({ ...layout, docks: { ...layout.docks, right: { ...layout.docks.right, width, mode: panelMode === 'floating' ? 'floating' : 'docked' } } });
  }, [panelMode]);

  // Reset layout (View → Panels → Reset) restores the documented defaults.
  useEffect(() => {
    if (layoutResetToken === 0) return;
    setRightWidth(DEFAULT_RIGHT_WIDTH);
    persistWidth(DEFAULT_RIGHT_WIDTH);
  }, [layoutResetToken, persistWidth]);

  useEffect(() => {
    const onResize = () => setTooNarrow(window.innerWidth < MIN_SUPPORTED_WIDTH);
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  const onDragStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragRef.current = { startX: e.clientX, startWidth: rightWidthRef.current };
    const onMove = (ev: MouseEvent) => {
      if (!dragRef.current) return;
      const newWidth = Math.max(MIN_RIGHT_WIDTH, Math.min(MAX_RIGHT_WIDTH, dragRef.current.startWidth + (dragRef.current.startX - ev.clientX)));
      setRightWidth(newWidth);
    };
    const onUp = () => {
      dragRef.current = null;
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      persistWidth(rightWidthRef.current);
    };
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }, [persistWidth]);

  return (
    <div className="app-shell">
      {leftPanel && (
        <aside className="sidebar">
          {leftPanel}
        </aside>
      )}
      <section className="viewer-panel" data-panel-visible={rightPanelVisible && panelMode === 'docked' ? 'true' : 'false'}>
        {titleBar}
        {toolbar}
        <div className="viewer-body" data-panel-mode={rightPanelVisible && panelMode === 'docked' ? 'docked' : undefined}>
          <div className="viewer-content">
            {children}
            {bottomPanel && (
              <div className="bottom-panel">
                {bottomPanel}
              </div>
            )}
          </div>
          {rightPanelVisible && panelMode === 'docked' && (
            <>
              <div className="panel-resize-handle" onMouseDown={onDragStart} />
              <div className="right-panel" style={{ width: rightWidth }}>{rightPanel}</div>
            </>
          )}
        </div>
        {rightPanelVisible && panelMode === 'floating' && (
          <div className="floating-panel-overlay">
            <div className="floating-panel">{rightPanel}</div>
          </div>
        )}
      </section>
      {statusBar}
      {tooNarrow && (
        <div className="min-width-notice" role="alertdialog" aria-modal="true">
          <div>
            <h2>Window too small</h2>
            <p>OmniM needs at least 1024 px of width. Resize the window to continue.</p>
          </div>
        </div>
      )}
    </div>
  );
}
