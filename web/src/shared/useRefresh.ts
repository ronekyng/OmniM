import { useCallback } from 'react';
import { useAppStore } from '../app/store/appStore';

export function useRefresh() {
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);

  const refreshAll = useCallback(() => {
    refresh();
    reloadScene();
  }, [refresh, reloadScene]);

  return refreshAll;
}