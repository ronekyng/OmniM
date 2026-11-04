import { expect, test } from '@playwright/test';

/** INTERACTION §10 (UI-INT-12): the viewport navigation scheme is a real,
 * configurable setting — discoverable in the command palette, persisted to
 * `omnim.nav`, and applied to the viewport controls. */

test('nav scheme cycles through the palette, persists and applies', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const before = await page.evaluate(() => localStorage.getItem('omnim.nav'));

  await page.keyboard.press('ControlOrMeta+K');
  await page.keyboard.type('navigation');
  const palette = page.getByRole('dialog', { name: 'Command palette' });
  await expect(palette.getByRole('combobox', { name: 'Search commands' })).toBeFocused();
  await expect(palette.getByText('Viewport navigation scheme')).toBeVisible();
  await page.keyboard.press('Enter');
  const afterCycle1 = await page.evaluate(() => localStorage.getItem('omnim.nav'));
  expect(afterCycle1).toBe(before === 'contract' ? 'current' : 'contract');

  // Reload: the setting persists (store rehydrates from omnim.nav).
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await expect(page.locator('body')).toContainText(/工作区|Workspace/);
  expect(await page.evaluate(() => localStorage.getItem('omnim.nav'))).toBe(afterCycle1);

  // Cycle back to restore the shipped scheme for other specs.
  await page.keyboard.press('ControlOrMeta+K');
  await page.keyboard.type('navigation');
  await page.keyboard.press('Enter');
  expect(await page.evaluate(() => localStorage.getItem('omnim.nav'))).toBe('current');
});
