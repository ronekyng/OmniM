import { useCallback, useEffect, useState } from 'react';
import { api } from '../../shared/api';
import { storeyOptions, type StoreyOption } from '../../shared/types/creation';
import { useAppStore } from '../store/appStore';
import { useI18n } from '../../shared/i18n';

/** Storey view selector (P8-S4): filters the viewport to the chosen storey and
 *  sets a plan section cut 1.2 m above its datum. The "all" entry restores everything and
 *  disables the storey-driven section. */
export function StoreyViewSelect() {
  const { t } = useI18n();
  const activeStoreyId = useAppStore((s) => s.activeStoreyId);
  const setActiveStorey = useAppStore((s) => s.setActiveStorey);
  const setSection = useAppStore((s) => s.setSection);
  const reloadScene = useAppStore((s) => s.reloadScene);
  const reloadToken = useAppStore((s) => s.reloadToken);
  const [storeys, setStoreys] = useState<StoreyOption[]>([]);

  useEffect(() => {
    let cancelled = false;
    api
      .listObjects()
      .then((objects) => {
        if (!cancelled) setStoreys(storeyOptions(objects));
      })
      .catch(() => {
        if (!cancelled) setStoreys([]);
      });
    return () => {
      cancelled = true;
    };
  }, [reloadToken]);

  const onChange = useCallback(
    (value: string) => {
      const id = value || null;
      setActiveStorey(id);
      const storey = storeys.find((s) => s.id === id);
      if (storey) {
        // Plan cut 1.2 m above the storey datum.
        setSection({ enabled: true, axis: 'z', value: storey.elevation + 1.2 });
      } else {
        setSection((prev) => ({ ...prev, enabled: false }));
      }
      reloadScene();
    },
    [storeys, setActiveStorey, setSection, reloadScene],
  );

  return (
    <select
      value={activeStoreyId ?? ''}
      onChange={(e) => onChange(e.target.value)}
      title={t('storey.view.title')}
      style={{ maxWidth: 132 }}
    >
      <option value="">{t('storey.all')}</option>
      {storeys.map((s) => (
        <option key={s.id} value={s.id}>
          {s.name}（{s.elevation.toFixed(2)} m）
        </option>
      ))}
    </select>
  );
}
