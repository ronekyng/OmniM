import { useRef } from 'react';
import { createPortal } from 'react-dom';
import { useDialogFocus } from './useDialogFocus';
import type { PaletteCommand } from './CommandPalette';

/** A11Y §3: F1 shortcut reference generated from the command registry. */
export function ShortcutHelp({
  open,
  commands,
  onClose,
}: {
  open: boolean;
  commands: PaletteCommand[];
  onClose: () => void;
}) {
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const closeRef = useRef<HTMLButtonElement | null>(null);
  useDialogFocus(open, dialogRef, closeRef, onClose);

  if (!open) return null;
  return createPortal(
    <div className="confirm-overlay" role="presentation" onMouseDown={onClose}>
      <div
        ref={dialogRef}
        className="shortcut-help"
        role="dialog"
        aria-modal="true"
        aria-labelledby="shortcut-help-title"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <h2 id="shortcut-help-title">Keyboard shortcuts</h2>
        <p className="shortcut-help-intro">
          Every command works without a pointer. F6 cycles the shell regions; Esc always
          cancels the innermost action.
        </p>
        <div className="shortcut-help-grid" role="list">
          {commands
            .filter((command) => command.shortcut)
            .map((command) => (
              <div className="shortcut-help-row" role="listitem" key={command.id}>
                <span>{command.title}</span>
                <kbd>{command.shortcut}</kbd>
              </div>
            ))}
        </div>
        <h3>Tree and grid</h3>
        <div className="shortcut-help-grid" role="list">
          {[
            ['Move between rows', '↑ / ↓'],
            ['Expand / collapse', '← / →'],
            ['Activate', 'Enter / Space'],
            ['Context menu', 'Shift+F10'],
            ['Command palette', 'Ctrl/Cmd+K'],
            ['This help', 'F1'],
          ].map(([label, keys]) => (
            <div className="shortcut-help-row" role="listitem" key={keys}>
              <span>{label}</span>
              <kbd>{keys}</kbd>
            </div>
          ))}
        </div>
        <div className="confirm-actions">
          <button ref={closeRef} type="button" className="primary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
}
