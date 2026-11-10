import { useSyncExternalStore } from 'react';
import { api } from './api';
import type { ComponentDef, ParamDef } from './types';
import type { ObjectType } from './types/creation';
import { hasMessage } from './i18n';

// ============================================================
// Data-driven catalog: fetch ComponentDef list from the backend
// API instead of hardcoding variants. ObjectType → ComponentDef
// mapping is derived from each def's ifc_class (the product-level
// path per the dev plan §1.2).
// ============================================================

/** ObjectType → primary IFC class used by seeded catalog components. */
export const OBJECT_TYPE_IFC: Partial<Record<ObjectType, string>> = {
  wall: 'IfcWall',
  slab: 'IfcSlab',
  beam: 'IfcBeam',
  column: 'IfcColumn',
  road: 'IfcRoad',
  pipe: 'IfcPipeSegment',
  tank: 'IfcTank',
  pump: 'IfcPump',
  valve: 'IfcValve',
  door: 'IfcDoor',
  window: 'IfcWindow',
  duct: 'IfcDuctSegment',
  roof: 'IfcRoof',
  floor: 'IfcFloor',
  bridge: 'IfcBridge',
  tunnel: 'IfcTunnel',
  feature_line: 'IfcGeometricCurveSet',
  contour_line: 'IfcGeometricCurveSet',
};

/** Fallback category heuristics when ifc_class is absent or unexpected. */
const CATEGORY_HINTS: Record<string, ObjectType> = {
  wall: 'wall', slab: 'slab', roof: 'roof', floor: 'floor',
  beam: 'beam', column: 'column', channel: 'beam', tee: 'beam', angle: 'beam',
  road: 'road', pipe: 'pipe', elbow: 'pipe', fitting: 'pipe',
  tank: 'tank', pump: 'pump', valve: 'valve',
  door: 'door', window: 'window',
  duct: 'duct', cabletray: 'duct',
};

/** Infer which ObjectType a ComponentDef instantiates into. */
export function defToObjectType(def: ComponentDef): ObjectType | null {
  if (def.ifc_class) {
    const normalized = def.ifc_class.replace(/\s/g, '');
    for (const [ot, ifc] of Object.entries(OBJECT_TYPE_IFC)) {
      if (normalized.startsWith(ifc)) return ot as ObjectType;
    }
  }
  const cat = (def.category || '').toLowerCase().replace(/[^a-z]/g, '');
  for (const [key, ot] of Object.entries(CATEGORY_HINTS)) {
    if (cat.includes(key)) return ot;
  }
  const tags = (def.tags || []).join(' ').toLowerCase();
  for (const [key, ot] of Object.entries(CATEGORY_HINTS)) {
    if (tags.includes(key)) return ot;
  }
  return null;
}

// ── Catalog cache + subscription ──

export type CatalogStoreState = {
  status: 'idle' | 'loading' | 'ready' | 'error';
  catalog: ComponentDef[];
  error: string | null;
};

let catalogState: CatalogStoreState = { status: 'idle', catalog: [], error: null };
let catalogLoadPromise: Promise<ComponentDef[]> | null = null;
const listeners = new Set<() => void>();

function emit() {
  listeners.forEach((l) => l());
}

function setState(next: CatalogStoreState) {
  catalogState = next;
  emit();
}

export function getCatalogSnapshot(): CatalogStoreState {
  return catalogState;
}

