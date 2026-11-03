import { useEffect, useRef, type RefObject } from 'react';

const FOCUSABLE = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';

/**
 * INTERACTION §7.2: one focus contract for every dialog family.
 * - focus moves to the first meaningful control (or a supplied target)
 * - Tab / Shift+Tab cycle inside the dialog
 * - Esc closes
 * - the background is inert while open and focus returns to the trigger
 */
export function useDialogFocus(
  open: boolean,
  dialogRef: RefObject<HTMLElement | null>,
  initialFocusRef: RefObject<HTMLElement | null> | undefined,
  onClose: () => void,
) {
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;

  useEffect(() => {
    if (!open) return;
    const root = document.getElementById('root');
    const trigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const previousAriaHidden = root?.getAttribute('aria-hidden') ?? null;
    const wasInert = root?.hasAttribute('inert') ?? false;
    root?.setAttribute('inert', '');
    root?.setAttribute('aria-hidden', 'true');

    const focusables = () => {
      const dialog = dialogRef.current;
      if (!dialog) return [] as HTMLElement[];
      return Array.from(dialog.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
        (el) => !el.hasAttribute('disabled') && el.getClientRects().length > 0,
      );
    };
    (initialFocusRef?.current ?? focusables()[0])?.focus();

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        onCloseRef.current();
        return;
      }
      if (event.key !== 'Tab') return;
      const dialog = dialogRef.current;
      const items = focusables();
      if (!dialog || items.length === 0) return;
      const first = items[0];
      const last = items[items.length - 1];
      const active = document.activeElement;
      if (event.shiftKey && (active === first || !dialog.contains(active))) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && (active === last || !dialog.contains(active))) {
        event.preventDefault();
        first.focus();
      }
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => {
      window.removeEventListener('keydown', onKeyDown, true);
      if (root) {
        if (wasInert) root.setAttribute('inert', '');
        else root.removeAttribute('inert');
        if (previousAriaHidden === null) root.removeAttribute('aria-hidden');
        else root.setAttribute('aria-hidden', previousAriaHidden);
      }
      if (trigger?.isConnected) trigger.focus();
    };
  }, [open, dialogRef, initialFocusRef]);
}
