import { useEffect, useState } from 'react';
import type { TreeNode } from '../../../shared/types';
import { CONTAINER_TYPES } from '../../../shared/catalog';

export function ParentSelector({ tree, value, onChange }: { tree?: TreeNode[]; value: string; onChange: (id: string) => void }) {
  const [containers, setContainers] = useState<{ id: string; name: string; type: string; depth: number }[]>([]);

  useEffect(() => {
    if (!tree) {
      setContainers([]);
      return;
    }
    const result: { id: string; name: string; type: string; depth: number }[] = [];
    function walk(nodes: TreeNode[], depth: number) {
      for (const n of nodes) {
        if (CONTAINER_TYPES.has(n.object_type) || CONTAINER_TYPES.has(n.ifc_class)) {
          result.push({ id: n.id, name: n.name, type: n.object_type, depth });
        }
        walk(n.children, depth + 1);
      }
    }
    walk(tree, 0);
    setContainers(result);
  }, [tree]);

  return (
    <div className="create-row create-row--labeled">
      <span className="field-label">放置于</span>
      <select value={value} onChange={(e) => onChange(e.target.value)}>
        <option value="">-- 项目根级 --</option>
        {containers.map((c) => (
          <option key={c.id} value={c.id}>
            {'\u3000'.repeat(c.depth)}{c.name} ({c.type})
          </option>
        ))}
      </select>
      {value && <small className="hint">图元将作为此容器的子项</small>}
    </div>
  );
}