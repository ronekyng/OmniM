import { panelRegistry } from './PanelRegistry';
import type { ReactNode } from 'react';
import { PropertiesPanel } from './PropertiesPanel';
import { OmniPanelBottom } from './OmniPanelBottom';
import { AvailableComponentsPanel } from '../components/AvailableComponentsPanel';
import { BuildingPanel, CivilPanel, PlantPanel, MechanicalPanel } from './DomainPanels';
import { useAppStore } from '../store/appStore';

export function registerDefaultPanels() {
  panelRegistry.registerMany([
    {
      id: 'properties',
      position: 'right',
      component: PropertiesPanel,
      visible: () => useAppStore.getState().selectedIds.length > 0 || useAppStore.getState().creationForm !== null || useAppStore.getState().creationPhase === 'picking',
      order: 0,
      title: 'Properties',
    },
    {
      id: 'available-components',
      position: 'right',
      component: AvailableComponentsPanel,
      visible: () => useAppStore.getState().catalogPanelOpen,
      order: 1,
      title: 'Available Components',
    },
    {
      id: 'domain-building',
      position: 'right',
      component: BuildingPanel,
      visible: () => !!useAppStore.getState().domainPanelOpen['building'],
      order: 3,
      title: '建筑',
    },
    {
      id: 'domain-civil',
      position: 'right',
      component: CivilPanel,
      visible: () => !!useAppStore.getState().domainPanelOpen['civil'],
      order: 4,
      title: '土木',
    },
    {
      id: 'domain-plant',
      position: 'right',
      component: PlantPanel,
      visible: () => !!useAppStore.getState().domainPanelOpen['plant'],
      order: 5,
      title: '工厂',
    },
    {
      id: 'domain-mechanical',
      position: 'right',
      component: MechanicalPanel,
      visible: () => !!useAppStore.getState().domainPanelOpen['mechanical'],
      order: 6,
      title: '机械',
    },
    {
      id: 'omni-console',
      position: 'bottom',
      component: OmniPanelBottom,
      visible: () => true,
      order: 0,
      title: 'Console',
    },
  ]);

  // Dev/e2e aid for the panel error boundary (UI-FBK-06): a panel that always
  // throws while rendering, gated by `sessionStorage['omnim.crashTest']`.
  const crashTest = typeof window !== 'undefined'
    ? window.sessionStorage.getItem('omnim.crashTest') === 'panel'
    : false;
  if (crashTest && !panelRegistry.getRegistrations().some((p) => p.id === 'crash-test')) {
    function CrashPanel(): ReactNode {
      throw new Error('crash-test fixture: panel threw while rendering');
    }
    panelRegistry.register({
      id: 'crash-test',
      position: 'bottom',
      component: CrashPanel,
      visible: () => true,
      order: 1,
      title: 'Crash test',
    });
  }
}