export function subscribeCatalog(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Ensure the catalog is loaded (once per session). */
export function ensureCatalogLoaded(force = false): Promise<ComponentDef[]> {
  if (catalogState.status === 'ready' && !force) return Promise.resolve(catalogState.catalog);
  if (catalogState.status === 'loading' && catalogLoadPromise) return catalogLoadPromise;
  setState({ ...catalogState, status: 'loading', error: null });
  catalogLoadPromise = api.listCatalog()
    .then((list) => {
      catalogLoadPromise = null;
      setState({ status: 'ready', catalog: list, error: null });
      return list;
    })
    .catch((err) => {
      catalogLoadPromise = null;
      const msg = err instanceof Error ? err.message : String(err);
      setState({ status: 'error', catalog: catalogState.catalog, error: msg });
      throw err;
    });
  return catalogLoadPromise;
}

/** React hook: subscribe to catalog load state. */
export function useCatalog() {
  const state = useSyncExternalStore(subscribeCatalog, getCatalogSnapshot);
  return state;
}

/** Catalog defs that map to the given ObjectType (API-driven). */
export function defsForType(type: ObjectType, catalog: ComponentDef[] = catalogState.catalog): ComponentDef[] {
  return catalog.filter((d) => defToObjectType(d) === type);
}

/** Best default def for an ObjectType (prefers published, then first). */
export function defaultDefForType(type: ObjectType, catalog: ComponentDef[] = catalogState.catalog): ComponentDef | undefined {
  const defs = defsForType(type, catalog);
  return defs.find((d) => d.published) ?? defs[0];
}

/** Build a default param-values record from a ComponentDef's ParamDefs. */
export function defaultParamsForDef(def: ComponentDef): Record<string, number> {
  const out: Record<string, number> = {};
  for (const p of def.params) out[p.name] = p.default_value;
  return out;
}

/** Resolve a def by id or by name (fallback for stale references). */
export function resolveDef(ref: string, catalog: ComponentDef[] = catalogState.catalog): ComponentDef | undefined {
  return catalog.find((d) => d.id === ref) ?? catalog.find((d) => d.name === ref);
}

// ── Favorites (persisted in localStorage) ──

const FAVORITES_KEY = 'omnim.catalog.favorites';

export function getFavorites(): string[] {
  try {
    return JSON.parse(localStorage.getItem(FAVORITES_KEY) || '[]');
  } catch {
    return [];
  }
}

export function isFavorite(defId: string): boolean {
  return getFavorites().includes(defId);
}

export function toggleFavorite(defId: string): string[] {
  const favs = getFavorites();
  const next = favs.includes(defId) ? favs.filter((f) => f !== defId) : [...favs, defId];
  localStorage.setItem(FAVORITES_KEY, JSON.stringify(next));
  return next;
}

// ============================================================
// Legacy static maps — kept for backwards compatibility and for
// object types that are NOT catalog-backed (containers, genrule
// doors/windows, primitives). The catalog-backed sets are now
// derived dynamically, but these conservative static sets remain
// the source of truth for which ObjectTypes support picking.
// ============================================================

export const DOMAIN_GROUPS: Record<string, { labelKey: string; types: ObjectType[]; labelKeys: Record<string, string> }> = {
  building: {
    labelKey: 'nav.group.label.building', types: ['wall', 'slab', 'column', 'beam', 'door', 'window', 'opening', 'stair', 'roof', 'building', 'storey', 'space', 'site'],
    labelKeys: { wall: 'nav.type.wall', slab: 'nav.type.slab', column: 'nav.type.column', beam: 'nav.type.beam', door: 'nav.type.door', window: 'nav.type.window', opening: 'nav.type.opening', stair: 'nav.type.stair', roof: 'nav.type.roof', building: 'nav.type.building', storey: 'nav.type.storey', space: 'nav.type.space', site: 'nav.type.site' },
  },
  civil: {
    labelKey: 'nav.group.label.civil', types: ['road', 'bridge', 'tunnel', 'pipe', 'duct', 'feature_line', 'contour_line'],
    labelKeys: { road: 'nav.type.road', bridge: 'nav.type.bridge', tunnel: 'nav.type.tunnel', pipe: 'nav.type.pipe', duct: 'nav.type.duct', feature_line: 'nav.type.feature_line', contour_line: 'nav.type.contour_line' },
  },
  plant: {
    labelKey: 'nav.group.label.plant', types: ['tank', 'pump', 'valve', 'pipe'],
    labelKeys: { tank: 'nav.type.tank', pump: 'nav.type.pump', valve: 'nav.type.valve', pipe: 'nav.type.pipe' },
  },
  mechanical: {
    labelKey: 'nav.group.label.mechanical', types: ['beam', 'column', 'part', 'motor', 'assembly'],
    labelKeys: { beam: 'nav.type.beam', column: 'nav.type.column', part: 'nav.type.part', motor: 'nav.type.motor', assembly: 'nav.type.assembly' },
  },
};

export const CONTAINER_TYPES = new Set(['Site', 'Building', 'BuildingStorey', 'Storey', 'Space', 'building', 'storey', 'site', 'space']);

/** Hierarchy-only types: they are datum/containers and own no geometry.
 *  A storey is an elevation, a building/site is a container — none of them
 *  should be created as a primitive box. `Space` is excluded on purpose: it
 *  is a body object with a primitive volume today. */
export const DATUM_TYPES = new Set<ObjectType>(['building', 'storey', 'site']);

export const OBJECT_TYPE_DOMAIN: Record<string, string> = {
  wall: 'building', slab: 'building', beam: 'building', column: 'building',
  door: 'building', window: 'building', stair: 'building', roof: 'building',
  building: 'building', storey: 'building', space: 'building', site: 'building',
  road: 'civil', bridge: 'civil', tunnel: 'civil',
  pipe: 'piping', duct: 'piping', tank: 'piping', pump: 'piping', valve: 'piping', motor: 'piping',
  part: 'mechanical', assembly: 'mechanical', structuralsteel: 'structural',
};

export const GENRULE_TYPES = new Set<ObjectType>(['door', 'window', 'opening']);
/** Types driven by the `line` genrule (thin N-point guide tubes). */
export const LINE_GENRULE_TYPES = new Set<ObjectType>(['feature_line', 'contour_line']);
export const CATALOG_TYPES = new Set<ObjectType>(['wall', 'slab', 'road', 'pipe', 'beam', 'column', 'tank', 'pump', 'valve']);
export const SURFACE_TYPES = new Set<ObjectType>(['wall', 'slab', 'roof', 'floor']);
export const LINEAR_TYPES = new Set<ObjectType>(['road', 'pipe', 'beam', 'column', 'duct', 'bridge', 'tunnel']);
export const POINT_TYPES = new Set<ObjectType>(['point_cloud', 'terrain_model', 'feature_line', 'contour_line']);
export const BODY_TYPES = new Set<ObjectType>(['tank', 'pump', 'valve', 'gear', 'equipment', 'part', 'assembly']);
export const PICKING_TYPES = new Set<ObjectType>(['wall', 'slab', 'roof', 'floor', 'road', 'bridge', 'tunnel', 'pipe', 'beam', 'column', 'duct', 'feature_line', 'contour_line', 'tank', 'pump', 'valve']);
/** Surface types that support a layered stack via LayerEditor + layered genrule. */
export const LAYERED_TYPES = new Set<ObjectType>(['wall', 'slab', 'roof', 'floor']);

/** i18n message key for an ObjectType (for catalog UI); null falls back to the raw type. */
export function objectTypeLabelKey(type: string): string | null {
  const k = `nav.type.${type}`;
  return hasMessage(k) ? k : null;
}

export type { ParamDef };
