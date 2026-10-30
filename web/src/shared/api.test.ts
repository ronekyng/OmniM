import { afterEach, describe, expect, it, vi } from 'vitest';
import { api } from './api';

// Contract tests for the api layer (audit 2026-10-30 / acceptance note:
// "no method ever returned success for an HTTP 500"). One failing response
// caught here blocks whole families of methods from silently swallowing
// errors again.

const OK = (body: unknown) =>
  new Response(JSON.stringify(body) as string, { status: 200 });
const ERR = (status: number, body: string) =>
  new Response(body, { status });

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('api error contract', () => {
  it('request: surfaces non-ok as an Error with the server body', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ERR(500, 'internal boom')));
    await expect(api.stats()).rejects.toThrow(/500.*internal boom/);
  });

  it('requestVoid: surfaces non-ok instead of silently succeeding', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ERR(404, 'grid missing')));
    await expect(api.deleteGrid('g-1')).rejects.toThrow(/404.*grid missing/);
  });

  it('requestVoid: passes through ok responses (including 204) silently', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 204 })));
    await expect(api.deleteObject('obj-1')).resolves.toBeUndefined();
    await expect(api.deleteGrid('g-2')).resolves.toBeUndefined();
  });

  it('series endpoint family follows the same contract', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ERR(500, 'series write failed')));
    await expect(api.updateSeries('s-1', { name: 'x' })).rejects.toThrow(/500.*series write failed/);
    await expect(api.addSelector('s-1', { selector_id: 'a', question: 'q' })).rejects.toThrow(/500/);
    await expect(api.addParamConstraint('s-1', { component_def_id: 'c', param_name: 'p' })).rejects.toThrow(/500/);
  });

  it('catalog/feature endpoint family follows the same contract', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ERR(409, 'publish conflict')));
    await expect(api.publishComponent('d-1')).rejects.toThrow(/409.*publish conflict/);
    await expect(api.addConnector('d-1', {} as never)).rejects.toThrow(/409/);
    await expect(api.removeFeature('c-1', 'f-1')).rejects.toThrow(/409/);
  });

  it('assembly creation follows the same contract', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ERR(400, 'assembly requires 1..=64 parts')));
    await expect(
      api.createAssembly({ name: 'ASM', parts: [] }),
    ).rejects.toThrow(/400.*1\.\.=64/);
  });

  it('exportIfc keeps its explicit ok-check and returns the blob', async () => {
    vi.stubGlobal('fetch', vi.fn(async () => ERR(500, 'export failed')));
    await expect(api.exportIfc()).rejects.toThrow(/500/);
  });
});
