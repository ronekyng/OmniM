import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

/** UI-A11Y-01: axe audit of the critical paths (WCAG 2.2 AA tags). */

async function audit(page: Page, label: string) {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
    .analyze();
  const serious = results.violations.filter(
    (v) => v.impact === 'serious' || v.impact === 'critical',
  );
  console.log(
    `AXE ${label}`,
    JSON.stringify(
      results.violations.map((v) => ({
        id: v.id,
        impact: v.impact,
        nodes: v.nodes.slice(0, 3).map((n) => ({ target: n.target.join(' '), html: n.html.slice(0, 140), why: n.failureSummary })),
      })),
    ),
  );
  expect(serious, `${label}: ${JSON.stringify(serious.map((v) => v.id))}`).toEqual([]);
}

async function openSample(page: Page) {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
}

test('critical paths have no serious axe violations', async ({ page }) => {
  test.setTimeout(240_000);
  await page.addInitScript(() => localStorage.setItem('omnim.theme', 'dark'));
  await page.goto('/projects');
  await expect(page.getByRole('heading', { name: 'OmniM Projects' })).toBeVisible();
  await audit(page, 'projects');

  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await page.locator('.tree-node').first().click();
  await page.waitForTimeout(300);
  await audit(page, 'workspace-dark');

  await page.getByRole('button', { name: 'Switch to light theme' }).click();
  await page.waitForTimeout(300);
  await audit(page, 'workspace-light');

  await page.getByRole('button', { name: 'Save as' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await audit(page, 'save-as-dialog');
  await page.keyboard.press('Escape');

  await page.locator('.omni-toggles button', { hasText: '>_ Console' }).click();
  await page.waitForTimeout(200);
  await audit(page, 'console-panel');

  await page.getByRole('button', { name: 'Delete', exact: true }).click();
  await expect(page.getByRole('alertdialog')).toBeVisible();
  await audit(page, 'delete-confirm');
  await page.keyboard.press('Escape');

  // Command palette and shortcut help.
  await page.keyboard.press('ControlOrMeta+K');
  await expect(page.getByRole('dialog', { name: 'Command palette' })).toBeVisible();
  await audit(page, 'command-palette');
  await page.keyboard.press('Escape');
  await page.keyboard.press('F1');
  await expect(page.getByRole('dialog', { name: 'Keyboard shortcuts' })).toBeVisible();
  await audit(page, 'shortcut-help');
  await page.keyboard.press('Escape');

  // Create panel (J1 create step): the column command opens the form directly.
  await page.getByRole('button', { name: '门', exact: true }).click();
  await expect(page.locator('.create-panel')).toBeVisible();
  await audit(page, 'create-panel');
  await page.locator('.create-panel .panel-close').click();
  await expect(page.locator('.create-panel')).toBeHidden();

  // Error recovery: a rejected save surfaces a persistent toast.
  await page.route('**/api/project/save', (route) => route.fulfill({ status: 500, body: 'boom' }));
  await page.locator('.title-actions').getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.locator('.toast-item.error')).toBeVisible();
  await audit(page, 'save-error-toast');
  await page.unroute('**/api/project/save');

  // Error fallback: the panel boundary renders and is announced.
  await page.evaluate(() => sessionStorage.setItem('omnim.crashTest', 'panel'));
  await page.reload();
  await expect(page.locator('.bottom-panel .error-boundary')).toBeVisible({ timeout: 20_000 });
  await audit(page, 'error-fallback');
});
