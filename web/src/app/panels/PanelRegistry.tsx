import { useSyncExternalStore } from 'react';
import type { PanelRegistration, PanelPosition } from './PanelTypes';

let registrations: PanelRegistration[] = [];
const listeners = new Set<() => void>();

function notify() {
  for (const fn of listeners) fn();
}

export const panelRegistry = {
  register(panel: PanelRegistration) {
    registrations = [...registrations, panel];
    notify();
  },
  registerMany(panels: PanelRegistration[]) {
    const existing = new Set(registrations.map((p) => p.id));
    const newPanels = panels.filter((p) => !existing.has(p.id));
    if (newPanels.length === 0) return;
    registrations = [...registrations, ...newPanels];
    notify();
  },
  unregister(id: string) {
    registrations = registrations.filter((p) => p.id !== id);
    notify();
  },
  getRegistrations(): PanelRegistration[] {
    return registrations;
  },
  getForPosition(position: PanelPosition): PanelRegistration[] {
    return registrations
      .filter((p) => p.position === position && p.visible())
      .sort((a, b) => a.order - b.order);
  },
  subscribe(fn: () => void) {
    listeners.add(fn);
    return () => listeners.delete(fn);
  },
};

function subscribeToRegistry(fn: () => void) {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

let cachedSnapshots = new Map<string, PanelRegistration[]>();

function getRegistrationsForPosition(position: PanelPosition): PanelRegistration[] {
  const result = panelRegistry.getForPosition(position);
  const key = position;
  const prev = cachedSnapshots.get(key);
  if (prev && prev.length === result.length && prev.every((p, i) => p.id === result[i].id && p.visible === result[i].visible)) {
    return prev;
  }
  cachedSnapshots.set(key, result);
  return result;
}

export function PanelSlot({ position }: { position: PanelPosition }) {
  const panels = useSyncExternalStore(
    subscribeToRegistry,
    () => getRegistrationsForPosition(position),
    () => getRegistrationsForPosition(position),
  );

  return (
    <>
      {panels.map((panel) => {
        const Component = panel.component;
        return <Component key={panel.id} />;
      })}
    </>
  );
}