import { useEffect, useState } from 'react';
import { SKELETON_DELAY_MS } from './delayedVisibility';

/**
 * True only after `active` has been true for `delayMs` (default 400 ms).
 * Prevents a skeleton flash for fast responses (UI-FBK-02).
 */
export function useDelayedLoading(active: boolean, delayMs: number = SKELETON_DELAY_MS): boolean {
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    if (!active) {
      setVisible(false);
      return;
    }
    setVisible(false);
    const timer = setTimeout(() => setVisible(true), delayMs);
    return () => clearTimeout(timer);
  }, [active, delayMs]);
  return visible;
}
