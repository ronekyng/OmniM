import { useRef, useState, useEffect, useCallback } from 'react';
import type { TreeNode } from '../../shared/types';
import { colorForType } from './typeColors';
import { IcoChevronDown, IcoChevronRight } from './Icons';

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
  onRename?: (id: string, name: string) => void;
};

function findNode(nodes: TreeNode[], id: string): TreeNode | undefined {
  for (const node of nodes) {
    if (node.id === id) return node;
    const nested = findNode(node.children, id);
    if (nested) return nested;
  }
  return undefined;
}

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
  level: number;
  focusedId: string | null;
  setFocusedId: (id: string) => void;
  onKeyboardContextMenu: (nodeId: string) => void;
  onRename?: (id: string, name: string) => void;
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
  level,
  focusedId,
  setFocusedId,
  onKeyboardContextMenu,
  onRename,
}: ObjectTreeNodeProps) {
  const color = colorForType(node.ifc_class || node.object_type);
  const hexColor = '#' + color.toString(16).padStart(6, '0');
  const isSelected = selectedIds?.includes(node.id) ?? false;
  const hasChildren = node.children.length > 0;
  const [expanded, setExpanded] = useState(true);
  const [dragOver, setDragOver] = useState(false);
  const [editing, setEditing] = useState(false);
  const [draftName, setDraftName] = useState(node.name);
  const renameRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editing) {
      renameRef.current?.focus();
      renameRef.current?.select();
    }
  }, [editing]);

  const beginRename = () => {
    setDraftName(node.name);
    setEditing(true);
  };
  const commitRename = () => {
    const next = draftName.trim();
    setEditing(false);
    if (next && next !== node.name) onRename?.(node.id, next);
  };

  const select = useCallback(() => {
    onSelectionChanged([node.id]);
    setFocusedId(node.id);
  }, [node.id, onSelectionChanged, setFocusedId]);

  const visibleItems = (li: HTMLElement): HTMLElement[] => {
    const root = li.closest('[role="tree"]');
    if (!root) return [];
    return Array.from(root.querySelectorAll<HTMLElement>('[role="treeitem"]')).filter(
      (item) => item.getClientRects().length > 0,
    );
  };

  const focusItem = (item: HTMLElement | undefined | null) => {
    if (!item) return;
    item.focus();
    const id = item.dataset.nodeId;
    if (id) setFocusedId(id);
  };

  const moveFocus = (li: HTMLElement, delta: number) => {
    const items = visibleItems(li);
    focusItem(items[items.indexOf(li) + delta]);
  };

  const handleTreeKeyDown = (event: React.KeyboardEvent<HTMLLIElement>) => {
    // Nested <li> rows bubble: only the focused row may act on the key.
    if (event.target !== event.currentTarget) return;
    const li = event.currentTarget;
    // INTERACTION §5.3: the row owns roving focus, expansion and selection.
    if (event.key === 'ArrowDown') { event.preventDefault(); moveFocus(li, 1); }
    else if (event.key === 'ArrowUp') { event.preventDefault(); moveFocus(li, -1); }
    else if (event.key === 'Home') { event.preventDefault(); focusItem(visibleItems(li)[0]); }
    else if (event.key === 'End') { event.preventDefault(); const items = visibleItems(li); focusItem(items[items.length - 1]); }
    else if (event.key === 'ArrowRight') {
      event.preventDefault();
      if (hasChildren && !expanded) setExpanded(true);
      else if (hasChildren) focusItem(li.querySelector<HTMLElement>('[role="treeitem"]'));
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault();
      if (hasChildren && expanded) setExpanded(false);
      else focusItem(li.closest('ul[role="group"]')?.closest<HTMLElement>('[role="treeitem"]'));
    } else if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      select();
    } else if (event.key === 'F2') {
      event.preventDefault();
      beginRename();
    } else if (event.key === 'ContextMenu' || (event.shiftKey && event.key === 'F10')) {
      event.preventDefault();
      onKeyboardContextMenu(node.id);
    }
  };

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
      role="treeitem"
      aria-level={level}
      aria-selected={isSelected}
      aria-expanded={hasChildren ? expanded : undefined}
      tabIndex={focusedId === node.id ? 0 : -1}
      data-node-id={node.id}
      draggable={!editing}
      onFocus={() => setFocusedId(node.id)}
      onKeyDown={handleTreeKeyDown}
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
        setFocusedId(node.id);
        e.currentTarget.focus();
      }}
      onDoubleClick={(e) => {
        e.stopPropagation();
        beginRename();
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
      {hasChildren ? (
        <button
          type="button"
          className="tree-toggle"
          data-tree-toggle
          aria-label={expanded ? 'Collapse' : 'Expand'}
          aria-expanded={expanded}
          onClick={(e) => {
            e.stopPropagation();
            setExpanded((v) => !v);
          }}
        >
          {expanded ? <IcoChevronDown size={12} /> : <IcoChevronRight size={12} />}
        </button>
      ) : (
        <span className="tree-toggle-spacer" />
      )}
      <span className="tree-dot" style={{ background: hexColor }} />
      <div className="tree-info">
        {editing ? (
          <input
            ref={renameRef}
            className="tree-rename"
            aria-label="Object name"
            value={draftName}
            onClick={(e) => e.stopPropagation()}
            onChange={(e) => setDraftName(e.target.value)}
            onBlur={commitRename}
            onKeyDown={(e) => {
              e.stopPropagation();
              if (e.key === 'Enter') {
                e.preventDefault();
                commitRename();
              } else if (e.key === 'Escape') {
                e.preventDefault();
                setEditing(false);
              }
            }}
          />
        ) : (
          <div className="tree-name">{node.name || 'Unnamed'}</div>
        )}
        <div className="tree-meta">{node.ifc_class || node.object_type}</div>
      </div>
      {hasChildren && expanded && (
        <ul className="tree-children" role="group">
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
              level={level + 1}
              focusedId={focusedId}
              setFocusedId={setFocusedId}
              onKeyboardContextMenu={onKeyboardContextMenu}
              onRename={onRename}
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
  onRename,
}: ObjectTreeProps) {
  const onSelectionChangedRef = useRef(onSelectionChanged);
  const onDblRef = useRef(onDoubleClick);
  const onCreateRef = useRef(onCreateChild);
  const onDupRef = useRef(onDuplicate);
  const onDelRef = useRef(onDelete);
  const onRepRef = useRef(onReparent);
  const onEditRef = useRef(onEditComponent);
  const onRenameRef = useRef(onRename);
  onSelectionChangedRef.current = onSelectionChanged;
  onDblRef.current = onDoubleClick;
  onCreateRef.current = onCreateChild;
  onDupRef.current = onDuplicate;
  onDelRef.current = onDelete;
  onRepRef.current = onReparent;
  onEditRef.current = onEditComponent;
  onRenameRef.current = onRename;

  const [searchQuery, setSearchQuery] = useState('');
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    nodeId: string;
    viaKeyboard: boolean;
  } | null>(null);
  const [dragNodeId, setDragNodeId] = useState<string | null>(null);
  const [focusedId, setFocusedId] = useState<string | null>(null);

  const contextMenuRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const treeRef = useRef<HTMLUListElement>(null);
  const previousFocusedRef = useRef<string | null>(null);

  const filtered = filterTree(nodes, searchQuery);

  useEffect(() => {
    const previous = previousFocusedRef.current;
    const previousStillExists = previous ? Boolean(findNode(nodes, previous)) : true;
    let next = focusedId;
    if (!next || !findNode(nodes, next)) next = nodes[0]?.id ?? null;
    previousFocusedRef.current = next;
    if (next !== focusedId) setFocusedId(next);
    // A11Y §4: if the focused row was removed, move focus to its neighbour.
    if (previous && !previousStillExists) {
      requestAnimationFrame(() => {
        if (document.activeElement && document.activeElement !== document.body) return;
        const target = next
          ? treeRef.current?.querySelector<HTMLElement>(`[data-node-id="${CSS.escape(next)}"]`)
          : null;
        target?.focus();
      });
    }
  }, [nodes, focusedId]);

  const openContextMenu = useCallback(
    (nodeId: string, clientX: number, clientY: number, viaKeyboard: boolean) => {
      const menuWidth = 200;
      const menuHeight = 220;
      let x = clientX;
      let y = clientY;
      if (x + menuWidth > window.innerWidth) x = window.innerWidth - menuWidth - 4;
      if (y + menuHeight > window.innerHeight) y = window.innerHeight - menuHeight - 4;
      if (x < 0) x = 4;
      if (y < 0) y = 4;
      setContextMenu({ x, y, nodeId, viaKeyboard });
    },
    [],
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, nodeId: string) => {
      openContextMenu(nodeId, e.clientX, e.clientY, false);
    },
    [openContextMenu],
  );

  const handleKeyboardContextMenu = useCallback((nodeId: string) => {
    const row = document.querySelector<HTMLElement>(`[data-node-id="${CSS.escape(nodeId)}"]`);
    const rect = row?.getBoundingClientRect();
    openContextMenu(nodeId, rect?.left ?? 0, rect?.bottom ?? 0, true);
  }, [openContextMenu]);

  const closeContextMenu = useCallback(() => {
    setContextMenu((current) => {
      if (current) {
        document.querySelector<HTMLElement>(`[data-node-id="${CSS.escape(current.nodeId)}"]`)?.focus();
      }
      return null;
    });
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
    if (contextMenu.viaKeyboard) {
      contextMenuRef.current?.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
    }
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

  const handleMenuKeyDown = useCallback((event: React.KeyboardEvent<HTMLDivElement>) => {
    const items = Array.from(
      event.currentTarget.querySelectorAll<HTMLElement>('[role="menuitem"]'),
    );
    const index = items.indexOf(document.activeElement as HTMLElement);
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      items[(index + 1) % items.length]?.focus();
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      items[(index - 1 + items.length) % items.length]?.focus();
    } else if (event.key === 'Home') {
      event.preventDefault();
      items[0]?.focus();
    } else if (event.key === 'End') {
      event.preventDefault();
      items[items.length - 1]?.focus();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      closeContextMenu();
    }
  }, [closeContextMenu]);

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
          placeholder="搜索名称 / 类型 / IFC Class… (/)"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>
      <ul
        ref={treeRef}
        className="object-tree"
        role="tree"
        aria-label="Object tree"
        aria-multiselectable="true"
      >
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
            level={1}
            focusedId={focusedId}
            setFocusedId={setFocusedId}
            onKeyboardContextMenu={handleKeyboardContextMenu}
            onRename={(id, name) => onRenameRef.current?.(id, name)}
          />
        ))}
      </ul>

      {contextMenu && (
        <div
          ref={contextMenuRef}
          className="tree-context-menu"
          role="menu"
          aria-label="Object actions"
          onKeyDown={handleMenuKeyDown}
          style={{
            position: 'fixed',
            left: contextMenu.x,
            top: contextMenu.y,
          }}
        >
          <button className="ctx-menu-item" role="menuitem" tabIndex={-1} onClick={handleFocus}>
            Focus in 3D
          </button>
          <button className="ctx-menu-item" role="menuitem" tabIndex={-1} onClick={handleProperties}>
            Properties
          </button>
          <button className="ctx-menu-item" role="menuitem" tabIndex={-1} onClick={handleEditComponent}>
            Edit Component
          </button>
          <button className="ctx-menu-item" role="menuitem" tabIndex={-1} onClick={handleCreateChild}>
            Create Child
          </button>
          <button className="ctx-menu-item" role="menuitem" tabIndex={-1} onClick={handleDuplicate}>
            Duplicate
          </button>
          <button className="ctx-menu-item" role="menuitem" tabIndex={-1} onClick={handleDelete}>
            Delete
          </button>
          <button className="ctx-menu-item" role="menuitem" tabIndex={-1} onClick={handleCopyId}>
            Copy ID
          </button>
        </div>
      )}
    </div>
  );
}

export default ObjectTree;
