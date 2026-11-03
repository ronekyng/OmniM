import {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import { createPortal } from 'react-dom';
import { useDialogFocus } from './useDialogFocus';

export type ConfirmOptions = {
  title: string;
  message?: string;
  /** Extra lines rendered as a list (impact, children by type, …). */
  details?: string[];
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
};

type Pending = ConfirmOptions & { resolve: (value: boolean) => void };

const ConfirmContext = createContext<((options: ConfirmOptions) => Promise<boolean>) | null>(null);

/** Promise-based replacement for `window.confirm` (UI-FBK-05, UI-INT-06). */
export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [pending, setPending] = useState<Pending | null>(null);
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const cancelButtonRef = useRef<HTMLButtonElement | null>(null);

  const confirm = useCallback(
    (options: ConfirmOptions) =>
      new Promise<boolean>((resolve) => {
        setPending({ ...options, resolve });
      }),
    [],
  );

  const close = useCallback(
    (value: boolean) => {
      setPending((current) => {
        current?.resolve(value);
        return null;
      });
    },
    [],
  );

  // The safe default for a destructive prompt is Cancel, not the danger action.
  useDialogFocus(Boolean(pending), dialogRef, cancelButtonRef, () => close(false));

  return (
    <ConfirmContext.Provider value={confirm}>
      {children}
      {pending &&
        createPortal(
          <div className="confirm-overlay" role="presentation" onMouseDown={() => close(false)}>
            <div
              ref={dialogRef}
              className="confirm-dialog"
              role="alertdialog"
              aria-modal="true"
              aria-labelledby="confirm-title"
              aria-describedby={pending.message ? 'confirm-message' : undefined}
              onMouseDown={(e) => e.stopPropagation()}
            >
              <h2 id="confirm-title">{pending.title}</h2>
              {pending.message && <p id="confirm-message">{pending.message}</p>}
              {pending.details && pending.details.length > 0 && (
                <ul className="confirm-details">
                  {pending.details.map((line) => (
                    <li key={line}>{line}</li>
                  ))}
                </ul>
              )}
              <div className="confirm-actions">
                <button ref={cancelButtonRef} type="button" onClick={() => close(false)}>
                  {pending.cancelLabel ?? 'Cancel'}
                </button>
                <button
                  type="button"
                  className={pending.danger ? 'danger' : 'primary'}
                  onClick={() => close(true)}
                >
                  {pending.confirmLabel ?? 'OK'}
                </button>
              </div>
            </div>
          </div>,
          document.body,
        )}
    </ConfirmContext.Provider>
  );
}

export function useConfirm() {
  const confirm = useContext(ConfirmContext);
  if (!confirm) throw new Error('useConfirm must be used within <ConfirmProvider>');
  return confirm;
}
