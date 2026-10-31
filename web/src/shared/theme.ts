export type Theme = 'dark' | 'light';

export const THEME_STORAGE_KEY = 'omnim.theme';

/** Resolve the initial theme from an explicit choice, else the OS preference. */
export function resolveInitialTheme(
  stored: string | null,
  prefersLight: boolean,
): Theme {
  if (stored === 'dark' || stored === 'light') return stored;
  return prefersLight ? 'light' : 'dark';
}

export function loadTheme(): Theme {
  if (typeof window === 'undefined') return 'dark';
  const prefersLight = window.matchMedia?.('(prefers-color-scheme: light)').matches ?? false;
  return resolveInitialTheme(window.localStorage.getItem(THEME_STORAGE_KEY), prefersLight);
}

export function saveTheme(theme: Theme): void {
  if (typeof window === 'undefined') return;
  window.localStorage.setItem(THEME_STORAGE_KEY, theme);
}

/** The single DOM write for theming: components read CSS custom properties. */
export function applyTheme(theme: Theme, root: HTMLElement | null = null): void {
  const el = root ?? (typeof document === 'undefined' ? null : document.documentElement);
  if (!el) return;
  el.dataset.theme = theme;
}
