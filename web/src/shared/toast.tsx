import { createContext, useContext, useEffect, useState, useCallback, useRef, type ReactNode, type Context } from 'react';

type ToastType = 'success' | 'error' | 'info' | 'warning';

export type Toast = {
  id: number;
  type: ToastType;
  message: string;
};

type ToastContextValue = {
  toasts: Toast[];
  addToast: (type: ToastType, message: string) => void;
  removeToast: (id: number) => void;
};

const ToastContext: Context<ToastContextValue | null> = createContext<ToastContextValue | null>(null);

let nextId = 1;

type ToastListener = (type: ToastType, message: string) => void;
const listeners = new Set<ToastListener>();

/** Imperative toast bridge for modules that cannot use hooks. */
export const notify = {
  success: (message: string) => listeners.forEach((fn) => fn('success', message)),
  error: (message: string) => listeners.forEach((fn) => fn('error', message)),
  info: (message: string) => listeners.forEach((fn) => fn('info', message)),
  warning: (message: string) => listeners.forEach((fn) => fn('warning', message)),
};

export function subscribeToasts(listener: ToastListener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);
  const timersRef = useRef<Map<number, ReturnType<typeof setTimeout>>>(new Map());

  const removeToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
    const timer = timersRef.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
  }, []);

  const addToast = useCallback((type: ToastType, message: string) => {
    const id = nextId++;
    setToasts((prev) => [...prev, { id, type, message }]);
    // Errors persist until dismissed (UI-FBK-07); success/info auto-dismiss.
    if (type !== 'error') {
      const timer = setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
        timersRef.current.delete(id);
      }, 4000);
      timersRef.current.set(id, timer);
    }
  }, []);

  useEffect(() => subscribeToasts((type, message) => addToast(type, message)), [addToast]);

  return (
    <ToastContext.Provider value={{ toasts, addToast, removeToast }}>
      {children}
    </ToastContext.Provider>
  );
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) throw new Error('useToast must be used within <ToastProvider>');
  return ctx;
}

/** Convenience methods — use `toast.success('Done')` etc. */
export function useToastApi() {
  const { addToast } = useToast();
  return {
    success: (msg: string) => addToast('success', msg),
    error: (msg: string) => addToast('error', msg),
    info: (msg: string) => addToast('info', msg),
    warning: (msg: string) => addToast('warning', msg),
  };
}
