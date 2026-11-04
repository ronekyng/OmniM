/** Single shortcut/command registry (INTERACTION §5, UI-INT-03).
 *
 * `buildWorkspaceCommands` is the ONLY place where the workspace command list
 * is defined; `CommandPalette`, `ShortcutHelp` and the `F1`/`Ctrl+K` flows all
 * consume this array verbatim. No other file may register a global keyboard
 * listener — `scripts/check_ui_contracts.py` enforces the allow-list.
 */
export type PaletteCommand = {
  id: string;
  title: string;
  shortcut?: string;
  scope?: string;
  disabled?: boolean;
  run: () => void;
};


type NavTarget = '/projects' | '/series-manager' | '/library-manager';

export function buildWorkspaceCommands(ctx: {
  save: () => void;
  undo: () => void;
  redo: () => void;
  delete: () => void;
  canUndo: boolean;
  canRedo: boolean;
  selectionCount: number;
  viewer: React.RefObject<{ fit: () => void; home: () => void; toggleGrid: () => void; toggleOrtho: () => void; setTransformSpace: (s: 'world' | 'local') => void } | null>;
  grid: { visible: boolean; toggle: () => void };
  ortho: { toggle: () => void };
  transformSpaceSetting: { get: 'world' | 'local'; toggle: () => void };
  transformMode: { current: 'translate' | 'rotate' | 'scale' | null; set: (mode: 'translate' | 'rotate' | 'scale' | null) => void };
  navCycle: () => void;
  create: (objectType: 'storey' | 'door') => void;
  navigate: (to: NavTarget) => void;
  help: () => void;
}): PaletteCommand[] {
  return [
    { id: 'file.save', title: 'Save project', shortcut: 'Ctrl/Cmd+S', scope: 'Global', run: ctx.save },
    { id: 'edit.undo', title: 'Undo', shortcut: 'Ctrl/Cmd+Z', scope: 'Global', disabled: !ctx.canUndo, run: ctx.undo },
    { id: 'edit.redo', title: 'Redo', shortcut: 'Ctrl/Cmd+Shift+Z', scope: 'Global', disabled: !ctx.canRedo, run: ctx.redo },
    { id: 'edit.delete', title: 'Delete selected objects', shortcut: 'Delete', scope: 'Global', disabled: ctx.selectionCount === 0, run: ctx.delete },
    { id: 'view.fit', title: 'Fit view', shortcut: 'F', scope: 'Viewport', run: () => ctx.viewer.current?.fit() },
    { id: 'view.home', title: 'Home view', shortcut: 'H', scope: 'Viewport', run: () => ctx.viewer.current?.home() },
    { id: 'view.grid', title: 'Toggle grid', shortcut: 'G', scope: 'Viewport', run: ctx.grid.toggle },
    { id: 'view.ortho', title: 'Toggle orthographic / perspective', shortcut: 'O', scope: 'Viewport', run: ctx.ortho.toggle },
    { id: 'view.space', title: 'Toggle local / world transform space', shortcut: 'X', scope: 'Viewport', run: ctx.transformSpaceSetting.toggle },
    { id: 'transform.move', title: 'Move mode', shortcut: 'W', scope: 'Viewport', run: () => ctx.transformMode.set(ctx.transformMode.current === 'translate' ? null : 'translate') },
    { id: 'transform.rotate', title: 'Rotate mode', shortcut: 'E', scope: 'Viewport', run: () => ctx.transformMode.set(ctx.transformMode.current === 'rotate' ? null : 'rotate') },
    { id: 'create.storey', title: 'Create storey', scope: 'Create', run: () => ctx.create('storey') },
    { id: 'create.door', title: 'Create door', scope: 'Create', run: () => ctx.create('door') },
    { id: 'nav.projects', title: 'Go to projects', scope: 'Navigation', run: () => ctx.navigate('/projects') },
    { id: 'nav.series', title: 'Open Series Manager', scope: 'Navigation', run: () => ctx.navigate('/series-manager') },
    { id: 'nav.library', title: 'Open Library Manager', scope: 'Navigation', run: () => ctx.navigate('/library-manager') },
    // INT-12 (INTERACTION §10): the configurable navigation scheme lives in
    // the registry so it is discoverable/fixed like every other command.
    { id: 'view.nav', title: 'Viewport navigation scheme', scope: 'Viewport', run: () => ctx.navCycle() },
    { id: 'help.shortcuts', title: 'Keyboard shortcuts', shortcut: 'F1', scope: 'Help', run: ctx.help },
  ];
}
