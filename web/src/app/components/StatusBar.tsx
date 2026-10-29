import { useAppStore } from '../store/appStore';
import { saveLabel } from '../../shared/saveState';
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
  const selectedIds = useAppStore((s) => s.selectedIds);
  const tree = useAppStore((s) => s.tree);
  const cursorWorld = useAppStore((s) => s.cursorWorld);
  const saveState = useAppStore((s) => s.saveState);
  const fps = useAppStore((s) => s.fps);

  let selection = 'No selection';
  if (selectedIds.length === 1) {
    const node = findNode(tree, selectedIds[0]);
    selection = node ? `${node.name} (${node.object_type})` : '1 selected';
  } else if (selectedIds.length > 1) {
    selection = `${selectedIds.length} selected`;
  }

  const coordinates = cursorWorld
    ? `${cursorWorld[0].toFixed(3)}, ${cursorWorld[1].toFixed(3)}, ${cursorWorld[2].toFixed(3)}`
    : '—';

  return (
    <footer className="status-bar" role="status" aria-live="polite">
      <span className="status-slot status-coords" title="Cursor world position (m)">{coordinates} m</span>
      <span className="status-slot status-selection">{selection}</span>
      <span className="status-slot status-units" title="Display units">m</span>
      <span className={`status-slot status-save status-${saveState.status}`} title={saveState.error ?? undefined}>
        {saveLabel(saveState)}
      </span>
      {developerMode() && <span className="status-slot status-fps">{fps ?? '—'} fps</span>}
    </footer>
  );
}
