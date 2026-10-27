import { createContext, useContext, createRef } from 'react';
import type { RefObject } from 'react';
import type { ThreeViewerHandle } from '../components/ThreeViewer';

export const ThreeViewerContext = createContext<RefObject<ThreeViewerHandle | null>>(createRef<ThreeViewerHandle | null>());

export function useThreeViewer(): RefObject<ThreeViewerHandle | null> {
  return useContext(ThreeViewerContext);
}