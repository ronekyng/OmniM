import { useAppStore } from '../store/appStore';
import { useI18n } from '../../shared/i18n';
import { saveLabelKey } from '../../shared/saveState';
import type { TreeNode } from '../../shared/types';

function findNode(nodes: TreeNode[], id: string): TreeNode | undefined {
  for (const node of nodes) {
    if (node.id === id) return node;
    const child = findNode(node.children, id);
    if (child) return child;
  }
  return undefined;
}

function developerMode(): boolean {
  try {
    return typeof localStorage !== 'undefined' && localStorage.getItem('omnim.developer') === '1';
  } catch {
    return false;
  }
}

/** Read-only workspace status bar (IA_SHELL §6, UI-SHL-08). */
export function StatusBar() {
  const { t } = useI18n();
  const selectedIds = useAppStore((s) => s.selectedIds);
  const tree = useAppStore((s) => s.tree);
  const cursorWorld = useAppStore((s) => s.cursorWorld);
  const saveState = useAppStore((s) => s.saveState);
  const fps = useAppStore((s) => s.fps);

  let selection = t('shell.statusbar.no_selection');
  if (selectedIds.length === 1) {
    const node = findNode(tree, selectedIds[0]);
    selection = node
      ? `${node.name} (${node.object_type})`
      : t('shell.statusbar.one_selected');
  } else if (selectedIds.length > 1) {
    selection = t('shell.statusbar.n_selected', { count: selectedIds.length });
  }

  const coordinates = cursorWorld
    ? `${cursorWorld[0].toFixed(3)}, ${cursorWorld[1].toFixed(3)}, ${cursorWorld[2].toFixed(3)}`
    : '—';

  return (
    <footer className="status-bar" role="status" aria-live="polite">
      <span className="status-slot status-coords" title={t('shell.statusbar.coords_hint')}>{coordinates} m</span>
      <span className="status-slot status-selection">{selection}</span>
      <span className="status-slot status-units" title={t('shell.statusbar.units_hint')}>m</span>
      <span className={`status-slot status-save status-${saveState.status}`} title={saveState.error ?? undefined}>
        {(() => {
          const { key, time } = saveLabelKey(saveState);
          return time != null && saveState.status === 'clean'
            ? t('shell.save.clean_at', { time })
            : t(key);
        })()}
      </span>
      {developerMode() && <span className="status-slot status-fps">{fps ?? '—'} fps</span>}
    </footer>
  );
}
