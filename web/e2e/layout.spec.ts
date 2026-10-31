import { expect, test } from '@playwright/test';

/**
 * G1 dock evidence: UI-SHL-03 (properties dock 260–600 px), UI-SHL-04
 * (left dock 240–400 px), UI-SHL-06 (width persists across reload + reset).
 */
test('dock widths persist across reload and reset', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await page.locator('.tree-node', { hasText: 'Ground Floor' }).first().click();
  await expect(page.locator('.right-panel')).toBeVisible();

  const left = await page.locator('.sidebar').boundingBox();
  expect(left!.width).toBeGreaterThanOrEqual(240);
  expect(left!.width).toBeLessThanOrEqual(400);

  const right = page.locator('.right-panel');
  const initial = (await right.boundingBox())!.width;
  expect(initial).toBeGreaterThanOrEqual(260);
  expect(initial).toBeLessThanOrEqual(600);

  // Drag the splitter left: the right dock grows.
  const handle = page.locator('.panel-resize-handle');
  const box = (await handle.boundingBox())!;
  await page.mouse.move(box.x + 1, box.y + 20);
  await page.mouse.down();
  await page.mouse.move(box.x - 80, box.y + 20, { steps: 6 });
  await page.mouse.up();

  const widened = (await right.boundingBox())!.width;
  expect(widened).toBeGreaterThan(initial + 40);
  expect(widened).toBeLessThanOrEqual(600);

  const stored = await page.evaluate(() => localStorage.getItem('omnim.layout.v1'));
  expect(stored).toContain('"right"');
  expect(stored).toContain('"width"');

  // Reload: the persisted width comes back with the workspace.
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await page.locator('.tree-node', { hasText: 'Ground Floor' }).first().click();
  await expect(page.locator('.right-panel')).toBeVisible();
  const restored = (await page.locator('.right-panel').boundingBox())!.width;
  expect(Math.abs(restored - widened)).toBeLessThanOrEqual(2);

  // Reset layout restores the documented default (340 px).
  await page.getByRole('button', { name: 'Reset layout' }).click();
  await expect
    .poll(async () => (await page.locator('.right-panel').boundingBox())!.width)
    .toBeLessThanOrEqual(342);
  const reset = (await page.locator('.right-panel').boundingBox())!.width;
  expect(Math.abs(reset - 340)).toBeLessThanOrEqual(2);
});
