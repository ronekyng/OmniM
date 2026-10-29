/** Workspace layout persistence (P9 / UI-SHL-06).
 *
 *  Layout is per-user UI state: it lives in `localStorage` under
 *  `omnim.layout.v1` and must never be written into the project file. Unknown
 *  versions and corrupt payloads are discarded so a bad key can never blank
 *  the shell.
 */

export const LAYOUT_KEY = 'omnim.layout.v1';
export const LAYOUT_VERSION = 1;

export type DockMode = 'docked' | 'floating';

export type LayoutState = {
  version: typeof LAYOUT_VERSION;
  level: 'workspace';
  docks: {
    left: { width: number };
    right: { width: number; mode: DockMode };
    bottom: { collapsed: boolean; height: number };
  };
  ribbonCollapsed: boolean;
  floating: [];
};

export const DEFAULT_LAYOUT: LayoutState = {
  version: LAYOUT_VERSION,
  level: 'workspace',
  docks: {
    left: { width: 280 },
    right: { width: 340, mode: 'docked' },
    bottom: { collapsed: true, height: 160 },
  },
  ribbonCollapsed: false,
  floating: [],
};

const RIGHT_MIN = 260;
const RIGHT_MAX = 600;
const LEFT_MIN = 240;
const LEFT_MAX = 400;

const clamp = (value: number, min: number, max: number) =>
  Math.min(max, Math.max(min, value));

function storageOrNull(): Pick<Storage, 'getItem' | 'setItem' | 'removeItem'> | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage;
  } catch {
    return null;
  }
}

/** Parse a persisted layout, falling back to defaults on any doubt. */
export function parseLayout(raw: string | null | undefined): LayoutState {
  if (!raw) return structuredClone(DEFAULT_LAYOUT);
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return structuredClone(DEFAULT_LAYOUT);
  }
  if (!parsed || typeof parsed !== 'object') return structuredClone(DEFAULT_LAYOUT);
  const value = parsed as Partial<LayoutState>;
  if (value.version !== LAYOUT_VERSION) return structuredClone(DEFAULT_LAYOUT);
  const right = value.docks?.right;
  const left = value.docks?.left;
  const bottom = value.docks?.bottom;
  return {
    version: LAYOUT_VERSION,
    level: 'workspace',
    docks: {
      left: {
        width: clamp(Number(left?.width) || DEFAULT_LAYOUT.docks.left.width, LEFT_MIN, LEFT_MAX),
      },
      right: {
        width: clamp(Number(right?.width) || DEFAULT_LAYOUT.docks.right.width, RIGHT_MIN, RIGHT_MAX),
        mode: right?.mode === 'floating' ? 'floating' : 'docked',
      },
      bottom: {
        collapsed: bottom?.collapsed !== false,
        height: Math.max(120, Number(bottom?.height) || DEFAULT_LAYOUT.docks.bottom.height),
      },
    },
    ribbonCollapsed: value.ribbonCollapsed === true,
    floating: [],
  };
}

export function loadLayout(
  storage: Pick<Storage, 'getItem'> | null = storageOrNull(),
): LayoutState {
  if (!storage) return structuredClone(DEFAULT_LAYOUT);
  try {
    return parseLayout(storage.getItem(LAYOUT_KEY));
  } catch {
    return structuredClone(DEFAULT_LAYOUT);
  }
}

export function saveLayout(
  layout: LayoutState,
  storage: Pick<Storage, 'setItem'> | null = storageOrNull(),
): void {
  if (!storage) return;
  try {
    storage.setItem(LAYOUT_KEY, JSON.stringify({ ...layout, version: LAYOUT_VERSION }));
  } catch {
    // Quota or privacy mode: layout persistence is best-effort.
  }
}

export function resetLayout(
  storage: Pick<Storage, 'removeItem'> | null = storageOrNull(),
): LayoutState {
  if (storage) {
    try {
      storage.removeItem(LAYOUT_KEY);
    } catch {
      // ignore
    }
  }
  return structuredClone(DEFAULT_LAYOUT);
}
