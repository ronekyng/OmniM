import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from 'react';

export type ConfirmOptions = {
  title: string;
  message?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
};

type Pending = ConfirmOptions & { resolve: (value: boolean) => void };

const ConfirmContext = createContext<((options: ConfirmOptions) => Promise<boolean>) | null>(null);

/** Promise-based replacement for `window.confirm` (UI-FBK-05). */
export function ConfirmProvider({ children }: { children: ReactNode }) {
  const [pending, setPending] = useState<Pending | null>(null);
  const confirmButtonRef = useRef<HTMLButtonElement | null>(null);

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

  useEffect(() => {
    if (!pending) return;
    confirmButtonRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        close(false);
      } else if (event.key === 'Enter') {
        event.preventDefault();
        event.stopPropagation();
        close(true);
      }
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [pending, close]);

  return (
    <ConfirmContext.Provider value={confirm}>
      {children}
      {pending && (
        <div className="confirm-overlay" role="presentation" onMouseDown={() => close(false)}>
          <div
            className="confirm-dialog"
            role="alertdialog"
            aria-modal="true"
            aria-labelledby="confirm-title"
            onMouseDown={(e) => e.stopPropagation()}
          >
            <h2 id="confirm-title">{pending.title}</h2>
            {pending.message && <p>{pending.message}</p>}
            <div className="confirm-actions">
              <button type="button" onClick={() => close(false)}>
                {pending.cancelLabel ?? 'Cancel'}
              </button>
              <button
                ref={confirmButtonRef}
                type="button"
                className={pending.danger ? 'danger' : 'primary'}
                onClick={() => close(true)}
              >
                {pending.confirmLabel ?? 'OK'}
              </button>
            </div>
          </div>
        </div>
      )}
    </ConfirmContext.Provider>
  );
}

export function useConfirm() {
  const confirm = useContext(ConfirmContext);
  if (!confirm) throw new Error('useConfirm must be used within <ConfirmProvider>');
  return confirm;
}
