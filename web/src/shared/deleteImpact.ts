import type { TreeNode } from './types';

export type DeleteImpact = {
  selectedCount: number;
  /** Descendants that the server deletes with an Assembly container. */
  cascadedCount: number;
  /** Direct children that the server promotes when deleting a non-container. */
  promotedCount: number;
  /** Cascaded descendants grouped by object type, for the confirm dialog. */
  byType: Record<string, number>;
};

function walk(node: TreeNode, visit: (node: TreeNode) => void) {
  for (const child of node.children) {
    visit(child);
    walk(child, visit);
  }
}

/** INTERACTION §7.3: describe what a delete will remove before it happens. */
export function analyzeDeleteImpact(
  tree: readonly TreeNode[],
  ids: readonly string[],
): DeleteImpact {
  const selected = new Set(ids);
  const impact: DeleteImpact = {
    selectedCount: ids.length,
    cascadedCount: 0,
    promotedCount: 0,
    byType: {},
  };
  const counted = new Set<string>();

  const find = (nodes: readonly TreeNode[], id: string): TreeNode | undefined => {
    for (const node of nodes) {
      if (node.id === id) return node;
      const nested = find(node.children, id);
      if (nested) return nested;
    }
    return undefined;
  };

  for (const id of ids) {
    const node = find(tree, id);
    if (!node) continue;
    if (node.object_type === 'Assembly') {
      walk(node, (child) => {
        if (selected.has(child.id) || counted.has(child.id)) return;
        counted.add(child.id);
        impact.cascadedCount += 1;
        impact.byType[child.object_type] = (impact.byType[child.object_type] ?? 0) + 1;
      });
    } else {
      for (const child of node.children) {
        if (selected.has(child.id) || counted.has(child.id)) continue;
        counted.add(child.id);
        impact.promotedCount += 1;
      }
    }
  }
  return impact;
}

export function formatDeleteImpact(impact: DeleteImpact): string {
  const parts = [
    `This removes ${impact.selectedCount} object${impact.selectedCount === 1 ? '' : 's'} from the project.`,
  ];
  if (impact.cascadedCount > 0) {
    const types = Object.entries(impact.byType)
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([type, count]) => `${type} ×${count}`)
      .join(', ');
    parts.push(`It also deletes ${impact.cascadedCount} descendant object${impact.cascadedCount === 1 ? '' : 's'} (${types}).`);
  }
  if (impact.promotedCount > 0) {
    parts.push(`${impact.promotedCount} child object${impact.promotedCount === 1 ? '' : 's'} will be promoted one level.`);
  }
  parts.push('Undo restores the change.');
  return parts.join(' ');
}
