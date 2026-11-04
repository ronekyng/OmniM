import { expect, test } from '@playwright/test';

/** IA_SHELL §7 (UI-SHL-11): 1024–1279 px folds both docks into 44 px icon
 * strips; the right dock re-opens via a drawer. */

test.use({ viewport: { width: 1200, height: 768 } });

test('narrow band folds docks to icon strips and opens the drawer', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  await expect(page.locator('.app-shell')).toHaveClass(/narrow-shell/);

  // The left icons re-open their panel: open the Objects drawer and select an
  // object so the properties surface registers for the right strip.
  await page.locator('#left-tab-objects').click();
  const leftDrawer = page.locator('[data-testid="left-canvas-drawer"]');
  await expect(leftDrawer).toBeVisible();
  await leftDrawer.getByRole('tree', { name: 'Object tree' }).getByRole('treeitem').first().click();
  await expect(page.locator('.right-icon-strip')).toBeVisible();

  // Right dock folded: drawer closed until a strip icon is clicked.
  await expect(page.locator('[data-testid="right-drawer"]')).toHaveCount(0);
  const icon = page.locator('.right-icon-strip .strip-icon').first();
  await icon.click();
  const drawer = page.locator('[data-testid="right-drawer"]');
  await expect(drawer).toBeVisible();
  await expect(drawer.locator('.properties-panel').first()).toBeVisible();
  expect(await drawer.boundingBox().then((b) => b.width)).toBeLessThanOrEqual(360);

  // Left dock folded to a 44 px strip.
  expect(
    await page.locator('.sidebar').boundingBox().then((b) => b.width),
  ).toBeLessThanOrEqual(50);

  // Icon carries a name and expanded state (a11y contract preserved).
  await expect(icon).toHaveAttribute('aria-expanded', 'true');

  // Back to ≥1280: full docked layout returns, drawer closes.
  await page.setViewportSize({ width: 1600, height: 900 });
  await expect(page.locator('.app-shell')).not.toHaveClass(/narrow-shell/);
  await expect(page.locator('[data-testid="right-drawer"]')).toHaveCount(0);
  await expect(page.locator('.right-icon-strip')).toHaveCount(0);
});

test('CMP-05: at exactly 1280×720 the full docked layout stays usable', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // ≥1280 px: normal docked shell, no folding classes or notices.
  await expect(page.locator('.app-shell')).not.toHaveClass(/narrow-shell/);
  await expect(page.locator('.min-width-notice')).toHaveCount(0);

  // Core path remains reachable: select, the docked properties panel shows.
  const tree = page.getByRole('tree', { name: 'Object tree' });
  await tree.getByRole('treeitem').first().click();
  await expect(page.locator('.properties-panel').first()).toBeVisible();
  expect(
    await page.locator('.right-panel').boundingBox().then((b) => b.width),
  ).toBeGreaterThan(260);
});

test('<1024 keeps the blocking notice (workspace route)', async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 700 });
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.min-width-notice')).toBeVisible({ timeout: 20_000 });
});
