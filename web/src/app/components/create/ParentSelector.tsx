import { useEffect, useState } from 'react';
import { useI18n } from '../../../shared/i18n';
import type { TreeNode } from '../../../shared/types';
import { CONTAINER_TYPES } from '../../../shared/catalog';

export function ParentSelector({ tree, value, onChange }: { tree?: TreeNode[]; value: string; onChange: (id: string) => void }) {
  const { t } = useI18n();
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
      <span className="field-label">{t('create.parent.label')}</span>
      <select aria-label="Parent container" value={value} onChange={(e) => onChange(e.target.value)}>
        <option value="">{t('create.parent.root_opt')}</option>
        {containers.map((c) => (
          <option key={c.id} value={c.id}>
            {'\u3000'.repeat(c.depth)}{c.name} ({c.type})
          </option>
        ))}
      </select>
      {value && <small className="hint">{t('create.parent.hint')}</small>}
    </div>
  );
}