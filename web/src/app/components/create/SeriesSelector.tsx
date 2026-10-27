import { useEffect, useState } from 'react';
import { api } from '../../../shared/api';
import { OBJECT_TYPE_DOMAIN } from '../../../shared/catalog';

export function SeriesSelector({ value, onChange, objectType }: { value: string; onChange: (id: string) => void; objectType: string }) {
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
      <span className="field-label">系列</span>
      <select value={value} onChange={(e) => onChange(e.target.value)}>
        <option value="">-- 默认模板 --</option>
        {filtered.map((s) => (
          <option key={s.id} value={s.name}>{s.name} ({s.domain})</option>
        ))}
      </select>
      {filtered.length === 0 && seriesList.length > 0 && (
        <small className="hint">⚠️ 当前类型无匹配系列</small>
      )}
      {value ? <small className="hint">通过系列选择构件规格</small> : <small className="hint">使用类型默认目录构件</small>}
    </div>
  );
}