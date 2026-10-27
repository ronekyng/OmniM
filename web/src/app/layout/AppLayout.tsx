import { useState, useRef, useCallback, useSyncExternalStore } from 'react';
import type { ReactNode } from 'react';
import { panelRegistry } from '../panels/PanelRegistry';

type AppLayoutProps = {
  toolbar?: ReactNode;
  leftPanel?: ReactNode;
  rightPanel?: ReactNode;
  bottomPanel?: ReactNode;
  children: ReactNode;
  panelMode?: 'floating' | 'docked';
  onTogglePanelMode?: () => void;
};

const DEFAULT_RIGHT_WIDTH = 340;
const MIN_RIGHT_WIDTH = 260;
const MAX_RIGHT_WIDTH = 600;

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

export function AppLayout({ toolbar, leftPanel, rightPanel, bottomPanel, children, panelMode = 'floating', onTogglePanelMode }: AppLayoutProps) {
  const [rightWidth, setRightWidth] = useState(DEFAULT_RIGHT_WIDTH);
  const dragRef = useRef<{ startX: number; startWidth: number } | null>(null);
  const visibleRightPanels = useSyncExternalStore(subscribeToRegistry, getVisibleRightPanels, getVisibleRightPanels);

  const rightPanelVisible = rightPanel !== null && panelMode !== undefined && visibleRightPanels.length > 0;

  const onDragStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    dragRef.current = { startX: e.clientX, startWidth: rightWidth };
    const onMove = (ev: MouseEvent) => {
      if (!dragRef.current) return;
      const newWidth = Math.max(MIN_RIGHT_WIDTH, Math.min(MAX_RIGHT_WIDTH, dragRef.current.startWidth + (dragRef.current.startX - ev.clientX)));
      setRightWidth(newWidth);
    };
    const onUp = () => {
      dragRef.current = null;
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    };
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }, [rightWidth]);

  return (
    <div className="app-shell">
      {leftPanel && (
        <aside className="sidebar">
          {leftPanel}
        </aside>
      )}
      <section className="viewer-panel" data-panel-visible={rightPanelVisible && panelMode === 'docked' ? 'true' : 'false'}>
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
    </div>
  );
}