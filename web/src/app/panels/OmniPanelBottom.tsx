import { useCallback } from 'react';
import { OmniPanel } from '../components/OmniPanel';
import { useAppStore } from '../store/appStore';

export function OmniPanelBottom() {
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);
  const markDirty = useAppStore((s) => s.markDirty);

  const handleExecuted = useCallback(() => {
    markDirty();
    refresh();
    reloadScene();
  }, [markDirty, refresh, reloadScene]);

  return <OmniPanel onExecuted={handleExecuted} />;
}