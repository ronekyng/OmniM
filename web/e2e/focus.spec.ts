import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/** UI-A11Y-03: visible focus, F6 region cycling, focus restoration, route focus. */

async function openSample(page: import('@playwright/test').Page) {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
}

test('keyboard focus is visible and F6 cycles shell regions', async ({ page }) => {
  await openSample(page);

  // F6 lands on the next region's first focusable control, with a 2 px ring.
  await page.locator('.main-toolbar button').first().focus();
  await page.keyboard.press('F6');
  const tab = page.getByRole('tab', { name: /对象/ });
  await expect(tab).toBeFocused();
  const ring = await tab.evaluate((el) => {
    const style = getComputedStyle(el);
    return { width: style.outlineWidth, offset: style.outlineOffset, style: style.outlineStyle };
  });
  expect(ring).toEqual({ width: '2px', offset: '2px', style: 'solid' });

  await page.keyboard.press('Shift+F6');
  expect(
    await page.evaluate(() => !!document.activeElement?.closest('.main-toolbar')),
  ).toBe(true);
});

test('deleting the focused tree row moves focus to a neighbour', async ({ page }) => {
  await openSample(page);
  const created = await page.request.post('http://127.0.0.1:3100/api/assemblies', {
    data: {
      name: 'Focus Assembly',
      parts: [
        { name: 'P1', shape: 'box', p1: 0.2, p2: 0.2, p3: 0.2 },
        { name: 'P2', shape: 'box', p1: 0.2, p2: 0.2, p3: 0.2 },
      ],
    },
  });
  expect(created.ok()).toBeTruthy();
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const assembly = page.getByRole('tree', { name: 'Object tree' }).getByRole('treeitem', { name: /Focus Assembly/ });
  await assembly.locator('> .tree-info').click();
  await assembly.focus();
  await page.keyboard.press('Delete');

  const dialog = page.getByRole('alertdialog');
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: 'Delete' }).click();
  await expect(assembly).toHaveCount(0);

  await expect
    .poll(async () =>
      page.evaluate(() => (document.activeElement as HTMLElement | null)?.getAttribute('role')),
    )
    .toBe('treeitem');
});

test('route changes and error fallbacks take focus', async ({ page }) => {
  // Route change: workspace heading → projects h1.
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await expect(page.locator('[data-page-heading]')).toBeFocused();

  await page.locator('.title-project').click();
  await expect(page.getByRole('heading', { name: 'OmniM Projects' })).toBeFocused();

  // Error fallback: heading receives focus and is announced.
  await page.addInitScript(() => sessionStorage.setItem('omnim.crashTest', 'panel'));
  await page.reload();
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  const heading = page.locator('.bottom-panel .error-boundary h3').first();
  await expect(heading).toBeVisible();
  await expect(heading).toBeFocused();
  await expect(page.locator('.bottom-panel .error-boundary').first()).toHaveAttribute('role', 'alert');
});
