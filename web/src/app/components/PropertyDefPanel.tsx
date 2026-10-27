import { useCallback, useEffect, useRef, useState } from 'react';
import type { ObjectPropertyDef, PropertyValue } from '../../shared/types';

type PropertyDefPanelProps = {
  defs: ObjectPropertyDef[];
  values: Record<string, string>;
  onChange: (key: string, value: string) => Promise<void>;
  /** Optional: hide groups whose defs list is empty */
  collapsedGroups?: Set<string>;
};

/**
 * Data-driven property editor panel.
 * Renders inputs for each `ObjectPropertyDef` — slider (double), text (string),
 * checkbox (bool), or select (choice) — keyed by property name.
 *
 * This replaces the hardcoded `TYPE_PARAMS` approach: the definitions come from
 * the backend's `GET /api/object-types/{type}/properties` endpoint, which in
 * turn derives them from the catalog's `ComponentDef.params` for catalog-aware
 * objects and from a curated per-type schema for built-in types.
 */
export function PropertyDefPanel({ defs, values, onChange, collapsedGroups }: PropertyDefPanelProps) {
  if (defs.length === 0) return null;

  const groups = groupBy(defs, (d) => d.group);

  return (
    <>
      {Object.entries(groups).map(([groupName, groupDefs]) => {
        if (collapsedGroups?.has(groupName)) return null;
        return (
          <div className="propsets-section" key={groupName}>
            <h3>{groupName}</h3>
            {groupDefs.map((def) => (
              <PropertyDefRow
                key={def.name}
                def={def}
                value={values[def.name] ?? ''}
                onChange={onChange}
              />
            ))}
          </div>
        );
      })}
    </>
  );
}

function PropertyDefRow({ def, value, onChange }: { def: ObjectPropertyDef; value: string; onChange: (key: string, value: string) => Promise<void> }) {
  const [saving, setSaving] = useState(false);
  const [local, setLocal] = useState(value);

  useEffect(() => { setLocal(value); }, [value]);

  const save = useCallback(async (v: string) => {
    setSaving(true);
    await onChange(def.name, v);
    setSaving(false);
  }, [def.name, onChange]);

  const handleBlur = useCallback(async () => {
    if (local === value) return;
    await save(local);
  }, [local, value, save]);

  switch (def.kind) {
    case 'double':
      return (
        <div className="prop-row">
          <span className="prop-label" title={def.description}>{def.label}</span>
          <input
            className="prop-value"
            type="number"
            min={isFinite(def.min) ? def.min : undefined}
            max={isFinite(def.max) ? def.max : undefined}
            step={def.step > 0 ? def.step : 'any'}
            value={local}
            onChange={(e) => setLocal(e.target.value)}
            onBlur={() => void handleBlur()}
            onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
          />
          {def.unit && <span className="prop-unit">{def.unit}</span>}
          {saving && <span className="prop-saving">...</span>}
        </div>
      );

    case 'string':
      return (
        <div className="prop-row">
          <span className="prop-label" title={def.description}>{def.label}</span>
          <input
            className="prop-value"
            value={local}
            onChange={(e) => setLocal(e.target.value)}
            onBlur={() => void handleBlur()}
            onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
          />
          {saving && <span className="prop-saving">...</span>}
        </div>
      );

    case 'bool':
      return (
        <div className="prop-row">
          <span className="prop-label" title={def.description}>{def.label}</span>
          <label className="prop-value-toggle">
            <input
              type="checkbox"
              checked={local === 'true' || local === '1'}
              onChange={(e) => {
                const v = e.target.checked ? 'true' : 'false';
                setLocal(v);
                void save(v);
              }}
            />
            <span>{local === 'true' ? '是' : '否'}</span>
          </label>
        </div>
      );

    case 'choice':
      return (
        <div className="prop-row">
          <span className="prop-label" title={def.description}>{def.label}</span>
          <select
            className="prop-value"
            value={local}
            onChange={(e) => {
              const v = e.target.value;
              setLocal(v);
              void save(v);
            }}
          >
            <option value="">—</option>
            {def.choices.map((c) => (
              <option key={c} value={c}>{c}</option>
            ))}
          </select>
          {saving && <span className="prop-saving">...</span>}
        </div>
      );

    default:
      return null;
  }
}

function groupBy<T>(items: T[], keyFn: (item: T) => string): Record<string, T[]> {
  const groups: Record<string, T[]> = {};
  for (const item of items) {
    const key = keyFn(item);
    (groups[key] ??= []).push(item);
  }
  return groups;
}