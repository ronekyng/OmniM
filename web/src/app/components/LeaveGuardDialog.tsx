import { useRef } from 'react';
import { createPortal } from 'react-dom';
import { useDialogFocus } from './useDialogFocus';
import { useI18n } from '../../shared/i18n';
import { formatClock } from '../../shared/saveState';

/** DATA_INTEGRITY §3.3: three-way unsaved-changes interception.
 *
 * Choices: Save (safe default, Enter) · Discard (danger) · Cancel (Esc).
 * The dialog names the project and the last explicit save time.
 */
export function LeaveGuardDialog({
  open,
  projectName,
  lastSavedAt,
  busy,
  onSave,
  onDiscard,
  onCancel,
}: {
  open: boolean;
  projectName: string;
  lastSavedAt: number | null;
  busy: boolean;
  onSave: () => void;
  onDiscard: () => void;
  onCancel: () => void;
}) {
  const { t } = useI18n();
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const saveButtonRef = useRef<HTMLButtonElement | null>(null);
  // Esc closes (cancel) and the background is inert while open.
  useDialogFocus(open, dialogRef, saveButtonRef, onCancel);
  if (!open) return null;

  return createPortal(
    <div className="confirm-overlay" role="presentation" onMouseDown={onCancel}>
      <div
        ref={dialogRef}
        className="confirm-dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="leave-title"
        aria-describedby="leave-message"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <h2 id="leave-title">{t('shell.leave.title')}</h2>
        <p id="leave-message">{t('shell.leave.message')}</p>
        <ul className="confirm-details">
          <li>{t('shell.leave.project', { name: projectName })}</li>
          <li>
            {t('shell.leave.last_saved', {
              time: lastSavedAt ? formatClock(lastSavedAt) : t('shell.leave.never'),
            })}
          </li>
        </ul>
        {/* A form makes Enter submit Save — the safe default per §3.3 rule 3. */}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            if (!busy) onSave();
          }}
        >
          <div className="confirm-actions">
            <button type="button" onClick={onCancel} disabled={busy}>
              {t('shell.leave.cancel')}
            </button>
            <button type="button" className="danger" onClick={onDiscard} disabled={busy}>
              {t('shell.leave.confirm')}
            </button>
            <button ref={saveButtonRef} type="submit" className="primary" disabled={busy}>
              {busy ? t('shell.save.saving') : t('shell.leave.save')}
            </button>
          </div>
        </form>
      </div>
    </div>,
    document.body,
  );
}
