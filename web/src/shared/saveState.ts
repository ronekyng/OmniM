/** Explicit-save state machine (UI-DAT-02/03/06).
 *
 *  Pure transitions so the shell can render the same contract everywhere and
 *  tests can exercise it without a browser.
 */

export type SaveStatus = 'clean' | 'dirty' | 'saving' | 'failed';

export type SaveState = {
  status: SaveStatus;
  /** Epoch milliseconds of the last successful explicit save. */
  lastSavedAt: number | null;
  /** Last failure reason; cleared by a new save attempt or a mutation. */
  error: string | null;
};

export const initialSaveState: SaveState = {
  status: 'clean',
  lastSavedAt: null,
  error: null,
};

export function markDirty(state: SaveState): SaveState {
  if (state.status === 'saving') return state;
  return { ...state, status: 'dirty', error: null };
}

export function beginSave(state: SaveState): SaveState {
  if (state.status === 'saving') return state;
  return { ...state, status: 'saving', error: null };
}

export function saveSucceeded(state: SaveState, at: number = Date.now()): SaveState {
  return { status: 'clean', lastSavedAt: at, error: null };
}

export function saveFailed(state: SaveState, error: string): SaveState {
  return { ...state, status: 'failed', error };
}

export function formatClock(ms: number): string {
  const date = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

/** Status-bar / title text for the current state.
 *  Note (UI/UX audit F1): the dirty state used to read "Unsaved changes",
 *  which was a false alarm when server-side auto-persist is on — every
 *  mutation is already on disk. The copy states that fact and points at the
 *  explicit snapshot shortcut instead. */
/** Status slot key (I18N_L10N §2): components resolve via `t()`/format. */
export type SaveLabelKey =
  | 'shell.save.clean_at'
  | 'shell.save.dirty_hint'
  | 'shell.save.saving'
  | 'shell.save.failed';

export function saveLabelKey(state: SaveState): { key: SaveLabelKey; time?: string } {
  switch (state.status) {
    case 'clean':
      return { key: 'shell.save.clean_at', time: state.lastSavedAt ? formatClock(state.lastSavedAt) : undefined };
    case 'dirty':
      return { key: 'shell.save.dirty_hint' };
    case 'saving':
      return { key: 'shell.save.saving' };
    case 'failed':
      return { key: 'shell.save.failed' };
  }
}

export function saveLabel(state: SaveState): string {
  const { key, time } = saveLabelKey(state);
  if (key === 'shell.save.clean_at') return time ? `Saved ${time}` : 'Saved';
  return { 'shell.save.dirty_hint': 'Auto-saved server-side · Ctrl+S to snapshot', 'shell.save.saving': 'Saving…', 'shell.save.failed': 'Save failed' }[key] ?? key;
}

/** Whether an unload/navigation needs the unsaved-changes prompt. */
export function shouldWarnBeforeUnload(state: SaveState): boolean {
  return state.status === 'dirty' || state.status === 'failed';
}
