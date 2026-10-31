import { useState } from 'react';
import { ObjectTreePanel } from '../panels/ObjectTreePanel';
import { GridPanel } from '../components/GridPanel';
import { AvailableComponentsPanel } from '../components/AvailableComponentsPanel';
import { useAppStore } from '../store/appStore';

type LeftTab = 'objects' | 'grids' | 'catalog';

/** Left dock with tab groups (IA_SHELL §3.4): the object tree, the datum
 *  grid/level editor and the catalog browser no longer compete for the same
 *  vertical space. */
export function LeftDock() {
  const [tab, setTab] = useState<LeftTab>('objects');
  const objectCount = useAppStore((s) => s.stats?.object_count ?? 0);

  return (
    <div className="left-dock">
      <div className="dock-tabs" role="tablist" aria-label="Left dock">
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'objects'}
          className={tab === 'objects' ? 'active' : ''}
          onClick={() => setTab('objects')}
        >
          Objects <span className="dock-count">{objectCount}</span>
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'grids'}
          className={tab === 'grids' ? 'active' : ''}
          onClick={() => setTab('grids')}
        >
          Grids
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'catalog'}
          className={tab === 'catalog' ? 'active' : ''}
          onClick={() => setTab('catalog')}
        >
          构件库
        </button>
      </div>
      <div className="dock-body" role="tabpanel">
        {tab === 'objects' && <ObjectTreePanel />}
        {tab === 'grids' && <GridPanel />}
        {tab === 'catalog' && <AvailableComponentsPanel embedded />}
      </div>
    </div>
  );
}
