import { create } from 'zustand';

/**
 * PERFORMANCE §3.2: the command palette's open flag lives in its own store so
 * that `Ctrl/Cmd+K` toggles it without re-rendering the workspace. The
 * palette component subscribes directly; `App` only calls `setOpen` through
 * `getState()` and never subscribes.
 */
type PaletteState = {
  open: boolean;
  setOpen: (open: boolean) => void;
};

export const usePaletteStore = create<PaletteState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
}));
