import type { ComponentType } from 'react';

export type PanelPosition = 'right' | 'left' | 'bottom' | 'floating' | 'modal';

export type PanelRegistration = {
  id: string;
  position: PanelPosition;
  component: ComponentType;
  visible: () => boolean;
  order: number;
  title?: string;
};