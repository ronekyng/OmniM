import { expect, test } from '@playwright/test';

/** UI-INT-06 dialog focus contract + UI-INT-07 destructive impact. */

async function openSample(page: import('@playwright/test').Page) {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
}

test('confirm dialog traps focus, inerts the background and restores focus', async ({ page }) => {
  await openSample(page);
  await page.locator('.tree-node').first().click();

  const deleteButton = page.getByRole('button', { name: 'Delete', exact: true });
  await deleteButton.click();

  const dialog = page.getByRole('alertdialog');
  await expect(dialog).toBeVisible();
  // Safe default for a destructive prompt.
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
  // The workspace behind the portal is inert while the dialog is open.
  await expect(page.locator('#root')).toHaveAttribute('inert', '');
  await expect(dialog).toContainText('Undo restores the change.');

  // Tab cycles inside the dialog.
  await page.keyboard.press('Tab');
  await expect(dialog.getByRole('button', { name: 'Delete' })).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
  await page.keyboard.press('Shift+Tab');
  await expect(dialog.getByRole('button', { name: 'Delete' })).toBeFocused();

  // Esc closes, releases inert and returns focus to the trigger.
  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  await expect(page.locator('#root')).not.toHaveAttribute('inert', '');
  await expect(deleteButton).toBeFocused();
});

test('deleting a container states child counts and both delete paths confirm', async ({ page }) => {
  await openSample(page);
  const created = await page.request.post('http://127.0.0.1:3100/api/assemblies', {
    data: {
      name: 'Impact Assembly',
      parts: [
        { name: 'P1', shape: 'box', p1: 0.2, p2: 0.2, p3: 0.2 },
        { name: 'P2', shape: 'box', p1: 0.2, p2: 0.2, p3: 0.2 },
      ],
    },
  });
  expect(created.ok()).toBeTruthy();
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const assembly = page.locator('.tree-node', { hasText: 'Impact Assembly' }).first();
  await expect(assembly).toBeVisible({ timeout: 20_000 });
  const assemblyRow = assembly.locator('> .tree-info');
  await assemblyRow.click();
  await page.getByRole('button', { name: 'Delete', exact: true }).click();

  const dialog = page.getByRole('alertdialog');
  await expect(dialog).toBeVisible();
  await expect(dialog).toContainText('removes 1 object');
  await expect(dialog).toContainText('2 descendant objects');
  await expect(dialog).toContainText('Part ×2');
  await expect(dialog).toContainText('Undo restores the change.');

  // Cancel keeps the assembly.
  await dialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(assembly).toBeVisible();

  // The tree context-menu path uses the same guard (previously it deleted directly).
  await assemblyRow.click({ button: 'right' });
  await page.locator('.tree-context-menu button', { hasText: 'Delete' }).click();
  await expect(page.getByRole('alertdialog')).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(assembly).toBeVisible();

  // Confirm for real: the subtree is gone and the toast names the object.
  await page.getByRole('button', { name: 'Delete', exact: true }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
  await expect(page.locator('.toast-item', { hasText: 'Object deleted' })).toBeVisible();
  await expect(page.locator('.tree-node', { hasText: 'Impact Assembly' })).toHaveCount(0);
});

test('save-as input dialog shares the same focus contract', async ({ page }) => {
  await openSample(page);
  const trigger = page.getByRole('button', { name: 'Save as' });
  await trigger.click();

  const dialog = page.getByRole('dialog');
  await expect(dialog).toBeVisible();
  await expect(dialog.getByLabel('Project file path')).toBeFocused();
  await expect(page.locator('#root')).toHaveAttribute('inert', '');

  await page.keyboard.press('Tab');
  await expect(dialog.getByRole('button', { name: 'Cancel' })).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(dialog.getByRole('button', { name: 'Save' })).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(dialog.getByLabel('Project file path')).toBeFocused();

  await page.keyboard.press('Escape');
  await expect(dialog).toBeHidden();
  await expect(page.locator('#root')).not.toHaveAttribute('inert', '');
  await expect(trigger).toBeFocused();
});
