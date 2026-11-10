import { useI18n } from '../../shared/i18n';
import { useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { ObjectTree } from '../components/ObjectTree';
import { useAppStore } from '../store/appStore';
import { useThreeViewer } from '../context/ThreeViewerContext';
import { api } from '../../shared/api';
import { nextAutoName, uniqueName } from '../../shared/naming';
import type { TreeNode } from '../../shared/types';
import { useConfirm } from '../components/ConfirmDialog';
import { useToastApi } from '../../shared/toast';
import { analyzeDeleteImpact, formatDeleteImpact } from '../../shared/deleteImpact';

function findTreeNode(nodes: TreeNode[], id: string): TreeNode | undefined {
  for (const node of nodes) {
    if (node.id === id) return node;
    const child = findTreeNode(node.children ?? [], id);
    if (child) return child;
  }
  return undefined;
}

function flattenNames(nodes: TreeNode[]): string[] {
  const names: string[] = [];
  const walk = (list: TreeNode[]) => {
    for (const node of list) {
      names.push(node.name);
      walk(node.children);
    }
  };
  walk(nodes);
  return names;
}

export function ObjectTreePanel() {
  const { t } = useI18n();
  const navigate = useNavigate();
  const viewerRef = useThreeViewer();
  const tree = useAppStore((s) => s.tree);
  const stats = useAppStore((s) => s.stats);
  const error = useAppStore((s) => s.error);
  const selectedIds = useAppStore((s) => s.selectedIds);
  const setSelectedIds = useAppStore((s) => s.setSelectedIds);
  const setPrimaryMeshInfo = useAppStore((s) => s.setPrimaryMeshInfo);
  const confirm = useConfirm();
  const toast = useToastApi();
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);
  const markDirty = useAppStore((s) => s.markDirty);

  const handleSelectionChanged = useCallback((ids: string[]) => {
    setSelectedIds(ids);
    if (ids.length === 1) {
      const info = viewerRef.current?.getMeshInfo(ids[0]) ?? null;
      setPrimaryMeshInfo(info);
    } else {
      setPrimaryMeshInfo(null);
    }
  }, [setSelectedIds, setPrimaryMeshInfo, viewerRef]);

  const handleCreateChild = useCallback(async (parentId: string) => {
    const parent = findTreeNode(tree, parentId);
    const type = parent?.object_type?.toLowerCase() || 'part';
    await api.createObject({
      object_type: type,
      name: nextAutoName(`${type}-child`, flattenNames(tree)),
      description: '',
      geometry: { operations: [{ operation: { Box: { length: 1, width: 1, height: 1 } } }] },
      parent_id: parentId,
    });
    markDirty();
    refresh();
    reloadScene();
  }, [tree, markDirty, refresh, reloadScene]);

  const handleDuplicate = useCallback(async (id: string) => {
    const node = findTreeNode(tree, id);
    const name = node ? uniqueName(`${node.name || 'Unnamed'}-copy`, flattenNames(tree)) : 'copy';
    await api.duplicateObject(id, name, '');
    markDirty();
    refresh();
    reloadScene();
  }, [tree, markDirty, refresh, reloadScene]);

  const handleDelete = useCallback(async (id: string) => {
    // The tree path must use the same guarded delete as the toolbar (UI-INT-07).
    const node = findTreeNode(tree, id);
    const impact = analyzeDeleteImpact(tree, [id]);
    const ok = await confirm({
      title: 'Delete object?',
      message: formatDeleteImpact(impact),
      confirmLabel: 'Delete',
      danger: true,
    });
    if (!ok) return;
    try {
      await api.deleteObject(id);
    } catch (err) {
      toast.error(`Delete failed: ${err instanceof Error ? err.message : String(err)}`);
      return;
    }
    if (selectedIds.includes(id)) {
      setSelectedIds((prev) => prev.filter((prevId) => prevId !== id));
    }
    markDirty();
    toast.success(`Deleted ${node?.name ?? 'object'}`);
    refresh();
    reloadScene();
  }, [tree, confirm, toast, selectedIds, setSelectedIds, markDirty, refresh, reloadScene]);

  const handleRename = useCallback(async (id: string, name: string) => {
    const trimmed = name.trim();
    if (!trimmed) return;
    try {
      await api.updateObject(id, { name: trimmed });
      markDirty();
      toast.success(`Renamed to ${trimmed}`);
      refresh();
      reloadScene();
    } catch (err) {
      toast.error(`Rename failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [markDirty, toast, refresh, reloadScene]);

  const handleReparent = useCallback(async (childId: string, newParentId: string | null) => {
    await api.setParent(childId, newParentId);
    markDirty();
    refresh();
    reloadScene();
  }, [markDirty, refresh, reloadScene]);

  const handleEditComponent = useCallback((id: string) => {
    navigate(`/component-editor?component=${id}`);
  }, [navigate]);

  return (
    <>
      {stats && (
        <div className="stats-bar">
          Objects: <b>{stats.object_count}</b> · Materials: <b>{stats.material_count}</b> · Lifecycle: <b>{stats.lifecycle_record_count}</b>
        </div>
      )}
      {error && <p className="error">{error}</p>}
      {tree.length === 0 ? (
        <div className="panel-state">
          {t('tree.empty_workspace')}
        </div>
      ) : (
        <ObjectTree
          nodes={tree}
          selectedIds={selectedIds}
          onSelectionChanged={handleSelectionChanged}
          onDoubleClick={(id) => viewerRef.current?.focusObject(id)}
          onCreateChild={handleCreateChild}
          onDuplicate={handleDuplicate}
          onDelete={handleDelete}
          onReparent={handleReparent}
          onEditComponent={handleEditComponent}
          onRename={handleRename}
        />
      )}
    </>
  );
}