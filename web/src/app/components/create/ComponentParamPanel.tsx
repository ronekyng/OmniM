import type { ParamDef, ParamValueMap } from '../../../shared/types';

type Props = {
  params: ParamDef[];
  values: ParamValueMap;
  onChange: (name: string, value: number) => void;
  /** Only render params in this list (e.g. [ 'length', 'height' ]); undefined = all. */
  filter?: string[];
  compact?: boolean;
};

/** Determine if a ParamDef should be rendered as a choices dropdown. */
function isChoicesParam(p: ParamDef): boolean {
  return p.choices.length > 0 || p.param_type === 4;
}

/** Render a single ParamDef-driven input: slider for ranged numerics,
 *  select for choices, plain number otherwise. */
function ParamInput({ param, value, onChange }: { param: ParamDef; value: number; onChange: (v: number) => void }) {
  const hasRange = isFinite(param.min_value) && isFinite(param.max_value) && param.min_value < param.max_value;
  const hasStep = param.step > 0;

  if (isChoicesParam(param)) {
    return (
      <select
        className="comp-param-select"
        value={String(value)}
        onChange={(e) => onChange(parseFloat(e.target.value))}
      >
        {param.choices.map((c) => (
          <option key={c} value={c}>{c}</option>
        ))}
      </select>
    );
  }

  if (hasRange) {
    return (
      <div className="comp-param-slider-wrap">
        <input
          type="range"
          className="comp-param-slider"
          min={param.min_value}
          max={param.max_value}
          step={hasStep ? param.step : 'any'}
          value={value}
          onChange={(e) => onChange(parseFloat(e.target.value))}
        />
        <div className="comp-param-range">
          <span>{param.min_value}</span>
          <span>{param.max_value}</span>
        </div>
      </div>
    );
  }

  return (
    <input
      type="number"
      className="comp-param-input"
      value={Number.isFinite(value) ? value : 0}
      min={isFinite(param.min_value) ? param.min_value : undefined}
      max={isFinite(param.max_value) ? param.max_value : undefined}
      step={hasStep ? param.step : 'any'}
      onChange={(e) => onChange(parseFloat(e.target.value))}
    />
  );
}

export function ComponentParamPanel({ params, values, onChange, filter, compact }: Props) {
  const visible = filter ? params.filter((p) => filter.includes(p.name)) : params;
  if (visible.length === 0) return null;

  return (
    <div className={`comp-param-panel ${compact ? 'comp-param-panel--compact' : ''}`}>
      {visible.map((p) => (
        <div key={p.name} className="comp-param-row">
          <div className="comp-param-header">
            <span className="comp-param-name" title={p.description || ''}>{p.name}</span>
            <span className="comp-param-value">
              {(values[p.name] ?? p.default_value).toFixed(p.unit === 'mm' || p.unit === 'deg' ? 0 : 2)} {p.unit}
            </span>
          </div>
          <ParamInput
            param={p}
            value={values[p.name] ?? p.default_value}
            onChange={(v) => onChange(p.name, v)}
          />
        </div>
      ))}
    </div>
  );
}
