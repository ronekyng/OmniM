import { useRef, useState, useEffect, useCallback } from 'react';
import type { TreeNode } from '../../shared/types';
import { colorForType } from './ThreeViewer';

type ObjectTreeProps = {
  nodes: TreeNode[];
  selectedIds?: readonly string[];
  onSelectionChanged?: (ids: string[]) => void;
  onDoubleClick?: (id: string) => void;
  onCreateChild?: (parentId: string) => void;
  onDuplicate?: (id: string) => void;
  onDelete?: (id: string) => void;
  onReparent?: (childId: string, newParentId: string | null) => void;
  onEditComponent?: (id: string) => void;
};

function filterTree(nodes: TreeNode[], query: string): TreeNode[] {
  const q = query.toLowerCase().trim();
  if (!q) return nodes;
  return nodes.reduce<TreeNode[]>((acc, node) => {
    const matches =
      node.name.toLowerCase().includes(q) ||
      node.object_type.toLowerCase().includes(q) ||
      node.ifc_class.toLowerCase().includes(q);
    const filteredChildren = filterTree(node.children, query);
    if (matches || filteredChildren.length > 0) {
      acc.push({
        ...node,
        children: matches ? node.children : filteredChildren,
      });
    }
    return acc;
  }, []);
}

function isDescendantOfAny(nodes: TreeNode[], nodeId: string, targetId: string): boolean {
  for (const node of nodes) {
    if (node.id === nodeId) {
      return hasDescendant(node, targetId);
    }
    const found = isDescendantOfAny(node.children, nodeId, targetId);
    if (found) return true;
  }
  return false;
}

function hasDescendant(node: TreeNode, targetId: string): boolean {
  for (const child of node.children) {
    if (child.id === targetId) return true;
    if (hasDescendant(child, targetId)) return true;
  }
  return false;
}

type ObjectTreeNodeProps = {
  node: TreeNode;
  allNodes: TreeNode[];
  selectedIds?: readonly string[];
  searchQuery: string;
  onSelectionChanged: (ids: string[]) => void;
  onDoubleClick: (id: string) => void;
  onContextMenu: (e: React.MouseEvent, nodeId: string) => void;
  onCreateChild?: (parentId: string) => void;
  onDuplicate?: (id: string) => void;
  onDelete?: (id: string) => void;
  onReparent?: (childId: string, newParentId: string | null) => void;
  dragNodeId: string | null;
  setDragNodeId: (id: string | null) => void;
};

function ObjectTreeNode({
  node,
  allNodes,
  selectedIds,
  searchQuery,
  onSelectionChanged,
  onDoubleClick,
  onContextMenu,
  onCreateChild,
  onDuplicate,
  onDelete,
  onReparent,
  dragNodeId,
  setDragNodeId,
}: ObjectTreeNodeProps) {
  const color = colorForType(node.ifc_class || node.object_type);
  const hexColor = '#' + color.toString(16).padStart(6, '0');
  const isSelected = selectedIds?.includes(node.id) ?? false;
  const [dragOver, setDragOver] = useState(false);

  const handleDragStart = useCallback(
    (e: React.DragEvent) => {
      e.dataTransfer.setData('text/plain', node.id);
      e.dataTransfer.effectAllowed = 'move';
      setDragNodeId(node.id);
    },
    [node.id, setDragNodeId],
  );

  const handleDragOver = useCallback(
    (e: React.DragEvent) => {
      if (!dragNodeId || dragNodeId === node.id) return;
      if (isDescendantOfAny(allNodes, node.id, dragNodeId)) return;
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
      setDragOver(true);
    },
    [dragNodeId, node.id, allNodes],
  );

  const handleDragLeave = useCallback(() => {
    setDragOver(false);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      const childId = e.dataTransfer.getData('text/plain');
      if (!childId || childId === node.id) return;
      if (isDescendantOfAny(allNodes, childId, node.id)) return;
      onReparent?.(childId, node.id);
      setDragNodeId(null);
    },
    [node.id, allNodes, onReparent, setDragNodeId],
  );

  const handleDragEnd = useCallback(() => {
    setDragOver(false);
    setDragNodeId(null);
  }, [setDragNodeId]);

  return (
    <li
      className={`tree-node${isSelected ? ' selected' : ''}${dragOver ? ' drag-over' : ''}`}
      draggable
      onClick={(e) => {
        e.stopPropagation();
        const ctrl = e.ctrlKey || e.metaKey;
        if (ctrl) {
          const current = new Set(selectedIds ?? []);
          current.has(node.id) ? current.delete(node.id) : current.add(node.id);
          onSelectionChanged([...current]);
        } else {
          onSelectionChanged([node.id]);
        }
      }}
      onDoubleClick={(e) => {
        e.stopPropagation();
        onDoubleClick(node.id);
      }}
      onContextMenu={(e) => {
        e.preventDefault();
        e.stopPropagation();
        onContextMenu(e, node.id);
      }}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      onDragEnd={handleDragEnd}
    >
      <span className="tree-dot" style={{ background: hexColor }} />
      <div className="tree-info">
        <div className="tree-name">{node.name || 'Unnamed'}</div>
        <div className="tree-meta">{node.ifc_class || node.object_type}</div>
      </div>
      {node.children.length > 0 && (
        <ul className="tree-children">
          {node.children.map((child) => (
            <ObjectTreeNode
              key={child.id}
              node={child}
              allNodes={allNodes}
              selectedIds={selectedIds}
              searchQuery={searchQuery}
              onSelectionChanged={onSelectionChanged}
              onDoubleClick={onDoubleClick}
              onContextMenu={onContextMenu}
              onCreateChild={onCreateChild}
              onDuplicate={onDuplicate}
              onDelete={onDelete}
              onReparent={onReparent}
              dragNodeId={dragNodeId}
              setDragNodeId={setDragNodeId}
            />
          ))}
        </ul>
      )}
    </li>
  );
}

