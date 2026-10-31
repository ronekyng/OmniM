import { expect, test } from '@playwright/test';

/**
 * G1 shell evidence: seven fixed regions (UI-SHL-02), breadcrumb (UI-SHL-10),
 * status-bar readings (UI-SHL-08), input shortcut guard (UI-INT-04) and
 * dark/light screenshots for UI-DS-02.
 */
test('workspace exposes the seven regions and guards shortcuts in inputs', async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem('omnim.developer', '1'));
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Seven regions: left, title, toolbar, viewport, right, bottom, status.
  await expect(page.locator('.sidebar')).toBeVisible();
  await expect(page.locator('.title-bar')).toBeVisible();
  await expect(page.locator('.main-toolbar')).toBeVisible();
  await expect(page.locator('.viewport-area')).toBeVisible();
  await expect(page.locator('.bottom-panel')).toBeVisible();
  await expect(page.locator('.status-bar')).toBeVisible();

  // Selecting a storey opens the right dock.
  await page.locator('.tree-node', { hasText: 'Ground Floor' }).first().click();
  await expect(page.locator('.right-panel')).toBeVisible();

  // Breadcrumb + status readings.
  await expect(page.locator('.breadcrumb')).toContainText('Workspace');
  await expect(page.locator('.breadcrumb')).toContainText('Modeling');
  await expect(page.locator('.status-coords')).toBeVisible();
  await expect(page.locator('.status-selection')).toContainText('Ground Floor');
  await expect(page.locator('.status-units')).toHaveText('m');
  await expect(page.locator('.status-save')).toBeVisible();
  await expect(page.locator('.status-fps')).toBeVisible();

  // Typing/deleting inside an input must not trigger the global Delete shortcut.
  const nameInput = page.locator('input[name="name"]');
  await nameInput.click();
  await page.keyboard.press('Delete');
  await expect(page.locator('.tree-node', { hasText: 'Ground Floor' })).toHaveCount(1);

  // Theme screenshots (attached to the report as UI-DS-02 evidence).
  const toggle = page.getByRole('button', { name: /Switch to (light|dark) theme/ });
  await toggle.click();
  await test.info().attach('theme-a.png', {
    body: await page.screenshot(),
    contentType: 'image/png',
  });
  await toggle.click();
  await test.info().attach('theme-b.png', {
    body: await page.screenshot(),
    contentType: 'image/png',
  });
});
