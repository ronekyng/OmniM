import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/**
 * G1 smoke: project home → sample workspace → theme toggle → left-dock tabs
 * (Objects / Grids+levels / Catalog) → create and read back a storey.
 *
 * Evidence for UI-DAT-01, UI-DS-02, UI-SHL-04 and the storey container
 * regression fix.
 */
test('sample project: theme, dock tabs and storey creation', async ({ page }) => {
  await page.goto('/projects');
  await expect(page.getByRole('heading', { name: 'OmniM Projects' })).toBeVisible();

  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await expect(page.getByRole('tab', { name: /对象/ })).toBeVisible();

  // Theme toggle writes <html data-theme> and persists it.
  const html = page.locator('html');
  const before = await html.getAttribute('data-theme');
  await page.getByRole('button', { name: /切换到(浅色|深色)主题/ }).click();
  await expect
    .poll(async () => html.getAttribute('data-theme'), { timeout: 5_000 })
    .not.toBe(before);
  const after = await html.getAttribute('data-theme');
  expect(after === 'light' || after === 'dark').toBe(true);

  // Left dock: levels live next to grids (IA_SHELL §3.4).
  await page.getByRole('tab', { name: '轴网' }).click();
  await expect(page.getByText('楼层 Levels')).toBeVisible();
  await page.getByLabel('Name').fill('E2E-Level');
  await page.getByLabel('Elevation (m)').fill('4.5');
  await page.getByRole('button', { name: 'Add storey' }).click();
  await expect(page.locator('.grid-line-row', { hasText: 'E2E-Level' })).toBeVisible();

  // The storey view selector picks the new datum up.
  const storeySelect = page.locator('.viewport-storey-control select');
  await expect(storeySelect.locator('option', { hasText: 'E2E-Level' })).toHaveCount(1);

  // Catalog tab is embedded, not a second floating panel.
  await page.getByRole('tab', { name: '构件库' }).click();
  await expect(page.getByRole('heading', { name: '可用构件' })).toBeVisible();
});
