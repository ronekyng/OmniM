import { expect, test } from '@playwright/test';

/**
 * UI-FBK-06 route-level boundary + UI-FBK-01 console/AI three states.
 */
test('workspace route crash recovers; console and AI expose empty/error states', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Route-level crash: only the workspace boundary reacts, nothing blanks.
  const routeFallback = page.locator('.bottom-panel > *, .bottom-panel, .error-boundary').first();
  await page.evaluate(() => sessionStorage.setItem('omnim.crashTest', 'route'));
  await page.reload();
  const fallback = page.locator('.error-boundary, .min-width-notice').first();
  await expect(fallback).toBeVisible();
  // Clear the fixture and reload: the shell recovers.
  await page.evaluate(() => sessionStorage.removeItem('omnim.crashTest'));
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const dock = page.locator('.bottom-panel');
  await dock.locator('.omni-toggles button', { hasText: '>_ Console' }).click();
  const panel = dock.locator('.omni-panel');
  await expect(panel.locator('.console-empty')).toBeVisible();

  await panel.locator('.omni-tab', { hasText: 'AI' }).click();
  await expect(panel.locator('.aichat-empty')).toBeVisible();

  // Server failure surfaces the error state; retry via New works.
  await page.route('**/api/amir/**', (route) => route.fulfill({ status: 500, body: 'amir down' }));
  await panel.locator('textarea.aichat-input').fill('hello');
  await panel.locator('button.aichat-send').click();
  await expect(panel.locator('.aichat-error')).toBeVisible();
  await panel.getByRole('button', { name: 'New' }).click();
  await expect(panel.locator('.aichat-empty')).toBeVisible();
});
