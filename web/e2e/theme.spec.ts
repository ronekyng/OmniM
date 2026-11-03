import { expect, test, type Page } from '@playwright/test';

/**
 * UI-DS-02: theme switches in one frame without reload, persists an explicit
 * choice, and follows the OS preference when none is stored. Text/background
 * pairs on the critical path must keep WCAG 2.2 AA contrast in both themes.
 */

type Pair = { fg: string; bg: string; label: string; large?: boolean };

async function computedPairs(page: Page, selectors: string[]): Promise<Pair[]> {
  return page.evaluate((sels) => {
    const parse = (value: string) => {
      const m = value.match(/rgba?\(([^)]+)\)/);
      if (!m) return null;
      const parts = m[1].split(',').map((v) => parseFloat(v));
      return { r: parts[0], g: parts[1], b: parts[2], a: parts[3] ?? 1 };
    };
    const blend = (top: number, bottom: number, alpha: number) =>
      Math.round(top * alpha + bottom * (1 - alpha));
    const samples: Pair[] = [];
    for (const sel of sels) {
      const el = document.querySelector(sel) as HTMLElement | null;
      if (!el) continue;
      const style = getComputedStyle(el);
      const fg = parse(style.color);
      let node: HTMLElement | null = el;
      let bg = null as ReturnType<typeof parse>;
      while (node) {
        const candidate = parse(getComputedStyle(node).backgroundColor);
        if (candidate && candidate.a > 0) {
          bg = candidate;
          break;
        }
        node = node.parentElement;
      }
      if (!fg || !bg) continue;
      const base = bg.r + ',' + bg.g + ',' + bg.b;
      const alpha = fg.a;
      const rr = blend(fg.r, bg.r, alpha);
      const gg = blend(fg.g, bg.g, alpha);
      const bb = blend(fg.b, bg.b, alpha);
      samples.push({
        fg: `${rr},${gg},${bb}`,
        bg: base,
        label: sel,
        large: parseFloat(style.fontSize) >= 18.66 || parseFloat(style.fontWeight) >= 700,
      });
    }
    return samples;
  }, selectors);
}

function luminance(rgb: string): number {
  const [r, g, b] = rgb.split(',').map((v) => {
    const c = parseFloat(v) / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(pair: Pair): number {
  const a = luminance(pair.fg);
  const b = luminance(pair.bg);
  const [hi, lo] = a > b ? [a, b] : [b, a];
  return (hi + 0.05) / (lo + 0.05);
}

const CRITICAL = [
  'body',
  '.title-bar',
  '.breadcrumb-sep',
  '.sidebar .tree-node',
  '.tree-meta',
  '.main-toolbar button',
  '.domain-toolbar button',
  '.bottom-panel .omni-tab',
  '.status-bar',
  '.hint',
];

const DIALOG = [
  '.confirm-dialog h2',
  '.confirm-dialog p',
  '.confirm-dialog input',
  '.confirm-actions button.primary',
  '.confirm-actions button:not(.primary):not(.danger)',
];

test('theme follows OS, switches without reload, persists, and stays legible', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  // 1. No stored choice: the OS preference decides.
  await page.emulateMedia({ colorScheme: 'light' });
  await page.goto('/projects');
  await page.evaluate(() => localStorage.removeItem('omnim.theme'));
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

  // 2. Open the workspace and switch from the title bar.
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await page.evaluate(() => {
    (window as unknown as { __noReloadMarker: number }).__noReloadMarker = 42;
  });
  const before = await page.evaluate(() => performance.getEntriesByType('navigation').length);

  await page.getByRole('button', { name: 'Switch to light theme' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  expect(await page.evaluate(() => (window as unknown as { __noReloadMarker: number }).__noReloadMarker)).toBe(42);
  expect(await page.evaluate(() => performance.getEntriesByType('navigation').length)).toBe(before);
  expect(await page.evaluate(() => localStorage.getItem('omnim.theme'))).toBe('light');
  await expect(page.getByRole('button', { name: 'Switch to dark theme' })).toBeVisible();

  // 3. Explicit choice survives a reload and beats the OS preference.
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');

  // 4. Contrast audit in both themes.
  for (const theme of ['light', 'dark'] as const) {
    if (theme === 'dark') {
      await page.getByRole('button', { name: 'Switch to dark theme' }).click();
      await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    }
    await page.waitForTimeout(250);
    let pairs = await computedPairs(page, CRITICAL);
    expect(pairs.length).toBeGreaterThan(0);

    await page.getByRole('button', { name: 'Save as' }).click();
    pairs = pairs.concat(await computedPairs(page, DIALOG));
    await page.keyboard.press('Escape');

    const failures = pairs
      .map((pair) => ({ ...pair, ratio: contrast(pair) }))
      .filter((pair) => pair.ratio < (pair.large ? 3 : 4.5));
    expect(failures, JSON.stringify(failures, null, 2)).toEqual([]);
  }
});
