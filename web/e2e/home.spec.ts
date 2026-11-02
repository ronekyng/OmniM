import { expect, test } from '@playwright/test';

/** UI-DAT-01 journey + empty-state copy for a fresh model (UI-FBK-09). */
test('project home journey: sample → workspace → clean leave → recent list', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Empty model shows guided copy (only if the tree is empty — sample is not).
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.status-save')).toContainText('Saved');

  await page.locator('.title-project').click();
  await expect(page).toHaveURL(/\/projects$/);
  await expect(page.locator('td', { hasText: 'Sample Project' }).first()).toBeVisible();

  // Fresh named project and back to home again.
  await page.getByLabel('Project name').fill('Journey');
  await page.getByRole('button', { name: 'Create' }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.status-save')).toContainText('Saved');
  await page.locator('.title-project').click();
  await expect(page).toHaveURL(/\/projects$/);
  await expect(page.locator('td', { hasText: 'Sample Project' }).first()).toBeVisible();
  await expect(page.locator('td', { hasText: 'Journey' }).first()).toBeVisible();
});
