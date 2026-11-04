import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/** UI-SHL-07: contextual ribbon row always visible; switching the discipline
 *  replaces the command set instead of hiding it. */
test('ribbon keeps the command row visible across disciplines', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const row = page.locator('.domain-toolbar');
  await expect(row).toBeVisible();

  // Building by default.
  await expect(row.locator('button', { hasText: '墙体' })).toBeVisible();

  // Switching discipline replaces the commands.
  await page.locator('.domain-switch', { hasText: '土木' }).click();
  await expect(row).toBeVisible();
  await expect(row.locator('button', { hasText: '道路' })).toBeVisible();
  await expect(row.locator('button', { hasText: '墙体' })).toHaveCount(0);

  await page.locator('.domain-switch', { hasText: '工厂' }).click();
  await expect(row.locator('button', { hasText: '储罐' })).toBeVisible();

  await page.locator('.domain-switch', { hasText: '机械' }).click();
  await expect(row.locator('button', { hasText: '零件' })).toBeVisible();
});
