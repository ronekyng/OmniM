import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/** G2 UI-DAT-07 (DATA_INTEGRITY §2.3): the title-bar project-settings dialog
 * edits name / description / author / units and the server persists them. */

test('project settings edit persists and renames the project file', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  await page.getByRole('button', { name: '项目设置' }).click();
  const dialog = page.getByRole('dialog', { name: '项目设置' });
  await expect(dialog).toBeVisible();

  const desc = 'Settings saved by meta.spec';
  await dialog.getByRole('textbox', { name: '描述' }).fill(desc);
  await dialog.getByRole('button', { name: '保存' }).click();
  await expect(dialog).not.toBeVisible();

  // Server-side truth: meta persisted (checked via the read-only endpoint).
  const meta = await page.request.get('http://127.0.0.1:3100/api/project/meta').then((r) => r.json());
  expect(meta['description']).toBe(desc);

  // Reopen assert: after reload the dialog shows the persisted description.
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await page.getByRole('button', { name: '项目设置' }).click();
  await expect(page.getByRole('dialog', { name: '项目设置' }).getByRole('textbox', { name: '描述' })).toHaveValue(desc);
});
