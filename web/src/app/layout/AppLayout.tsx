import { useEffect, useState, useRef, useCallback, useSyncExternalStore, type ReactNode } from 'react';
import { panelRegistry } from '../panels/PanelRegistry';
import {
  DEFAULT_LAYOUT,
  FLOATING_EDGE_MARGIN,
  clampFloating,
  loadLayout,
  saveLayout,
  type FloatingWindow,
} from '../../shared/layout';
import { useI18n } from '../../shared/i18n';
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

/** Default floating box: a right-side window that respects the 32 px margin. */
function FALLING_DEFAULT(): { x: number; y: number; w: number; h: number } {
  if (typeof window === 'undefined') return { x: 640, y: 120, w: 420, h: 560 };
  const w = Math.min(420, window.innerWidth - 2 * 32);
  const h = Math.min(560, window.innerHeight - 2 * 32);
  return {
    x: window.innerWidth - w - 32,
    y: Math.max(32, Math.round((window.innerHeight - h) / 2)),
    w: Math.max(260, w),
    h: Math.max(180, h),
  };
}

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
  const { t } = useI18n();
  const initial = useRef(loadLayout()).current;
  const [rightWidth, setRightWidth] = useState(initial.docks.right.width);
  // SHL-09 §4.4: the floating window box (per panel id) persists with the
  // layout; the handle wires drag, the corner wires resize, both clamped.
  const [floatingBox, setFloatingBox] = useState<FloatingWindow | null>(() => loadLayout().floating[0] ?? null);
  const dragStateRef = useRef<{ kind: 'move' | 'resize'; dx: number; dy: number; box: FloatingWindow } | null>(null);
  const persistFloating = useCallback((box: FloatingWindow) => {
    const layout = loadLayout();
    saveLayout({ ...layout, floating: [box] });
  }, []);
  const [tooNarrow, setTooNarrow] = useState(
    typeof window !== 'undefined' && window.innerWidth < MIN_SUPPORTED_WIDTH,
  );
  // SHL-11 (IA_SHELL §7): 1024–1279 px folds both docks into 44 px icon
  // strips; the right dock re-opens through a drawer. ↝ band, not <1024
  // (the blocking notice keeps precedence).
  const [narrow, setNarrow] = useState(
    typeof window !== 'undefined' &&
      window.innerWidth >= MIN_SUPPORTED_WIDTH && window.innerWidth < 1280,
  );
  const [drawerOpen, setDrawerOpen] = useState(false);
  const sidebarRef = useRef<HTMLElement | null>(null);
  // SHL-11: the left icons re-open their tab panel in a drawer too.
  const [leftDrawerOpen, setLeftDrawerOpen] = useState(false);
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
    const onResize = () => {
      const w = window.innerWidth;
      setTooNarrow(w < MIN_SUPPORTED_WIDTH);
      setNarrow(w >= MIN_SUPPORTED_WIDTH && w < 1280);
      if (w >= 1280) {
        setDrawerOpen(false);
        setLeftDrawerOpen(false);
      }
    };
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, []);

  const beginFloatingDrag = (e: React.MouseEvent, kind: 'move' | 'resize') => {
    e.preventDefault();
    e.stopPropagation();
    const start = floatingBox ?? FALLING_DEFAULT();
    const box = clampFloating(start);
    setFloatingBox(box);
    dragStateRef.current = {
      kind,
      dx: e.clientX - (kind === 'move' ? box.x : box.x + box.w),
      dy: e.clientY - (kind === 'move' ? box.y : box.y + box.h),
      box,
    };
    const onMove = (ev: MouseEvent) => {
      const st = dragStateRef.current;
      if (!st) return;
      const clamped = clampFloating(
        st.kind === 'move'
          ? { ...st.box, x: ev.clientX - st.dx, y: ev.clientY - st.dy }
          : {
              ...st.box,
              w: Math.max(260, ev.clientX - st.box.x),
              h: Math.max(180, ev.clientY - st.box.y),
            },
      );
      setFloatingBox(clamped);
    };
    const onUp = () => {
      dragStateRef.current = null;
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
      setFloatingBox((box) => {
        if (box) persistFloating(box);
        return box;
      });
    };
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  };

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

  // SHL-11: in the narrow band a left-dock tab click opens/closes the left
  // drawer (capture listener so it preempts the internal tab switch).
  useEffect(() => {
    if (!narrow) {
      return undefined;
    }
    const aside = sidebarRef.current;
    if (!aside) return undefined;
    const capture = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      if (target?.closest('.dock-tabs button')) {
        e.preventDefault();
        e.stopPropagation();
        setLeftDrawerOpen((v) => !v);
      }
    };
    aside.addEventListener('click', capture, true);
    return () => aside.removeEventListener('click', capture, true);
  }, [narrow]);

  return (
    <div className={`app-shell${narrow ? ' narrow-shell' : ''}`}>
      {leftPanel && (
        <aside
          className="sidebar"
          ref={sidebarRef}
          data-left-drawer-open={narrow && leftDrawerOpen ? 'true' : undefined}
        >
          {leftPanel}
        </aside>
      )}
      {narrow && leftPanel && leftDrawerOpen && (
        <aside className="right-drawer left-anchor" data-testid="left-canvas-drawer">
          {leftPanel}
        </aside>
      )}
      {narrow && rightPanelVisible && (
        <nav className="right-icon-strip" aria-label="Right dock panels">
          {visibleRightPanels.map((panel) => (
            <button
              key={panel.id}
              type="button"
              title={panel.titleKey ? t(panel.titleKey as never) : (panel.title ?? panel.id)}
              aria-label={panel.titleKey ? t(panel.titleKey as never) : (panel.title ?? panel.id)}
              aria-expanded={drawerOpen}
              className={drawerOpen ? 'strip-icon active' : 'strip-icon'}
              onClick={() => setDrawerOpen((v) => !v)}
            >
              <span aria-hidden="true">{(panel.titleKey ? t(panel.titleKey as never) : (panel.title ?? panel.id)).slice(0, 1)}</span>
            </button>
          ))}
        </nav>
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
          {narrow && rightPanelVisible && drawerOpen && (
            <aside className="right-drawer" data-testid="right-drawer">
              {rightPanel}
            </aside>
          )}
          {!narrow && rightPanelVisible && panelMode === 'docked' && (
            <>
              {/* INTERACTION §7.4: divider double-click resets the width. */}
              <div
                className="panel-resize-handle"
                onMouseDown={onDragStart}
                onDoubleClick={() => {
                  setRightWidth(DEFAULT_LAYOUT.docks.right.width);
                  persistWidth(DEFAULT_LAYOUT.docks.right.width);
                }}
                title="Double-click resets the width"
              />
              <div className="right-panel" style={{ width: rightWidth }}>{rightPanel}</div>
            </>
          )}
        </div>
        {rightPanelVisible && panelMode === 'floating' && (
          <div className="floating-panel-overlay">
            {(() => {
              const box = floatingBox
                ? clampFloating(floatingBox)
                : (() => {
                    const d = FALLING_DEFAULT();
                    const c = clampFloating(d);
                    return c;
                  })();
              const uid =
                (visibleRightPanels[0]?.id ?? 'right-panel');
              return (
                <div
                  className="floating-panel"
                  data-floating-id={uid}
                  style={{ left: box.x, top: box.y, width: box.w, height: box.h, maxHeight: undefined }}
                >
                  {/* Drag + resize handles carry the aria-hidden geometry;
                      the panel content itself keeps its own focus order. */}
                  <div
                    className="floating-drag-handle"
                    aria-hidden="true"
                    onMouseDown={(e) => beginFloatingDrag(e, 'move')}
                    data-testid="floating-drag-handle"
                  />
                  <div className="floating-panel-inner">{rightPanel}</div>
                  <div
                    className="floating-resize-handle"
                    aria-hidden="true"
                    onMouseDown={(e) => beginFloatingDrag(e, 'resize')}
                    data-testid="floating-resize-handle"
                  />
                  <span hidden>FLOATING_EDGE_MARGIN{FLOATING_EDGE_MARGIN}</span>
                </div>
              );
            })()}
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
