import type { Toast } from '../../shared/toast';
import { useToast } from '../../shared/toast';
import { IcoClose } from './Icons';

/** Feedback container (FEEDBACK_ERRORS §8 / UI-FBK-07·08):
 *  bottom-right above the status bar, ≤3 visible with an overflow count,
 *  320–420 px, dismissible, errors persist. */
export function ToastContainer() {
  const { removeToast, toasts } = useToast();

  if (toasts.length === 0) return null;

  const visible = toasts.slice(-3);
  const overflow = Math.max(0, toasts.length - 3);

  return (
    <div className="toast-container" role="status" aria-live="polite" aria-atomic="false" aria-label="Notifications">
      {overflow > 0 && <div className="toast-more">+{overflow} more</div>}
      {visible.map((toast: Toast) => {
        return (
          <div key={toast.id} className={`toast-item ${toast.type}`}>
            <span className={`toast-dot ${toast.type}`} />
            <span className="toast-message">{toast.message}</span>
            <button
              type="button"
              className="toast-close"
              onClick={() => removeToast(toast.id)}
              aria-label="Dismiss"
            >
              <IcoClose />
            </button>
          </div>
        );
      })}
    </div>
  );
}
