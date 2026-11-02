import { expect, test } from '@playwright/test';

/** UI-INT-01 (tree subset): click/replace, Ctrl/Cmd toggle, Esc clears.
 *  Canvas box-select/empty-click stay pending until a calibrated camera test. */
test('tree gestures: click replaces, ctrl toggles, Esc clears', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const first = page.locator('.tree-node').first();
  const second = page.locator('.tree-node').nth(1);

  await first.click();
  await expect(page.locator('.status-selection')).not.toHaveText('No selection');

  await second.click({ modifiers: ['ControlOrMeta'] });
  await expect(page.locator('.status-selection')).toContainText('2 selected');

  await second.click({ modifiers: ['ControlOrMeta'] });
  await expect(page.locator('.status-selection')).not.toContainText('2 selected');

  await page.keyboard.press('Escape');
  await expect(page.locator('.status-selection')).toHaveText('No selection');
});
