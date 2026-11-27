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

/** INTERACTION §2 (right-button): the canvas suppresses the browser context
 *  menu unconditionally, so right-drag completes as a real viewport gesture —
 *  orbit under the current scheme, pan under contract. */
test('right-drag is a viewport gesture and the canvas context menu is suppressed', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const canvas = page.locator('.three-viewer canvas').first();
  await expect(canvas).toBeVisible();

  const assertMenuSuppressed = async () => {
    const prevented = await canvas.evaluate((el) => {
      const ev = new MouseEvent('contextmenu', { bubbles: true, cancelable: true });
      el.dispatchEvent(ev);
      return ev.defaultPrevented;
    });
    expect(prevented).toBe(true);
  };
  await assertMenuSuppressed();

  const dragAndCheck = async () => {
    const before = await canvas.screenshot();
    const box = await canvas.boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down({ button: 'right' });
    await page.mouse.move(box.x + box.width / 2 + 80, box.y + box.height / 2 + 40, { steps: 8 });
    await page.mouse.up({ button: 'right' });
    await assertMenuSuppressed();
    await page.waitForTimeout(500); // on-demand render settles after the drag
    const after = await canvas.screenshot();
    expect(after.equals(before)).toBe(false);
  };

  // current scheme: right-drag = orbit.
  await dragAndCheck();

  // contract scheme: right-drag = pan — still a gesture, still no menu.
  await page.keyboard.press('ControlOrMeta+K');
  await page.keyboard.type('navigation');
  await page.keyboard.press('Enter');
  expect(await page.evaluate(() => localStorage.getItem('omnim.nav'))).toBe('contract');
  await page.waitForTimeout(300);
  await dragAndCheck();

  // Restore the shipped scheme for other specs.
  await page.keyboard.press('ControlOrMeta+K');
  await page.keyboard.type('navigation');
  await page.keyboard.press('Enter');
  expect(await page.evaluate(() => localStorage.getItem('omnim.nav'))).toBe('current');
});
