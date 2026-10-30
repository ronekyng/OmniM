import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAppStore } from '../store/appStore';
import { saveLabel, shouldWarnBeforeUnload } from '../../shared/saveState';
import { useConfirm } from './ConfirmDialog';

/** Workspace title bar: project name, dirty mark, breadcrumb, explicit save.
 *  Contract: IA_SHELL §2.2, DATA_INTEGRITY §3. */
export function TitleBar() {
  const navigate = useNavigate();
  const confirm = useConfirm();
  const stats = useAppStore((s) => s.stats);
  const saveState = useAppStore((s) => s.saveState);
  const saveProject = useAppStore((s) => s.saveProject);
  const saveProjectAs = useAppStore((s) => s.saveProjectAs);
  const resetLayout = useAppStore((s) => s.resetLayout);
  const transformMode = useAppStore((s) => s.transformMode);
  const creationPhase = useAppStore((s) => s.creationPhase);
  const creationObjectType = useAppStore((s) => s.creationObjectType);
  const [saveAsOpen, setSaveAsOpen] = useState(false);
  const [path, setPath] = useState('project.omnim');

  const name = stats?.name || 'Untitled';
  const dirty = saveState.status !== 'clean';
  const tool = creationPhase !== 'idle'
    ? `Create ${creationObjectType ?? ''}`.trim()
    : transformMode
      ? 'Transform'
      : 'Modeling';

  useEffect(() => {
    document.title = `${dirty ? '• ' : ''}${name} — OmniM`;
  }, [dirty, name]);

  useEffect(() => {
    if (!saveAsOpen) return;
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setSaveAsOpen(false);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [saveAsOpen]);

  const openProjects = async () => {
    if (shouldWarnBeforeUnload(saveState)) {
      const ok = await confirm({
        title: 'Leave without saving?',
        message: 'The workspace has changes that were not explicitly saved in this session.',
        confirmLabel: 'Leave',
        cancelLabel: 'Stay',
        danger: true,
      });
      if (!ok) return;
    }
    navigate('/projects');
  };

  return (
    <div className="title-bar">
      <button type="button" className="title-project" title="Back to projects" onClick={() => void openProjects()}>
        {dirty && <span className="dirty-dot" aria-label="Unsaved changes" />}
        <strong>{name}</strong>
      </button>
      <nav className="breadcrumb" aria-label="Breadcrumb">
        <span>Workspace</span>
        <span className="breadcrumb-sep">/</span>
        <span aria-current="page">{tool}</span>
      </nav>
      <span className={`save-indicator status-${saveState.status}`} title={saveState.error ?? undefined}>
        {saveState.status === 'clean' ? '' : saveLabel(saveState)}
      </span>
      <div className="title-actions">
        <button type="button" onClick={() => void saveProject()} disabled={saveState.status === 'saving'} title="Save (Ctrl/Cmd+S)">
          Save
        </button>
        <button type="button" onClick={() => setSaveAsOpen(true)}>Save as</button>
        <span className="title-sep" />
        <button type="button" onClick={resetLayout} title="Reset docks to defaults">Reset layout</button>
      </div>

      {saveAsOpen && (
        <div className="confirm-overlay" role="presentation" onMouseDown={() => setSaveAsOpen(false)}>
          <form
            className="confirm-dialog"
            role="dialog"
            aria-modal="true"
            aria-labelledby="save-as-title"
            onMouseDown={(e) => e.stopPropagation()}
            onSubmit={(event) => {
              event.preventDefault();
              void saveProjectAs(path).then((ok) => { if (ok) setSaveAsOpen(false); });
            }}
          >
            <h2 id="save-as-title">Save project as</h2>
            <p>Path on the server, relative or absolute. Must end with <code>.omnim</code>.</p>
            <input
              autoFocus
              value={path}
              onChange={(e) => setPath(e.target.value)}
              aria-label="Project file path"
            />
            <div className="confirm-actions">
              <button type="button" onClick={() => setSaveAsOpen(false)}>Cancel</button>
              <button type="submit" className="primary" disabled={!path.trim()}>Save</button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
