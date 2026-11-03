import { useState } from 'react';
import { ObjectTreePanel } from '../panels/ObjectTreePanel';
import { GridPanel } from '../components/GridPanel';
import { AvailableComponentsPanel } from '../components/AvailableComponentsPanel';
import { useAppStore } from '../store/appStore';

type LeftTab = 'objects' | 'grids' | 'catalog';

/** Left dock with tab groups (IA_SHELL §3.4): the object tree, the datum
 *  grid/level editor and the catalog browser no longer compete for the same
 *  vertical space. */
const TABS: LeftTab[] = ['objects', 'grids', 'catalog'];

export function LeftDock() {
  const [tab, setTab] = useState<LeftTab>('objects');
  const objectCount = useAppStore((s) => s.stats?.object_count ?? 0);

  const select = (next: LeftTab) => {
    setTab(next);
    requestAnimationFrame(() => document.getElementById(`left-tab-${next}`)?.focus());
  };

  const onTabKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    const index = TABS.indexOf(tab);
    if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
      event.preventDefault();
      const delta = event.key === 'ArrowRight' ? 1 : -1;
      select(TABS[(index + delta + TABS.length) % TABS.length]);
    } else if (event.key === 'Home') {
      event.preventDefault();
      select(TABS[0]);
    } else if (event.key === 'End') {
      event.preventDefault();
      select(TABS[TABS.length - 1]);
    }
  };

  return (
    <div className="left-dock">
      <div className="dock-tabs" role="tablist" aria-label="Left dock" onKeyDown={onTabKeyDown}>
        <button
          id="left-tab-objects"
          type="button"
          role="tab"
          aria-selected={tab === 'objects'}
          aria-controls="left-dock-panel"
          tabIndex={tab === 'objects' ? 0 : -1}
          className={tab === 'objects' ? 'active' : ''}
          onClick={() => setTab('objects')}
        >
          Objects <span className="dock-count">{objectCount}</span>
        </button>
        <button
          id="left-tab-grids"
          type="button"
          role="tab"
          aria-selected={tab === 'grids'}
          aria-controls="left-dock-panel"
          tabIndex={tab === 'grids' ? 0 : -1}
          className={tab === 'grids' ? 'active' : ''}
          onClick={() => setTab('grids')}
        >
          Grids
        </button>
        <button
          id="left-tab-catalog"
          type="button"
          role="tab"
          aria-selected={tab === 'catalog'}
          aria-controls="left-dock-panel"
          tabIndex={tab === 'catalog' ? 0 : -1}
          className={tab === 'catalog' ? 'active' : ''}
          onClick={() => setTab('catalog')}
        >
          构件库
        </button>
      </div>
      <div
        id="left-dock-panel"
        className="dock-body"
        role="tabpanel"
        aria-labelledby={`left-tab-${tab}`}
        tabIndex={0}
      >
        {tab === 'objects' && <ObjectTreePanel />}
        {tab === 'grids' && <GridPanel />}
        {tab === 'catalog' && <AvailableComponentsPanel embedded />}
      </div>
    </div>
  );
}
