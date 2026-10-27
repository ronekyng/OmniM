export function LoadingSpinner({ size = 20 }: { size?: number }) {
  return (
    <span className="loading-spinner" style={{ width: size, height: size }}>
      <svg width={size} height={size} viewBox="0 0 16 16" fill="none">
        <circle cx="8" cy="8" r="6" stroke="var(--border-light, #3a3d45)" strokeWidth="2" />
        <path d="M8 2a6 6 0 0 1 6 6" stroke="var(--accent, #4a90d9)" strokeWidth="2" strokeLinecap="round">
          <animateTransform attributeName="transform" type="rotate" from="0 8 8" to="360 8 8" dur="0.8s" repeatCount="indefinite" />
        </path>
      </svg>
    </span>
  );
}

export function SkeletonBar({ width = '100%', height = 14 }: { width?: string | number; height?: number }) {
  return (
    <div
      className="skeleton-bar"
      style={{ width: typeof width === 'number' ? `${width}px` : width, height }}
    />
  );
}

export function SkeletonLines({ count = 3, height = 12 }: { count?: number; height?: number }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8, padding: '8px 0' }}>
      {Array.from({ length: count }, (_, i) => (
        <SkeletonBar key={i} width={60 + Math.random() * 30 + '%'} height={height} />
      ))}
    </div>
  );
}
