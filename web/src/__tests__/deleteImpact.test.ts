import { describe, expect, it } from 'vitest';
import { analyzeDeleteImpact, formatDeleteImpact } from '../shared/deleteImpact';
import type { TreeNode } from '../shared/types';

function node(partial: Partial<TreeNode> & Pick<TreeNode, 'id' | 'object_type'>, children: TreeNode[] = []): TreeNode {
  return {
    name: partial.name ?? partial.id,
    ifc_class: partial.ifc_class ?? '',
    has_geometry: partial.has_geometry ?? false,
    children,
    ...partial,
  };
}

const tree: TreeNode[] = [
  node({ id: 'asm', object_type: 'Assembly', name: 'Steel frame' }, [
    node({ id: 'c1', object_type: 'Component' }),
    node({ id: 'c2', object_type: 'Component' }),
    node({ id: 'p1', object_type: 'Part' }),
  ]),
  node({ id: 'wall', object_type: 'Wall' }, [
    node({ id: 'door', object_type: 'Door' }),
  ]),
];

describe('delete impact (INTERACTION §7.3)', () => {
  it('counts a container subtree by type', () => {
    const impact = analyzeDeleteImpact(tree, ['asm']);
    expect(impact).toMatchObject({
      selectedCount: 1,
      cascadedCount: 3,
      promotedCount: 0,
      byType: { Component: 2, Part: 1 },
    });
  });

  it('reports promoted children for non-container parents', () => {
    const impact = analyzeDeleteImpact(tree, ['wall']);
    expect(impact.cascadedCount).toBe(0);
    expect(impact.promotedCount).toBe(1);
  });

  it('does not double count a child that is also selected', () => {
    const impact = analyzeDeleteImpact(tree, ['asm', 'c1']);
    expect(impact.selectedCount).toBe(2);
    expect(impact.cascadedCount).toBe(2);
    expect(impact.byType).toEqual({ Component: 1, Part: 1 });
  });

  it('ignores ids that are no longer in the tree', () => {
    const impact = analyzeDeleteImpact(tree, ['missing']);
    expect(impact.selectedCount).toBe(1);
    expect(impact.cascadedCount).toBe(0);
    expect(impact.promotedCount).toBe(0);
  });

  it('states scope, child counts and undo in the message', () => {
    const message = formatDeleteImpact(analyzeDeleteImpact(tree, ['asm']));
    expect(message).toContain('removes 1 object');
    expect(message).toContain('3 descendant objects');
    expect(message).toContain('Component ×2');
    expect(message).toContain('Undo restores');
  });
});
