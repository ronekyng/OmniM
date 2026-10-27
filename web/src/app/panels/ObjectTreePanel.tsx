import { useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { ObjectTree } from '../components/ObjectTree';
import { useAppStore } from '../store/appStore';
import { useThreeViewer } from '../context/ThreeViewerContext';
import { api } from '../../shared/api';
import type { TreeNode } from '../../shared/types';

function findTreeNode(nodes: TreeNode[], id: string): TreeNode | undefined {
  for (const node of nodes) {
    if (node.id === id) return node;
    const child = findTreeNode(node.children ?? [], id);
    if (child) return child;
  }
  return undefined;
}

export function ObjectTreePanel() {
  const navigate = useNavigate();
  const viewerRef = useThreeViewer();
  const tree = useAppStore((s) => s.tree);
  const stats = useAppStore((s) => s.stats);
  const error = useAppStore((s) => s.error);
  const selectedIds = useAppStore((s) => s.selectedIds);
  const setSelectedIds = useAppStore((s) => s.setSelectedIds);
  const setPrimaryMeshInfo = useAppStore((s) => s.setPrimaryMeshInfo);
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);

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
      name: `${type}-child-${Date.now()}`,
      description: '',
      geometry: { operations: [{ operation: { Box: { length: 1, width: 1, height: 1 } } }] },
      parent_id: parentId,
    });
    refresh();
    reloadScene();
  }, [tree, refresh, reloadScene]);

  const handleDuplicate = useCallback(async (id: string) => {
    const node = findTreeNode(tree, id);
    const name = node ? `${node.name || 'Unnamed'}-copy` : 'copy';
    await api.duplicateObject(id, name, '');
    refresh();
    reloadScene();
  }, [tree, refresh, reloadScene]);

  const handleDelete = useCallback(async (id: string) => {
    const response = await api.deleteObject(id);
    if (!response.ok) throw new Error(`Delete failed: ${response.status} ${response.statusText}`);
    if (selectedIds.includes(id)) {
      setSelectedIds((prev) => prev.filter((prevId) => prevId !== id));
    }
    refresh();
    reloadScene();
  }, [selectedIds, setSelectedIds, refresh, reloadScene]);

  const handleReparent = useCallback(async (childId: string, newParentId: string | null) => {
    await api.setParent(childId, newParentId);
    refresh();
    reloadScene();
  }, [refresh, reloadScene]);

  const handleEditComponent = useCallback((id: string) => {
    navigate(`/component-editor?component=${id}`);
  }, [navigate]);

  return (
    <>
      <h1>OmniM Editor</h1>
      {stats && (
        <div className="stats-bar">
          Objects: <b>{stats.object_count}</b> · Materials: <b>{stats.material_count}</b> · Lifecycle: <b>{stats.lifecycle_record_count}</b>
        </div>
      )}
      {error && <p className="error">{error}</p>}
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
      />
    </>
  );
}