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

function slugifyProjectName(name: string): string {
  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return slug || 'project';
}

function uniqueProjectName(base: string, existing: { name: string; path: string }[]): string {
  // Display names and file slugs are different keys: an orphaned file can
  // occupy sample-project.omnim while its internal name is something else.
  const names = new Set(existing.map((p) => p.name.toLowerCase()));
  const slugs = new Set(
    existing.map((p) => p.path.split(/[\\/]/).pop()!.replace(/\.omnim$/i, '').toLowerCase()),
  );
  let candidate = base;
  let n = 2;
  while (names.has(candidate.toLowerCase()) || slugs.has(slugifyProjectName(candidate))) {
    candidate = `${base} ${n}`;
    n += 1;
  }
  return candidate;
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
  // Creating before the list loads would pick a name that slugifies onto an
  // existing file (the sample button used to race into a 409 on warm dirs).
  const [loaded, setLoaded] = useState(false);

  const reload = useCallback(async () => {
    setError(null);
    try {
      const [list, session] = await Promise.all([api.listProjects(), api.currentProject()]);
      setProjects(list);
      setCurrent(session);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoaded(true);
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
    // Session-open must be true before the workspace route guards run (App's
    // currentProject() effect redirects to /projects while it is still false).
    await refresh();
    reloadScene();
    navigate('/');
  }, [navigate, refresh, reloadScene, remember, resetSaveState]);

  const create = useCallback(async (projectName: string, sample: boolean) => {
    if (!loaded) {
      setError('Project list is still loading. Try again in a moment.');
      return;
    }
    setBusy(sample ? 'sample' : 'new');
    setError(null);
    try {
      const baseName = projectName.trim() || 'Untitled';
      let resolved = uniqueProjectName(baseName, rows);
      let attempt = 2;
      let project;
      // The server keys projects by file slug. A stale/orphaned file can still
      // collide after the client-side check, so resolve a 409 by suffixing.
      for (;;) {
        try {
          project = await api.newProject(resolved, sample);
          break;
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          if (!message.startsWith('409 ') || attempt > 100) throw err;
          resolved = `${baseName} ${attempt}`;
          attempt += 1;
        }
      }
      toast.success(`Created ${project.name}`);
      await enterWorkspace(project);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      toast.error(`New project failed: ${message}`);
    } finally {
      setBusy(null);
    }
  }, [enterWorkspace, loaded, rows, toast]);

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
            <button type="submit" className="primary" disabled={busy !== null || !loaded || !name.trim()}>
              {busy === 'new' ? 'Creating…' : 'Create'}
            </button>
          </form>
          <button
            type="button"
            className="sample-button"
            disabled={busy !== null || !loaded}
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
                        <button type="button" onClick={() => void open(row.path)} disabled={busy !== null || !loaded}>
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
