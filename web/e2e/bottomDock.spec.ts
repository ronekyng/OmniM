import { expect, test } from '@playwright/test';

/** UI-SHL-05: bottom dock is collapsed by default, tabbed, ≤40% height. */
test('bottom dock: collapsed default, tabbed, bounded height', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const dock = page.locator('.bottom-panel');
  await expect(dock.locator('.omni-toggles')).toBeVisible();
  await expect(dock.locator('.omni-toggles button')).toHaveCount(2);
  await expect(dock).not.toHaveClass(/active/);

  // Expanded console.
  await dock.locator('.omni-toggles button', { hasText: '>_ Console' }).click();
  const panel = dock.locator('.omni-panel');
  await expect(panel).toBeVisible();
  await expect(panel.locator('.omni-tab', { hasText: 'AI' })).toBeVisible();

  const viewport = page.viewportSize()!;
  const panelH = (await panel.boundingBox())!.height;
  expect(panelH).toBeLessThanOrEqual(viewport.height * 0.4);
  const dockH = (await dock.boundingBox())!.height;
  expect(dockH).toBeLessThanOrEqual(viewport.height * 0.4);

  // Same dock switches to the AI tab.
  await panel.locator('.omni-tab', { hasText: 'AI' }).click();
  await expect(panel.locator('.aichat-input-row')).toBeVisible();
});
