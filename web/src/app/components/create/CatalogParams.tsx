import { useEffect, useMemo } from 'react';
import type { CreationForm } from '../../../shared/types/creation';
import { CATALOG_TYPES, defsForType, defaultDefForType, defaultParamsForDef, ensureCatalogLoaded, resolveDef, useCatalog } from '../../../shared/catalog';
import { ComponentParamPanel } from './ComponentParamPanel';
import type { ComponentDef } from '../../../shared/types';
import { IcoWarning } from '../Icons';
import { useI18n } from '../../../shared/i18n';

type Props = {
  form: CreationForm;
  onChange: (p: Partial<CreationForm>) => void;
};

/** Catalog param section for catalog-backed object types. Fully driven by the
 *  selected ComponentDef's ParamDefs (the product path in the dev plan §1.2/§1.3). */
export function CatalogParams({ form, onChange }: Props) {
  const { t } = useI18n();
  const { status, catalog } = useCatalog();

  useEffect(() => { void ensureCatalogLoaded(); }, []);

  const defs = useMemo(
    () => (CATALOG_TYPES.has(form.objectType) ? defsForType(form.objectType, catalog) : []),
    [form.objectType, catalog],
  );

  const selectedDef: ComponentDef | undefined = useMemo(
    () => resolveDef(form.catalogDefId, catalog) ?? (form.catalogType ? resolveDef(form.catalogType, catalog) : undefined) ?? defaultDefForType(form.objectType, catalog),
    [form.catalogDefId, form.catalogType, form.objectType, catalog],
  );

  if (!CATALOG_TYPES.has(form.objectType)) return null;

  const handleSelectDef = (def: ComponentDef) => {
    onChange({
      catalogDefId: def.id,
      catalogType: def.name,
      paramValues: defaultParamsForDef(def),
    });
  };

  const handleParamChange = (name: string, value: number) => {
    onChange({ paramValues: { ...form.paramValues, [name]: value } });
  };

  // For wall/slab the length param is driven by picking; hide it from the panel.
  const hiddenParams = (form.objectType === 'wall' || form.objectType === 'slab') ? ['length'] : [];

  if (status === 'loading' && defs.length === 0) {
    return <div className="catalog-params"><small className="hint">{t('catp.loading')}</small></div>;
  }
  if (defs.length === 0) {
    return <div className="catalog-params"><small className="hint"><IcoWarning /> {t('catp.none_for_type', { type: form.objectType })}</small></div>;
  }

  return (
    <div className="catalog-params">
      <div className="create-row create-row--labeled">
        <span className="field-label">{t('catp.component')}</span>
        <select
          value={selectedDef?.id ?? ''}
          onChange={(e) => {
            const def = defs.find((d) => d.id === e.target.value);
            if (def) handleSelectDef(def);
          }}
        >
          {defs.map((d) => (
            <option key={d.id} value={d.id}>
              {d.name}{d.published ? '' : t('catp.badge.draft')}
            </option>
          ))}
        </select>
      </div>

      {selectedDef && (
        <>
          <small className="hint">
            {selectedDef.description || selectedDef.category} · {t('catp.materials', { mat: selectedDef.default_material })}
          </small>
          <ComponentParamPanel
            params={selectedDef.params}
            values={form.paramValues}
            onChange={handleParamChange}
            filter={hiddenParams.length ? selectedDef.params.map((p) => p.name).filter((n) => !hiddenParams.includes(n)) : undefined}
          />
        </>
      )}
    </div>
  );
}
