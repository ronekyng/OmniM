import { useEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import { useNavigate } from 'react-router-dom';
import { useAppStore } from '../store/appStore';
import { saveLabelKey, shouldWarnBeforeUnload } from '../../shared/saveState';
import { useConfirm } from './ConfirmDialog';
import { useDialogFocus } from './useDialogFocus';
import { IcoMoon, IcoSun } from './Icons';
import { api } from '../../shared/api';
import type { ProjectMeta } from '../../shared/types';
import { notify } from '../../shared/toast';
import { useI18n, type Locale } from '../../shared/i18n';

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
  const theme = useAppStore((s) => s.theme);
  const setTheme = useAppStore((s) => s.setTheme);
  const { t, locale, setLocale } = useI18n();
  const transformMode = useAppStore((s) => s.transformMode);
  const creationPhase = useAppStore((s) => s.creationPhase);
  const creationObjectType = useAppStore((s) => s.creationObjectType);
  const [saveAsOpen, setSaveAsOpen] = useState(false);
  const [path, setPath] = useState('project.omnim');
  // 项目设置（DATA_INTEGRITY §2.3, UI-DAT-07）
  const [metaOpen, setMetaOpen] = useState(false);
  const [meta, setMeta] = useState<ProjectMeta | null>(null);
  const [metaSaving, setMetaSaving] = useState(false);
  const saveAsDialogRef = useRef<HTMLFormElement | null>(null);
  const saveAsInputRef = useRef<HTMLInputElement | null>(null);
  useDialogFocus(saveAsOpen, saveAsDialogRef, saveAsInputRef, () => setSaveAsOpen(false));

  const name = stats?.name || 'Untitled';
  const dirty = saveState.status !== 'clean';
  const tool = creationPhase !== 'idle'
    ? `${t('shell.breadcrumb.create')} ${creationObjectType ?? ''}`.trim()
    : transformMode
      ? t('shell.breadcrumb.transform')
      : t('shell.breadcrumb.modeling');

  useEffect(() => {
    document.title = `${dirty ? '• ' : ''}${name} ${t('shell.titlebar.doc_prefix')}`;
  }, [dirty, name]);

  const openProjects = async () => {
    if (shouldWarnBeforeUnload(saveState)) {
      const ok = await confirm({
        title: t('shell.leave.title'),
        message: t('shell.leave.message'),
        confirmLabel: t('shell.leave.confirm'),
        cancelLabel: t('shell.leave.cancel'),
        danger: true,
      });
      if (!ok) return;
    }
    navigate('/projects');
  };

  const openMeta = async () => {
    setMetaOpen(true);
    try {
      setMeta(await api.projectMeta());
    } catch (e) {
      notify.error(e instanceof Error ? e.message : String(e));
    }
  };

  const saveMeta = async () => {
    if (!meta) return;
    setMetaSaving(true);
    try {
      const saved = await api.updateProjectMeta({
        name: meta.name,
        description: meta.description,
        author: meta.author,
        unit_system: meta.unit_system,
      });
      setMeta(saved);
      notify.success('Project settings saved');
      useAppStore.getState().refresh();
      useAppStore.getState().markDirty();
      setMetaOpen(false);
    } catch (e) {
      notify.error(e instanceof Error ? e.message : String(e));
    }
    setMetaSaving(false);
  };

  return (
    <div className="title-bar">
      <h1 className="sr-only" data-page-heading>{name}{t('shell.titlebar.workspace_suffix')}</h1>
      <button type="button" className="title-project" title={t('shell.titlebar.back_to_projects')} onClick={() => void openProjects()}>
        {dirty && <span className="dirty-dot" aria-label={t('shell.titlebar.dirty_hint')} />}
        <strong>{name}</strong>
      </button>
      <nav className="breadcrumb" aria-label="Breadcrumb">
        <span>{t('shell.breadcrumb.workspace')}</span>
        <span className="breadcrumb-sep">/</span>
        <span aria-current="page">{tool}</span>
      </nav>
      <span className={`save-indicator status-${saveState.status}`} title={saveState.error ?? undefined}>
        {(() => {
          const { key } = saveLabelKey(saveState);
          if (saveState.status === 'clean') return '';
          return t(key);
        })()}
      </span>
      <div className="title-actions">
        {/* I18N §3: 即时切换、不重载、`<html lang>` 同步。语言以自身语言命名。 */}
        <select
          className="lang-switch"
          value={locale}
          onChange={(e) => setLocale(e.target.value as Locale)}
          aria-label={t('shell.language.switcher')}
          title={t('shell.language.switcher')}
        >
          <option value="zh-CN">中文</option>
          <option value="en-US">English</option>
        </select>
        <button
          type="button"
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
          title={theme === 'dark' ? t('shell.theme.switch_to_light') : t('shell.theme.switch_to_dark')}
          aria-label={theme === 'dark' ? t('shell.theme.switch_to_light') : t('shell.theme.switch_to_dark')}
        >
          {theme === 'dark' ? <IcoSun /> : <IcoMoon />}
        </button>
        <button type="button" onClick={() => void saveProject()} disabled={saveState.status === 'saving'} title={t('shell.action.save_shortcut')}>
          {t('shell.action.save')}
        </button>
        <button type="button" onClick={() => setSaveAsOpen(true)}>{t('shell.action.save_as')}</button>
        <span className="title-sep" />
        <button type="button" onClick={() => void openMeta()}>{t('shell.meta.button')}</button>
        <button type="button" onClick={resetLayout} title={t('shell.action.reset_layout')}>{t('shell.action.reset_layout')}</button>
      </div>

      {saveAsOpen &&
        createPortal(
          <div className="confirm-overlay" role="presentation" onMouseDown={() => setSaveAsOpen(false)}>
            <form
              ref={saveAsDialogRef}
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
              <h2 id="save-as-title">{t('shell.saveas.title')}</h2>
              <p>{t('shell.saveas.hint')}<code>.omnim</code>{t('shell.saveas.hint_ext')}</p>
              <input
                ref={saveAsInputRef}
                value={path}
                onChange={(e) => setPath(e.target.value)}
                aria-label={t('shell.saveas.path_label')}
              />
              <div className="confirm-actions">
                <button type="button" onClick={() => setSaveAsOpen(false)}>{t('shell.saveas.cancel')}</button>
                <button type="submit" className="primary" disabled={!path.trim()}>{t('shell.action.save')}</button>
              </div>
            </form>
          </div>,
          document.body,
        )}

      {metaOpen &&
        createPortal(
          <div className="confirm-overlay" role="presentation" onMouseDown={() => setMetaOpen(false)}>
            <form
              className="confirm-dialog"
              role="dialog"
              aria-modal="true"
              aria-labelledby="meta-title"
              onMouseDown={(e) => e.stopPropagation()}
              onSubmit={(event) => {
                event.preventDefault();
                void saveMeta();
              }}
            >
              <h2 id="meta-title">{t('shell.meta.title')}</h2>
              {meta && (
                <>
                  <div className="form-actions meta-grid">
                    <label>{t('shell.meta.name')}<input value={meta.name} onChange={(e) => setMeta({ ...meta, name: e.target.value })} aria-label={t('shell.meta.name')} /></label>
                    <label>{t('shell.meta.description')}<textarea rows={2} value={meta.description} onChange={(e) => setMeta({ ...meta, description: e.target.value })} aria-label={t('shell.meta.description')} /></label>
                    <label>{t('shell.meta.author')}<input value={meta.author} onChange={(e) => setMeta({ ...meta, author: e.target.value })} aria-label={t('shell.meta.author')} /></label>
                    <label>{t('shell.meta.units')}
                      <select value={meta.unit_system} onChange={(e) => setMeta({ ...meta, unit_system: e.target.value })} aria-label={t('shell.meta.units')}>
                        <option value="metric">metric (m)</option>
                        <option value="imperial">imperial</option>
                      </select>
                    </label>
                  </div>
                  <p className="hint">{t('shell.meta.readonly')}</p>
                </>
              )}
              <div className="confirm-actions">
                <button type="button" onClick={() => { setMetaOpen(false); setMeta(null); }}>{t('shell.saveas.cancel')}</button>
                <button type="submit" className="primary" disabled={metaSaving}>{t('shell.action.save')}</button>
              </div>
            </form>
          </div>,
          document.body,
        )}
    </div>
  );
}