export function ObjectTree({
  nodes,
  selectedIds,
  onSelectionChanged,
  onDoubleClick,
  onCreateChild,
  onDuplicate,
  onDelete,
  onReparent,
  onEditComponent,
}: ObjectTreeProps) {
  const onSelectionChangedRef = useRef(onSelectionChanged);
  const onDblRef = useRef(onDoubleClick);
  const onCreateRef = useRef(onCreateChild);
  const onDupRef = useRef(onDuplicate);
  const onDelRef = useRef(onDelete);
  const onRepRef = useRef(onReparent);
  const onEditRef = useRef(onEditComponent);
  onSelectionChangedRef.current = onSelectionChanged;
  onDblRef.current = onDoubleClick;
  onCreateRef.current = onCreateChild;
  onDupRef.current = onDuplicate;
  onDelRef.current = onDelete;
  onRepRef.current = onReparent;
  onEditRef.current = onEditComponent;

  const [searchQuery, setSearchQuery] = useState('');
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    nodeId: string;
  } | null>(null);
  const [dragNodeId, setDragNodeId] = useState<string | null>(null);

  const contextMenuRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const filtered = filterTree(nodes, searchQuery);

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, nodeId: string) => {
      const menuWidth = 200;
      const menuHeight = 220;
      let x = e.clientX;
      let y = e.clientY;
      if (x + menuWidth > window.innerWidth) x = window.innerWidth - menuWidth - 4;
      if (y + menuHeight > window.innerHeight) y = window.innerHeight - menuHeight - 4;
      if (x < 0) x = 4;
      if (y < 0) y = 4;
      setContextMenu({ x, y, nodeId });
    },
    [],
  );

  const closeContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  useEffect(() => {
    if (!contextMenu) return;
    const handleClick = (e: MouseEvent) => {
      if (contextMenuRef.current && !contextMenuRef.current.contains(e.target as Node)) {
        closeContextMenu();
      }
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') closeContextMenu();
    };
    const timer = setTimeout(() => {
      document.addEventListener('mousedown', handleClick);
      document.addEventListener('keydown', handleKey);
    }, 0);
    return () => {
      clearTimeout(timer);
      document.removeEventListener('mousedown', handleClick);
      document.removeEventListener('keydown', handleKey);
    };
  }, [contextMenu, closeContextMenu]);

  const handleFocus = useCallback(() => {
    if (contextMenu) onDblRef.current?.(contextMenu.nodeId);
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  const handleProperties = useCallback(() => {
    if (contextMenu) onSelectionChangedRef.current?.([contextMenu.nodeId]);
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  const handleCreateChild = useCallback(() => {
    if (contextMenu) onCreateRef.current?.(contextMenu.nodeId);
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  const handleDuplicate = useCallback(() => {
    if (contextMenu) onDupRef.current?.(contextMenu.nodeId);
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  const handleDelete = useCallback(() => {
    if (contextMenu) onDelRef.current?.(contextMenu.nodeId);
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  const handleCopyId = useCallback(() => {
    if (contextMenu) navigator.clipboard.writeText(contextMenu.nodeId);
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  const handleEditComponent = useCallback(() => {
    if (contextMenu) onEditRef.current?.(contextMenu.nodeId);
    closeContextMenu();
  }, [contextMenu, closeContextMenu]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === '/' && !e.ctrlKey && !e.metaKey && document.activeElement !== searchRef.current) {
        e.preventDefault();
        searchRef.current?.focus();
      }
    };
    document.addEventListener('keydown', handleKey);
    return () => document.removeEventListener('keydown', handleKey);
  }, []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div className="tree-search-wrap">
        <input
          ref={searchRef}
          className="tree-search"
          type="text"
          placeholder="Search by name / type / IFC class… (/)"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>
      <ul className="object-tree">
        {filtered.map((node) => (
          <ObjectTreeNode
            key={node.id}
            node={node}
            allNodes={nodes}
            selectedIds={selectedIds}
            searchQuery={searchQuery}
            onSelectionChanged={(ids) => onSelectionChangedRef.current?.(ids)}
            onDoubleClick={(id) => onDblRef.current?.(id)}
            onContextMenu={handleContextMenu}
            onCreateChild={(id) => onCreateRef.current?.(id)}
            onDuplicate={(id) => onDupRef.current?.(id)}
            onDelete={(id) => onDelRef.current?.(id)}
            onReparent={(childId, newParentId) => onRepRef.current?.(childId, newParentId)}
            dragNodeId={dragNodeId}
            setDragNodeId={setDragNodeId}
          />
        ))}
      </ul>

      {contextMenu && (
        <div
          ref={contextMenuRef}
          className="tree-context-menu"
          style={{
            position: 'fixed',
            left: contextMenu.x,
            top: contextMenu.y,
            zIndex: 9999,
          }}
        >
          <button className="ctx-menu-item" onClick={handleFocus}>
            Focus in 3D
          </button>
          <button className="ctx-menu-item" onClick={handleProperties}>
            Properties
          </button>
          <button className="ctx-menu-item" onClick={handleEditComponent}>
            Edit Component
          </button>
          <button className="ctx-menu-item" onClick={handleCreateChild}>
            Create Child
          </button>
          <button className="ctx-menu-item" onClick={handleDuplicate}>
            Duplicate
          </button>
          <button className="ctx-menu-item" onClick={handleDelete}>
            Delete
          </button>
          <button className="ctx-menu-item" onClick={handleCopyId}>
            Copy ID
          </button>
        </div>
      )}
    </div>
  );
}

export default ObjectTree;
