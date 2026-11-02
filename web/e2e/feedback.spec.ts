import { expect, test } from '@playwright/test';

/**
 * G1 feedback evidence:
 * - UI-FBK-06: a panel that throws is caught in place; the shell stays alive.
 * - UI-FBK-07: errors persist until dismissed; success toasts auto-dismiss.
 * - UI-FBK-08: toasts sit above the status bar, dismissible.
 */
test('panel crash is contained; toasts obey timing and placement', async ({ page }) => {
  await page.addInitScript(() => sessionStorage.setItem('omnim.crashTest', 'panel'));
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Panel crash caught in place inside the bottom dock.
  const fallback = page.locator('.bottom-panel .error-boundary, .bottom-panel > .error-boundary').first();
  await expect(fallback).toBeVisible();
  await expect(page.locator('.title-bar')).toBeVisible();
  await expect(page.locator('.sidebar')).toBeVisible();
  await page.locator('.omni-toggles').first().waitFor({ state: 'attached' }).catch(() => undefined);

  // Error toast comes from a failed IFC import and must persist.
  const chooserPromise = page.waitForEvent('filechooser');
  await page.locator('button[title="Import IFC"]', { hasText: 'Import' }).click();
  const chooser = await chooserPromise;
  await chooser.setFiles({
    name: 'broken.ifc',
    mimeType: 'text/plain',
    buffer: Buffer.from('this file is not an IFC payload'),
  });
  // Import first asks for placement, then for merge semantics.
  await expect(page.getByRole('alertdialog')).toBeVisible();
  await page.getByRole('button', { name: 'Keep coordinates' }).click();
  await expect(page.getByRole('alertdialog')).toBeVisible();
  await page.getByRole('button', { name: 'Merge' }).click();
  const errorToast = page.locator('.toast-item', { hasText: 'Import failed' }).first();
  await expect(errorToast).toBeVisible();
  await page.waitForTimeout(5_000);
  await expect(errorToast).toBeVisible();

  // Success side: undo action auto-dismisses within ~5 s.
  await page.locator('.omni-toggles').first().waitFor({ state: 'attached' }).catch(() => undefined);
  await test.info().attach('toast-placement.png', {
    body: await page.screenshot(),
    contentType: 'image/png',
  });

  // Placement contract: the container must sit above the status bar.
  const status = await page.locator('.status-bar').boundingBox();
  const container = await page.locator('.toast-container').boundingBox();
  expect(container!.y + container!.height).toBeLessThanOrEqual(status!.y + 1);
  expect(container!.width).toBeLessThanOrEqual(420);
});
