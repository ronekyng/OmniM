import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../shared/api';
import type { CurrentProject, ProjectSummary } from '../shared/types';
import { useAppStore } from './store/appStore';
import { useToastApi } from '../shared/toast';
import {
  formatBytes,
  loadRecent,
  removeRecent,
  saveRecent,
  touchRecent,
  type RecentProject,
} from '../shared/recentProjects';

type Row = {
  path: string;
  name: string;
  size?: number;
  modified?: number | null;
  lastOpened?: number;
  missing: boolean;
};

function mergeRows(projects: ProjectSummary[], recent: RecentProject[]): Row[] {
  const byPath = new Map<string, Row>();
  for (const project of projects) {
    byPath.set(project.path, {
      path: project.path,
      name: project.name,
      size: project.size,
      modified: project.modified,
      missing: false,
    });
  }
  const rows: Row[] = [];
  for (const entry of recent) {
    const existing = byPath.get(entry.path);
    if (existing) {
      rows.push({ ...existing, lastOpened: entry.lastOpened });
      byPath.delete(entry.path);
    } else {
      rows.push({ ...entry, missing: true });
    }
  }
  const rest = [...byPath.values()].sort((a, b) => (b.modified ?? 0) - (a.modified ?? 0));
  return [...rows, ...rest];
}

function uniqueProjectName(base: string, existing: string[]): string {
  const taken = new Set(existing.map((n) => n.toLowerCase()));
  if (!taken.has(base.toLowerCase())) return base;
  let n = 2;
  while (taken.has(`${base} ${n}`.toLowerCase())) n += 1;
  return `${base} ${n}`;
}

function formatWhen(seconds: number | null | undefined): string {
  if (!seconds) return '—';
  return new Date(seconds * 1000).toLocaleString();
}

/** Project home (UI-DAT-01): new / open / recent / sample. */
export function ProjectsPage() {
  const navigate = useNavigate();
  const toast = useToastApi();
  const refresh = useAppStore((s) => s.refresh);
  const reloadScene = useAppStore((s) => s.reloadScene);
  const resetSaveState = useAppStore((s) => s.resetSaveState);

  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  const [recent, setRecent] = useState<RecentProject[]>(() => loadRecent());
  const [current, setCurrent] = useState<CurrentProject | null>(null);
  const [name, setName] = useState('Untitled');
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setError(null);
    try {
      const [list, session] = await Promise.all([api.listProjects(), api.currentProject()]);
      setProjects(list);
      setCurrent(session);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  const rows = useMemo(() => mergeRows(projects, recent), [projects, recent]);

  const remember = useCallback((entry: RecentProject) => {
    setRecent((prev) => {
      const next = touchRecent(prev, entry);
      saveRecent(next);
      return next;
    });
  }, []);

  const enterWorkspace = useCallback(async (project: CurrentProject) => {
    remember({
      path: project.path ?? project.name,
      name: project.name,
      lastOpened: Date.now(),
    });
    resetSaveState();
    await refresh();
    reloadScene();
    navigate('/');
  }, [navigate, refresh, reloadScene, remember, resetSaveState]);

  const create = useCallback(async (projectName: string, sample: boolean) => {
    setBusy(sample ? 'sample' : 'new');
    setError(null);
    try {
      const resolved = uniqueProjectName(projectName.trim() || 'Untitled', projects.map((p) => p.name));
      const project = await api.newProject(resolved, sample);
      toast.success(`Created ${project.name}`);
      await enterWorkspace(project);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      toast.error(`New project failed: ${message}`);
    } finally {
      setBusy(null);
    }
  }, [enterWorkspace, projects, toast]);

  const open = useCallback(async (path: string) => {
    setBusy(path);
    setError(null);
    try {
      const project = await api.openProject(path);
      toast.success(`Opened ${project.name}`);
      await enterWorkspace(project);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      toast.error(`Open failed: ${message}`);
    } finally {
      setBusy(null);
    }
  }, [enterWorkspace, toast]);

  const forget = useCallback((path: string) => {
    setRecent((prev) => {
      const next = removeRecent(prev, path);
      saveRecent(next);
      return next;
    });
  }, []);

  return (
    <div className="projects-page">
      <header className="projects-header">
        <div>
          <h1>OmniM Projects</h1>
          <p>Choose a project file or start a new model.</p>
        </div>
        <div className="projects-local-note" role="note">
          Local server, no authentication. Do not expose this port to a network.
        </div>
      </header>

      <section className="projects-grid">
        <div className="projects-panel">
          <h2>New project</h2>
          <form
            className="projects-new-form"
            onSubmit={(event) => {
              event.preventDefault();
              void create(name, false);
            }}
          >
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              aria-label="Project name"
              placeholder="Project name"
            />
            <button type="submit" className="primary" disabled={busy !== null || !name.trim()}>
              {busy === 'new' ? 'Creating…' : 'Create'}
            </button>
          </form>
          <button
            type="button"
            className="sample-button"
            disabled={busy !== null}
            onClick={() => void create('Sample Project', true)}
          >
            {busy === 'sample' ? 'Creating sample…' : 'Open sample project (2 storeys + grid)'}
          </button>

          {current?.open && (
            <div className="projects-current">
              <span className="projects-current-label">Current session</span>
              <strong>{current.name}</strong>
              <span>{current.path ?? 'in memory'}</span>
              <button type="button" onClick={() => navigate('/')}>Return to workspace</button>
            </div>
          )}
        </div>

        <div className="projects-panel">
          <h2>Recent and available projects</h2>
          {error && <p className="error" role="alert">{error}</p>}
          {rows.length === 0 ? (
            <div className="projects-empty">
              <p>No projects yet. Create one above or open the sample.</p>
            </div>
          ) : (
            <table className="projects-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Last opened / modified</th>
                  <th>Size</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {rows.map((row) => (
                  <tr key={row.path} className={row.missing ? 'missing' : undefined}>
                    <td>
                      <strong>{row.name}</strong>
                      <div className="projects-path">{row.path}</div>
                      {row.missing && <span className="projects-badge">Missing file</span>}
                    </td>
                    <td>{row.lastOpened ? formatWhen(row.lastOpened / 1000) : formatWhen(row.modified)}</td>
                    <td>{formatBytes(row.size)}</td>
                    <td className="projects-actions">
                      {!row.missing && (
                        <button type="button" onClick={() => void open(row.path)} disabled={busy !== null}>
                          {busy === row.path ? 'Opening…' : 'Open'}
                        </button>
                      )}
                      <button type="button" onClick={() => forget(row.path)}>Remove</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </section>
    </div>
  );
}
