import { useCallback } from 'react';
import { OmniPanel } from '../components/OmniPanel';
import { useAppStore } from '../store/appStore';

export function OmniPanelBottom() {
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);

  const handleExecuted = useCallback(() => {
    refresh();
    reloadScene();
  }, [refresh, reloadScene]);

  return <OmniPanel onExecuted={handleExecuted} />;
}