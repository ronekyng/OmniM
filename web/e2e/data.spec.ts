import { expect, test } from '@playwright/test';

/**
 * G1 data-integrity evidence: dirty state (UI-DAT-03), explicit save
 * (UI-DAT-02) and the in-app leave guard (UI-DAT-06).
 */
test('dirty state, explicit save and leave guard', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // A mutation marks the session dirty; server auto-persist copy is explicit.
  await page.getByRole('tab', { name: 'Grids' }).click();
  await page.getByLabel('Name').fill('Save-Test-Level');
  await page.getByRole('button', { name: 'Add storey' }).click();
  await expect(page.locator('.grid-line-row', { hasText: 'Save-Test-Level' })).toBeVisible();
  await expect(page.locator('.status-save')).toContainText('Auto-saved server-side');

  // Leaving while dirty prompts; choosing Stay keeps the workspace.
  await page.locator('.title-project').click();
  await expect(page.getByRole('alertdialog')).toBeVisible();
  await expect(page.getByRole('alertdialog')).toContainText('Leave without saving?');
  await page.getByRole('button', { name: 'Stay' }).click();
  await expect(page.locator('.title-bar')).toBeVisible();
  await expect(page).not.toHaveURL(/\/projects$/);

  // Explicit save clears the dirty state.
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.status-save')).toContainText('Saved');

  // Save-as retargets the session and stays clean.
  await page.getByLabel('Name').fill('Save-As-Level');
  await page.getByRole('button', { name: 'Add storey' }).click();
  await expect(page.locator('.status-save')).toContainText('Auto-saved server-side');
  await page.getByRole('button', { name: 'Save as' }).click();
  await page.locator('.confirm-dialog input').fill('save-as-target.omnim');
  await page.locator('.confirm-dialog button[type="submit"]').click();
  await expect(page.locator('.status-save')).toContainText('Saved');

  // Data-integrity §3.4: a failed save escalates and is retryable.
  await page.route('**/api/project/save', (route) => route.fulfill({ status: 500, body: 'boom' }));
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.status-save')).toContainText('Save failed');
  await expect(page.locator('.toast-item', { hasText: 'Save failed' }).first()).toBeVisible();
  await page.unroute('**/api/project/save');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.status-save')).toContainText('Saved');

  // While dirty, the "Leave" choice navigates to the project home.
  await page.getByLabel('Name').fill('Leave-Level');
  await page.getByRole('button', { name: 'Add storey' }).click();
  await expect(page.locator('.status-save')).toContainText('Auto-saved server-side');

  // While dirty, the "Leave" choice navigates to the project home.
  // NOTE: the refresh/beforeunload branch stays pending — headless Chromium
  // does not surface `beforeunload` on programmatic reload without a prior
  // user gesture, so it cannot be covered here reliably.
  await page.locator('.title-project').click();
  await expect(page.getByRole('alertdialog')).toBeVisible();
  await page.getByRole('button', { name: 'Leave' }).click();
  await expect(page).toHaveURL(/\/projects$/);
  await expect(page.getByRole('heading', { name: 'OmniM Projects' })).toBeVisible();
});
