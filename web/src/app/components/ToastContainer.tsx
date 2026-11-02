import type { Toast } from '../../shared/toast';
import { useToast } from '../../shared/toast';

const typeColors: Record<string, { bg: string; border: string; dot: string }> = {
  success: { bg: '#1a2e1a', border: '#2a4a2a', dot: '#4caf50' },
  error:   { bg: '#2e1a1a', border: '#4a2a2a', dot: '#f44336' },
  info:    { bg: '#1a2530', border: '#2a3a4a', dot: '#4fc3f7' },
  warning: { bg: '#2e2a1a', border: '#4a3a1a', dot: '#ff9800' },
};

/** Feedback container (FEEDBACK_ERRORS §8 / UI-FBK-07·08):
 *  bottom-right above the status bar, ≤3 visible with an overflow count,
 *  320–420 px, dismissible, errors persist. */
export function ToastContainer() {
  const { removeToast, toasts } = useToast();

  if (toasts.length === 0) return null;

  const visible = toasts.slice(-3);
  const overflow = Math.max(0, toasts.length - 3);

  return (
    <div className="toast-container">
      {overflow > 0 && <div className="toast-more">+{overflow} more</div>}
      {visible.map((toast: Toast) => {
        const c = typeColors[toast.type] ?? typeColors.info;
        return (
          <div
            key={toast.id}
            className={`toast-item ${toast.type}`}
            style={{
              background: c.bg,
              borderColor: c.border,
            }}
          >
            <span className="toast-dot" style={{ background: c.dot }} />
            <span className="toast-message">{toast.message}</span>
            <button
              type="button"
              className="toast-close"
              onClick={() => removeToast(toast.id)}
              aria-label="Dismiss"
            >
              ✕
            </button>
          </div>
        );
      })}
    </div>
  );
}
