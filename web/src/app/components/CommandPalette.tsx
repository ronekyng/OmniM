import { useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useDialogFocus } from './useDialogFocus';

// The command type + registry live in `app/commands.ts` (single source).
export type { PaletteCommand } from '../commands';
import type { PaletteCommand } from '../commands';
/** UI-SHL-12 / A11Y §3: keyboard command palette (Ctrl/Cmd+K). */
export function CommandPalette({
  open,
  commands,
  onClose,
}: {
  open: boolean;
  commands: PaletteCommand[];
  onClose: () => void;
}) {
  const [query, setQuery] = useState('');
  const [active, setActive] = useState(0);
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  useDialogFocus(open, dialogRef, inputRef, onClose);

  useEffect(() => {
    if (open) {
      setQuery('');
      setActive(0);
    }
  }, [open]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return commands.filter((c) => !q || c.title.toLowerCase().includes(q) || c.id.includes(q));
  }, [commands, query]);

  useEffect(() => {
    setActive(0);
  }, [query]);

  if (!open) return null;
  const optionId = (index: number) => `command-option-${index}`;
  const run = (command: PaletteCommand | undefined) => {
    if (!command || command.disabled) return;
    onClose();
    command.run();
  };

  return createPortal(
    <div className="confirm-overlay" role="presentation" onMouseDown={onClose}>
      <div
        ref={dialogRef}
        className="command-palette"
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <input
          ref={inputRef}
          className="command-input"
          role="combobox"
          aria-expanded="true"
          aria-controls="command-list"
          aria-activedescendant={filtered.length ? optionId(active) : undefined}
          aria-autocomplete="list"
          aria-label="Search commands"
          placeholder="Type a command…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'ArrowDown') {
              event.preventDefault();
              setActive((i) => Math.min(i + 1, filtered.length - 1));
            } else if (event.key === 'ArrowUp') {
              event.preventDefault();
              setActive((i) => Math.max(i - 1, 0));
            } else if (event.key === 'Enter') {
              event.preventDefault();
              run(filtered[active]);
            }
          }}
        />
        <ul id="command-list" className="command-list" role="listbox" aria-label="Commands">
          {filtered.map((command, index) => (
            <li
              key={command.id}
              id={optionId(index)}
              role="option"
              aria-selected={index === active}
              aria-disabled={command.disabled || undefined}
              className={`command-item${command.disabled ? ' disabled' : ''}`}
              onMouseEnter={() => setActive(index)}
              onMouseDown={(e) => e.preventDefault()}
              onClick={() => run(command)}
            >
              <span className="command-title">{command.title}</span>
              {command.scope && <span className="command-scope">{command.scope}</span>}
              {command.shortcut && <kbd className="command-shortcut">{command.shortcut}</kbd>}
            </li>
          ))}
          {filtered.length === 0 && <li className="command-empty">No matching commands</li>}
        </ul>
      </div>
    </div>,
    document.body,
  );
}
