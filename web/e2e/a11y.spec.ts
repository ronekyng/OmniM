import { expect, test } from '@playwright/test';

/** UI-A11Y-04: tree, tabs, toolbar, menu, table, status and viewport semantics. */

async function openSample(page: import('@playwright/test').Page) {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
}

test('tree exposes tree/treeitem semantics and roving keyboard focus', async ({ page }) => {
  await openSample(page);
  const created = await page.request.post('http://127.0.0.1:3100/api/assemblies', {
    data: {
      name: 'A11Y Assembly',
      parts: [
        { name: 'P1', shape: 'box', p1: 0.2, p2: 0.2, p3: 0.2 },
        { name: 'P2', shape: 'box', p1: 0.2, p2: 0.2, p3: 0.2 },
      ],
    },
  });
  expect(created.ok()).toBeTruthy();
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const tree = page.getByRole('tree', { name: 'Object tree' });
  await expect(tree).toHaveAttribute('aria-multiselectable', 'true');
  const assembly = tree.getByRole('treeitem', { name: /A11Y Assembly/ });
  // The <li> center overlaps a child row; click its own label area.
  const assemblyRow = tree.getByRole('treeitem', { name: /A11Y Assembly/ }).locator('> .tree-info');
  await expect(assembly).toHaveAttribute('aria-level', '1');
  await expect(assembly).toHaveAttribute('aria-expanded', 'true');
  await expect(assembly).toHaveAttribute('aria-selected', 'false');

  await assemblyRow.click();
  await expect(assembly).toHaveAttribute('aria-selected', 'true');

  // Left collapses, right expands, down moves into the first child.
  await assembly.focus();
  await page.keyboard.press('ArrowLeft');
  await expect(assembly).toHaveAttribute('aria-expanded', 'false');
  const firstChild = assembly.getByRole('treeitem').first();
  await expect(firstChild).toHaveCount(0);
  await page.keyboard.press('ArrowRight');
  await expect(assembly).toHaveAttribute('aria-expanded', 'true');
  await expect(firstChild).toBeVisible();
  await page.keyboard.press('ArrowDown');
  await expect(firstChild).toBeFocused();
  await page.keyboard.press('ArrowUp');
  await expect(assembly).toBeFocused();

  // Enter selects the focused row.
  await page.keyboard.press('Enter');
  await expect(assembly).toHaveAttribute('aria-selected', 'true');
});

test('F2 renames in place and property rows move with arrow keys', async ({ page }) => {
  await openSample(page);
  const tree = page.getByRole('tree', { name: 'Object tree' });
  const first = tree.getByRole('treeitem').first();
  await first.focus();
  await page.keyboard.press('F2');

  const input = page.getByLabel('Object name');
  await expect(input).toBeFocused();
  await input.fill('Renamed Storey');
  await page.keyboard.press('Enter');
  await expect(tree.getByRole('treeitem', { name: /Renamed Storey/ })).toBeVisible();

  // Property table: down moves to the next row, Esc leaves the cell.
  await tree.getByRole('treeitem', { name: /Renamed Storey/ }).click();
  const table = page.getByRole('table', { name: 'Object properties' });
  await expect(table).toBeVisible();
  await table.locator('input[aria-label="Type"]').focus();
  await page.keyboard.press('ArrowDown');
  await expect(table.locator('input[aria-label="Name"]')).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(table.locator('input[aria-label="Name"]')).not.toBeFocused();
});

test('context menu is keyboard-invokable with menu roles', async ({ page }) => {
  await openSample(page);
  const first = page.getByRole('tree').getByRole('treeitem').first();
  await first.focus();
  await page.keyboard.press('Shift+F10');

  const menu = page.getByRole('menu', { name: 'Object actions' });
  await expect(menu).toBeVisible();
  const items = menu.getByRole('menuitem');
  await expect(items.first()).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(items.nth(1)).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(menu).toBeHidden();
  await expect(first).toBeFocused();
});

test('dock tabs, bottom tabs, toolbar, status, table and viewport are named', async ({ page }) => {
  await openSample(page);

  // Left dock tabs.
  const leftTabs = page.getByRole('tablist', { name: 'Left dock' });
  const objectsTab = leftTabs.getByRole('tab', { name: /Objects/ });
  const gridsTab = leftTabs.getByRole('tab', { name: 'Grids' });
  await expect(objectsTab).toHaveAttribute('aria-selected', 'true');
  await objectsTab.focus();
  await page.keyboard.press('ArrowRight');
  await expect(gridsTab).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByRole('tabpanel', { name: 'Grids' })).toBeVisible();
  await page.keyboard.press('ArrowLeft');
  await expect(objectsTab).toHaveAttribute('aria-selected', 'true');

  // Bottom console tabs.
  await page.locator('.omni-toggles button', { hasText: '>_ Console' }).click();
  const bottomTabs = page.getByRole('tablist', { name: 'Bottom panel' });
  const consoleTab = bottomTabs.getByRole('tab', { name: /Console/ });
  const aiTab = bottomTabs.getByRole('tab', { name: 'AI' });
  await expect(consoleTab).toHaveAttribute('aria-selected', 'true');
  await consoleTab.focus();
  await page.keyboard.press('ArrowRight');
  await expect(aiTab).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByRole('tabpanel', { name: 'AI' })).toBeVisible();

  // Toolbar, viewport, toast region.
  await expect(page.getByRole('toolbar', { name: 'Modeling commands' })).toBeVisible();
  const viewport = page.getByRole('application', { name: '3D viewport' });
  await expect(viewport).toHaveAttribute('aria-describedby', 'viewport-description');
  await expect(page.locator('#viewport-description')).toContainText('objects in the project');

  await page.getByRole('button', { name: 'Save', exact: true }).click();
  const status = page.getByRole('status', { name: 'Notifications' });
  await expect(status).toBeVisible();
  await expect(status).toHaveAttribute('aria-live', 'polite');

  // Property table.
  await page.getByRole('tree').getByRole('treeitem').first().click();
  const table = page.getByRole('table', { name: 'Object properties' });
  await expect(table).toBeVisible();
  await expect(table).toHaveAttribute('aria-colcount', '2');
  await expect(table.getByRole('row')).not.toHaveCount(0);
  await expect(table.getByRole('rowheader').first()).toHaveText('Type');
  await expect(table.getByRole('cell').first()).toBeVisible();
});
