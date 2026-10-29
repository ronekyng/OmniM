import { describe, expect, it } from 'vitest';
import {
  RECENT_LIMIT,
  loadRecent,
  parseRecent,
  removeRecent,
  saveRecent,
  touchRecent,
} from '../shared/recentProjects';

function fakeStorage(initial: Record<string, string> = {}) {
  const data = { ...initial };
  return {
    data,
    getItem: (key: string) => data[key] ?? null,
    setItem: (key: string, value: string) => { data[key] = value; },
  };
}

describe('recent projects (UI-DAT-01)', () => {
  it('deduplicates, ignores malformed entries, and caps at ten', () => {
    const entries = Array.from({ length: 14 }, (_, i) => ({
      path: `/p/${i}.omnim`, name: `P${i}`, lastOpened: i,
    }));
    const parsed = parseRecent(JSON.stringify([...entries, { path: '/p/0.omnim' }, null, 'x']));
    expect(parsed).toHaveLength(RECENT_LIMIT);
    expect(parsed[0].path).toBe('/p/0.omnim');
    expect(parseRecent('{bad json')).toEqual([]);
  });

  it('touching an existing path moves it to the front without duplication', () => {
    const list = [
      { path: 'a', name: 'A', lastOpened: 1 },
      { path: 'b', name: 'B', lastOpened: 2 },
    ];
    const next = touchRecent(list, { path: 'b', name: 'B2', lastOpened: 3 });
    expect(next.map((e) => e.path)).toEqual(['b', 'a']);
    expect(next[0].name).toBe('B2');
  });

  it('round-trips through storage and removes a missing entry', () => {
    const storage = fakeStorage();
    saveRecent([{ path: 'a', name: 'A', lastOpened: 1 }], storage);
    expect(loadRecent(storage)).toHaveLength(1);
    expect(removeRecent(loadRecent(storage), 'a')).toEqual([]);
  });
});
