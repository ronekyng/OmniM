import { useEffect, useState } from 'react';
import type { ThreeViewerHandle, ViewName } from './ThreeViewer';

type Props = {
  viewer: React.RefObject<ThreeViewerHandle | null>;
};

// ── Arrow direction mapping ──
const ARROW_VIEWS: Record<ViewName, { up: ViewName; down: ViewName; left: ViewName; right: ViewName }> = {
  front:   { up: 'top',    down: 'bottom', left: 'left',    right: 'right' },
  back:    { up: 'top',    down: 'bottom', left: 'right',   right: 'left' },
  top:     { up: 'back',   down: 'front',  left: 'left',    right: 'right' },
  bottom:  { up: 'front',  down: 'back',   left: 'left',    right: 'right' },
  left:    { up: 'top',    down: 'bottom', left: 'back',    right: 'front' },
  right:   { up: 'top',    down: 'bottom', left: 'front',   right: 'back' },
  iso:     { up: 'top',    down: 'bottom', left: 'left',    right: 'right' },
  // edges
  'front-top':     { up: 'top',    down: 'front',  left: 'left',    right: 'right' },
  'front-bottom':  { up: 'front',  down: 'bottom', left: 'left',    right: 'right' },
  'back-top':      { up: 'top',    down: 'back',   left: 'left',    right: 'right' },
  'back-bottom':   { up: 'back',   down: 'bottom', left: 'left',    right: 'right' },
  'front-left':    { up: 'top',    down: 'bottom', left: 'left',    right: 'front' },
  'front-right':   { up: 'top',    down: 'bottom', left: 'front',   right: 'right' },
  'back-left':     { up: 'top',    down: 'bottom', left: 'back',    right: 'right' },
  'back-right':    { up: 'top',    down: 'bottom', left: 'left',    right: 'back' },
  'top-left':      { up: 'top',    down: 'left',   left: 'back',    right: 'front' },
  'top-right':     { up: 'top',    down: 'right',  left: 'front',   right: 'back' },
  'bottom-left':   { up: 'left',   down: 'bottom', left: 'back',    right: 'front' },
  'bottom-right':  { up: 'right',  down: 'bottom', left: 'front',   right: 'back' },
  // corners
  'front-left-top':     { up: 'top',    down: 'front',  left: 'left',    right: 'front' },
  'front-right-top':    { up: 'top',    down: 'front',  left: 'front',   right: 'right' },
  'front-left-bottom':  { up: 'front',  down: 'bottom', left: 'left',    right: 'front' },
  'front-right-bottom': { up: 'front',  down: 'bottom', left: 'front',   right: 'right' },
  'back-left-top':      { up: 'top',    down: 'back',   left: 'back',    right: 'right' },
  'back-right-top':     { up: 'top',    down: 'back',   left: 'left',    right: 'back' },
  'back-left-bottom':   { up: 'back',   down: 'bottom', left: 'back',    right: 'right' },
  'back-right-bottom':  { up: 'back',   down: 'bottom', left: 'left',    right: 'back' },
};

const PURE_ORTHO: ViewName[] = ['front', 'back', 'left', 'right', 'top', 'bottom'];

export function ViewCube3D({ viewer }: Props) {
  const [currentView, setCurrentView] = useState<ViewName>('iso');

  // Sync current view from viewer
  useEffect(() => {
    const checkView = () => {
      const v = viewer.current?.getView();
      if (v) setCurrentView(v);
    };
    checkView();
    const interval = setInterval(checkView, 100);
    return () => clearInterval(interval);
  }, [viewer]);

  const arrows = ARROW_VIEWS[currentView] || ARROW_VIEWS.iso;
  const showArrows = PURE_ORTHO.includes(currentView);

  return (
    <div className="viewcube-container">
      {showArrows && (
        <>
          <button className="vc-arrow vc-arrow-up" onClick={(e) => { e.stopPropagation(); viewer.current?.setView(arrows.up); }} title={arrows.up}>▼</button>
          <button className="vc-arrow vc-arrow-down" onClick={(e) => { e.stopPropagation(); viewer.current?.setView(arrows.down); }} title={arrows.down}>▲</button>
          <button className="vc-arrow vc-arrow-left" onClick={(e) => { e.stopPropagation(); viewer.current?.setView(arrows.left); }} title={arrows.left}>▶</button>
          <button className="vc-arrow vc-arrow-right" onClick={(e) => { e.stopPropagation(); viewer.current?.setView(arrows.right); }} title={arrows.right}>◀</button>
        </>
      )}
    </div>
  );
}