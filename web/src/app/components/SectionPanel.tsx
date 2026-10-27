import type { SectionState } from './ThreeViewer';

type SectionPanelProps = {
  section: SectionState;
  onChange: (section: SectionState) => void;
};

export function SectionPanel({ section, onChange }: SectionPanelProps) {
  return (
    <div className="section-panel">
      <label>
        Section: <span className="section-value">{section.value.toFixed(1)}</span>
      </label>
      <input
        type="range"
        min={-20}
        max={20}
        step={0.1}
        value={section.value}
        onChange={(event) => onChange({ ...section, value: parseFloat(event.currentTarget.value) })}
      />
      <div className="section-axes">
        {(['x','y','z'] as const).map((axis) => (
          <button
            key={axis}
            className={section.axis === axis ? 'active' : ''}
            onClick={() => onChange({ ...section, axis })}
          >
            {axis.toUpperCase()}
          </button>
        ))}
      </div>
    </div>
  );
}