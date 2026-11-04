import { expect, test } from '@playwright/test';

/** UI-SHL-10: every editor layer carries a breadcrumb back to the workspace.
 *  Also covers the `/component-editor` route (UI-SHL-01 route matrix). */
test('editor pages expose breadcrumbs back to the workspace', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  for (const [route, tail] of [
    ['/series-manager', 'Series Manager'],
    ['/library-manager', 'Library Manager'],
  ] as const) {
    await page.goto(route);
    await expect(page.locator('.page-breadcrumb')).toBeVisible();
    await expect(page.locator('.page-breadcrumb')).toContainText(/项目|Projects/);
    await expect(page.locator('.page-breadcrumb')).toContainText(tail);
    await page.locator('.page-breadcrumb .breadcrumb-home').click();
    await expect(page).toHaveURL(/^http:\/\/127\.0\.0\.1:5174\/$/);
    await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  }

  // Component editor via the main toolbar component action.
  await page.locator('button.component-action', { hasText: 'Component' }).click();
  await expect(page.locator('.page-breadcrumb')).toBeVisible();
  await expect(page.locator('.page-breadcrumb')).toContainText('Component Editor');
  await page.locator('.page-breadcrumb .breadcrumb-home').click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
});
