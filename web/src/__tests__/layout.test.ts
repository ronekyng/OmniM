import { describe, expect, it } from 'vitest';
import { DEFAULT_LAYOUT, LAYOUT_KEY, loadLayout, parseLayout, resetLayout, saveLayout } from '../shared/layout';

function fakeStorage(initial: Record<string, string> = {}) {
  const data = { ...initial };
  return {
    data,
    getItem: (key: string) => data[key] ?? null,
    setItem: (key: string, value: string) => { data[key] = value; },
    removeItem: (key: string) => { delete data[key]; },
  };
}

describe('workspace layout persistence (UI-SHL-06)', () => {
  it('returns the docked default when nothing is stored', () => {
    const layout = loadLayout(fakeStorage());
    expect(layout.docks.right.mode).toBe('docked');
    expect(layout.docks.right.width).toBe(340);
  });

  it('round-trips and clamps persisted widths', () => {
    const storage = fakeStorage();
    saveLayout({ ...DEFAULT_LAYOUT, docks: { ...DEFAULT_LAYOUT.docks, right: { width: 999, mode: 'floating' } } }, storage);
    const loaded = loadLayout(storage);
    expect(loaded.docks.right.width).toBe(600);
    expect(loaded.docks.right.mode).toBe('floating');
  });

  it('discards corrupt or unknown-version payloads instead of crashing', () => {
    expect(parseLayout('{not json')).toEqual(DEFAULT_LAYOUT);
    expect(parseLayout(JSON.stringify({ version: 99, docks: {} }))).toEqual(DEFAULT_LAYOUT);
    expect(parseLayout(null)).toEqual(DEFAULT_LAYOUT);
  });

  it('reset removes the key and restores defaults', () => {
    const storage = fakeStorage({ [LAYOUT_KEY]: '{"version":1}' });
    expect(resetLayout(storage)).toEqual(DEFAULT_LAYOUT);
    expect(storage.data[LAYOUT_KEY]).toBeUndefined();
  });
});
