import { test } from '@playwright/test';
import { writeFileSync } from 'node:fs';

/** UI-A11Y-05 review aid: snapshots the accessibility tree of the key
 *  surfaces into `/tmp/opencode/a11y-snapshots.json` so a human reviewer can
 *  compare the live screen-reader experience with the shipped names/roles.
 *  This spec asserts nothing about quality — the manual kit does. */
test('captures aria snapshots for the manual review', async ({ page }) => {
  const out: Record<string, string> = {};

  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await page.locator('.title-bar').waitFor({ timeout: 20_000 });

  // Seed geometry so the viewport description has a non-zero object count.
  await page.request.post('http://127.0.0.1:3100/api/genrule', {
    data: { genrule: 'wall', name: 'A11yWall', length: 5, height: 3, core_thickness: 0.2, finish_thickness: 0.02 },
  });
  await page.reload();
  await page.locator('.title-bar').waitFor({ timeout: 20_000 });

  out.titleBar = await page.locator('.title-bar').ariaSnapshot();
  out.viewport = page.locator('.viewer-content');
  out.viewportDescription = (await page.locator('#viewport-description').textContent()) ?? '';
  out.leftDock = await page.locator('aside').first().ariaSnapshot();
  const propsPanel = page.locator('.properties-panel').first();
  out.properties = (await propsPanel.count()) > 0 ? await propsPanel.ariaSnapshot() : '';

  // Describe-scene action announcement.
  const statusRegion = page.locator('.toast-container');
  const before = (await statusRegion.textContent({ timeout: 3000 }).catch(() => '')) ?? '';
  await page.keyboard.press('ControlOrMeta+K');
  const palette = page.getByRole('dialog', { name: 'Command palette' });
  await palette.getByRole('combobox', { name: 'Search commands' }).fill('describe scene');
  await page.keyboard.press('Enter');
  await page.waitForTimeout(300);
  const after = (await statusRegion.textContent({ timeout: 3000 }).catch(() => '')) ?? '';
  out.statusRegion = after !== before ? after : before;

  try {
    writeFileSync('/tmp/opencode/a11y-snapshots.json', JSON.stringify(out, null, 2));
  } catch { /* non-fatal */ }
});
