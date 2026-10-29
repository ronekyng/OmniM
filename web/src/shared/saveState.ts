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

/** Status-bar / title text for the current state. */
export function saveLabel(state: SaveState): string {
  switch (state.status) {
    case 'clean':
      return state.lastSavedAt ? `Saved ${formatClock(state.lastSavedAt)}` : 'Saved';
    case 'dirty':
      return 'Unsaved changes';
    case 'saving':
      return 'Saving…';
    case 'failed':
      return 'Save failed';
  }
}

/** Whether an unload/navigation needs the unsaved-changes prompt. */
export function shouldWarnBeforeUnload(state: SaveState): boolean {
  return state.status === 'dirty' || state.status === 'failed';
}
