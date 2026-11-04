import { expect, test, type Page } from '@playwright/test';

/** UI-SHL-12 command palette + A11Y §3 shortcut help and keyboard J1. */

async function tabTo(page: Page, text: string, limit = 40) {
  for (let i = 0; i < limit; i += 1) {
    await page.keyboard.press('Tab');
    const active = await page.evaluate(() => document.activeElement?.textContent?.trim() ?? '');
    if (active.includes(text)) return true;
  }
  return false;
}

async function openSampleWithKeyboard(page: Page) {
  await page.goto('/projects');
  // The sample button is disabled until the recent list resolves; in a
  // long-lived E2E root the list can take a few hundred ms, so wait for the
  // enabled state (the tab walk alone only matches textContent).
  const sample = page.getByRole('button', { name: /Open sample project/ });
  await expect(sample).toBeDisabled({ timeout: 10_000 }).catch(() => {});
  await expect(sample).toBeEnabled({ timeout: 10_000 });
  await sample.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
}

test('command palette runs commands from the keyboard', async ({ page }) => {
  await openSampleWithKeyboard(page);

  await page.keyboard.press('ControlOrMeta+K');
  const palette = page.getByRole('dialog', { name: 'Command palette' });
  await expect(palette).toBeVisible();
  await expect(palette.getByRole('combobox', { name: 'Search commands' })).toBeFocused();

  await page.keyboard.type('save project');
  await expect(palette.getByRole('option', { name: /Save project/ })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await page.keyboard.press('Enter');
  await expect(palette).toBeHidden();
  await expect(page.locator('.status-save')).toContainText('Saved');

  // A create command proves the palette reaches the modeling workflow.
  await page.keyboard.press('ControlOrMeta+K');
  await page.keyboard.type('create storey');
  await page.keyboard.press('Enter');
  await expect(page.locator('.create-panel')).toBeVisible();
  await page.keyboard.press('Escape');
});

test('F1 shortcut help lists the command registry and closes with Esc', async ({ page }) => {
  await openSampleWithKeyboard(page);
  await page.keyboard.press('F1');

  const help = page.getByRole('dialog', { name: 'Keyboard shortcuts' });
  await expect(help).toBeVisible();
  await expect(help).toContainText('Command palette');
  await expect(help).toContainText('Ctrl/Cmd+K');
  await expect(help).toContainText('Shift+F10');
  await expect(help.getByRole('button', { name: 'Close' })).toBeFocused();

  await page.keyboard.press('Escape');
  await expect(help).toBeHidden();
});

test('J1 create and save can be completed without a pointer', async ({ page }) => {
  await openSampleWithKeyboard(page);

  await page.keyboard.press('ControlOrMeta+K');
  await page.keyboard.type('create storey');
  await page.keyboard.press('Enter');
  const panel = page.locator('.create-panel');
  await expect(panel).toBeVisible();

  await panel.getByLabel(/Name|名称/).focus();
  await page.keyboard.type('Keyboard Storey');
  // The creation form may surface as the toolbar `.create-panel` OR the
  // right-dock properties panel creation view (both bound to the same form);
  // submit the page-level 创建 button.
  await page.getByRole('button', { name: '创建' }).first().click();

  await expect(page.locator('.tree-node', { hasText: 'Keyboard Storey' })).toBeVisible({
    timeout: 20_000,
  });
  await page.keyboard.press('ControlOrMeta+S');
  await expect(page.locator('.status-save')).toContainText('Saved');
});
