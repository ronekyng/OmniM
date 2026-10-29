/** Recent-project list stored in user settings, never in a project file
 *  (DATA_INTEGRITY §2.2). Keeps at most 10 entries and tolerates corrupt
 *  payloads. */
export const RECENT_KEY = 'omnim.recent.projects.v1';
export const RECENT_LIMIT = 10;

export type RecentProject = {
  path: string;
  name: string;
  lastOpened: number;
  size?: number;
};

export function parseRecent(raw: string | null | undefined): RecentProject[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    const seen = new Set<string>();
    const out: RecentProject[] = [];
    for (const item of parsed) {
      if (!item || typeof item !== 'object') continue;
      const path = typeof item.path === 'string' ? item.path : '';
      if (!path || seen.has(path)) continue;
      seen.add(path);
      out.push({
        path,
        name: typeof item.name === 'string' && item.name ? item.name : path,
        lastOpened: Number(item.lastOpened) || 0,
        size: typeof item.size === 'number' ? item.size : undefined,
      });
    }
    return out.slice(0, RECENT_LIMIT);
  } catch {
    return [];
  }
}

export function loadRecent(storage?: Pick<Storage, 'getItem'> | null): RecentProject[] {
  const store = storage ?? safeStorage();
  if (!store) return [];
  try {
    return parseRecent(store.getItem(RECENT_KEY));
  } catch {
    return [];
  }
}

export function saveRecent(
  entries: RecentProject[],
  storage?: Pick<Storage, 'setItem'> | null,
): void {
  const store = storage ?? safeStorage();
  if (!store) return;
  try {
    store.setItem(RECENT_KEY, JSON.stringify(entries.slice(0, RECENT_LIMIT)));
  } catch {
    // Best-effort user setting.
  }
}

/** Insert/refresh an entry at the front and trim to the limit. */
export function touchRecent(entries: RecentProject[], entry: RecentProject): RecentProject[] {
  const rest = entries.filter((e) => e.path !== entry.path);
  return [entry, ...rest].slice(0, RECENT_LIMIT);
}

export function removeRecent(entries: RecentProject[], path: string): RecentProject[] {
  return entries.filter((e) => e.path !== path);
}

export function formatBytes(size: number | undefined): string {
  if (!size || size < 0) return '—';
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

function safeStorage(): Pick<Storage, 'getItem' | 'setItem'> | null {
  try {
    return typeof localStorage === 'undefined' ? null : localStorage;
  } catch {
    return null;
  }
}
