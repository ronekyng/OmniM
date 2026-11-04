import { expect, test } from '@playwright/test';

/** IA_SHELL §4.4 (UI-SHL-09): floating mode is the escape hatch — the window
 * can be dragged and resized, stays clamped ≥32 px from every edge, and its
 * box persists per panel id across reloads. */

async function openWorkspace(page: import('@playwright/test').Page) {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  // Focus an object so the properties panel (the floatable surface) is up.
  const tree = page.getByRole('tree', { name: 'Object tree' });
  await tree.getByRole('treeitem').first().click();
  await expect(page.locator('.properties-panel').first()).toBeVisible({ timeout: 10_000 });
}

test('floating panel: drag, clamp, resize, persist across reload', async ({ page }) => {
  await openWorkspace(page);

  // Toggle float via the properties panel's own mode toggle (title carries
  // the accessible name).
  const toggle = page.locator('.properties-panel .mode-toggle').first();
  await toggle.click();
  const panel = page.locator('.floating-panel');
  await expect(panel).toBeVisible();
  await expect(page.locator('[data-testid="floating-drag-handle"]')).toBeVisible();

  // Drag: move by the handle toward the left top; box follows the pointer.
  const before = await panel.boundingBox();
  const handle = page.locator('[data-testid="floating-drag-handle"]');
  await page.mouse.move(before.x + before.width / 2, before.y + 5);
  await page.mouse.down();
  await page.mouse.move(200, 100, { steps: 8 });
  await page.mouse.up();
  const afterDrag = await panel.boundingBox();
  expect(afterDrag.x).toBeLessThan(before.x - 50);
  expect(Math.abs(afterDrag.y - 100)).toBeLessThanOrEqual(40);

  // Clamp: dragging far off-viewport keeps ≥32 px inside.
  await page.mouse.move(afterDrag.x + afterDrag.width / 2, afterDrag.y + 5);
  await page.mouse.down();
  await page.mouse.move(3000, 3000, { steps: 4 });
  await page.mouse.up();
  const box = await panel.boundingBox();
  const vw = await page.evaluate(() => window.innerWidth);
  const vh = await page.evaluate(() => window.innerHeight);
  expect(box.x).toBeGreaterThanOrEqual(32);
  expect(box.y).toBeGreaterThanOrEqual(32);
  expect(box.x + box.width).toBeLessThanOrEqual(vw - 32);
  expect(box.y + box.height).toBeLessThanOrEqual(vh - 32);

  // Resize from the corner (pointer lands exactly on the handle centre).
  const rezBefore = await panel.boundingBox();
  const corner = await page.locator('[data-testid="floating-resize-handle"]').boundingBox();
  await page.mouse.move(corner.x + corner.width / 2, corner.y + corner.height / 2);
  await page.mouse.down();
  await page.mouse.move(rezBefore.x + 240, rezBefore.y + 260, { steps: 6 });
  await page.mouse.up();
  const rezAfter = await panel.boundingBox();
  expect(rezAfter.width).toBeLessThan(rezBefore.width);
  expect(rezAfter.height).toBeLessThanOrEqual(rezBefore.height);

  // Persistence: box AND mode survive a reload (panel already floats).
  const persisted = await panel.boundingBox();
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  // Restore a selection so the properties panel (floating surface) registers.
  await page.getByRole('tree', { name: 'Object tree' }).getByRole('treeitem').first().click();
  const panel2 = page.locator('.floating-panel');
  await expect(panel2).toBeVisible();
  const after = await panel2.boundingBox();
  expect(Math.abs(after.x - persisted.x)).toBeLessThanOrEqual(24);
  expect(Math.abs(after.y - persisted.y)).toBeLessThanOrEqual(24);
});

test('docking the panel again clears the floating window', async ({ page }) => {
  await openWorkspace(page);
  await page.locator('.properties-panel .mode-toggle').first().click();
  await expect(page.locator('.floating-panel')).toBeVisible();
  await page.locator('.properties-panel .mode-toggle').first().click();
  await expect(page.locator('.floating-panel')).toHaveCount(0);
});
