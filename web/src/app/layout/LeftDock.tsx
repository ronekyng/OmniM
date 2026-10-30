import { useState } from 'react';
import { ObjectTreePanel } from '../panels/ObjectTreePanel';
import { GridPanel } from '../components/GridPanel';
import { useAppStore } from '../store/appStore';

type LeftTab = 'objects' | 'grids';

/** Left dock with tab groups (IA_SHELL §3.4): the object tree and the datum
 *  grid editor no longer compete for the same vertical space. */
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
      </div>
      <div className="dock-body">
        {tab === 'objects' ? <ObjectTreePanel /> : <GridPanel />}
      </div>
    </div>
  );
}
