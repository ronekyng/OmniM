import { useToast } from '../../shared/toast';

const typeColors: Record<string, { bg: string; border: string; dot: string }> = {
  success: { bg: '#1a2e1a', border: '#2a4a2a', dot: '#4caf50' },
  error:   { bg: '#2e1a1a', border: '#4a2a2a', dot: '#f44336' },
  info:    { bg: '#1a2530', border: '#2a3a4a', dot: '#4fc3f7' },
  warning: { bg: '#2e2a1a', border: '#4a3a1a', dot: '#ff9800' },
};

export function ToastContainer() {
  const { toasts, removeToast } = useToast();

  if (toasts.length === 0) return null;

  return (
    <div className="toast-container">
      {toasts.map((t) => {
        const c = typeColors[t.type] ?? typeColors.info;
        return (
          <div
            key={t.id}
            className="toast-item"
            style={{
              background: c.bg,
              borderColor: c.border,
            }}
            onClick={() => removeToast(t.id)}
          >
            <span className="toast-dot" style={{ background: c.dot }} />
            <span className="toast-message">{t.message}</span>
            <span className="toast-close">✕</span>
          </div>
        );
      })}
    </div>
  );
}
