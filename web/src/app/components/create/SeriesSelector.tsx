import { useEffect, useState } from 'react';
import { useI18n } from '../../../shared/i18n';
import { api } from '../../../shared/api';
import { OBJECT_TYPE_DOMAIN } from '../../../shared/catalog';
import { IcoWarning } from '../Icons';

export function SeriesSelector({ value, onChange, objectType }: { value: string; onChange: (id: string) => void; objectType: string }) {
  const { t } = useI18n();
  const [seriesList, setSeriesList] = useState<{ id: string; name: string; domain: string }[]>([]);

  useEffect(() => {
    api.listSeries()
      .then((list) => setSeriesList(list.map((s) => ({ id: s.id, name: s.name, domain: s.domain }))))
      .catch(() => setSeriesList([]));
  }, []);

  const domain = OBJECT_TYPE_DOMAIN[objectType.toLowerCase()] || '';
  const filtered = domain ? seriesList.filter((s) => s.domain === domain) : seriesList;

  return (
    <div className="create-row create-row--labeled">
      <span className="field-label">{t('series.sel.label')}</span>
      <select aria-label="Series" value={value} onChange={(e) => onChange(e.target.value)}>
        <option value="">{t('series.sel.default_opt')}</option>
        {filtered.map((s) => (
          <option key={s.id} value={s.name}>{s.name} ({s.domain})</option>
        ))}
      </select>
      {filtered.length === 0 && seriesList.length > 0 && (
        <small className="hint"><IcoWarning /> {t('series.sel.none_for_type')}</small>
      )}
      {value ? <small className="hint">{t('series.sel.via_series')}</small> : <small className="hint">{t('series.sel.via_default')}</small>}
    </div>
  );
}