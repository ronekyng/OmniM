import { expect, test } from '@playwright/test';

// 检测顺序 §3：浏览器语言交集在默认 locale 之前生效。锁定该 spec 的浏览器
// 语言为 zh-CN，以便断言“默认 zh-CN”的契约行为。
test.use({ locale: 'zh-CN' });

/** I18N_L10N §3: title-bar language switcher — instant switch without reload,
 * `omnim.locale` persistence, `<html lang>` sync, dictionary fallback. */

test('language switcher switches instantly, persists and syncs <html lang>', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // default: zh-CN (kernel initialises <html lang> from detection)
  await expect(page.locator('html')).toHaveAttribute('lang', 'zh-CN');
  const switcher = page.locator('select.lang-switch');
  await expect(switcher).toBeVisible({ timeout: 5_000 });

  // switch to English — instant, no reload (workspace action labels语言随切)
  await switcher.selectOption('en-US');
  await expect(page.locator('html')).toHaveAttribute('lang', 'en-US');
  await expect(page.getByRole('button', { name: 'Save as', exact: true })).toBeVisible();

  // persistence survives a real reload
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
  await expect(page.locator('html')).toHaveAttribute('lang', 'en-US');

  // restore zh for the rest of the suite and confirm the zh surface returns
  await switcher.selectOption('zh-CN');
  await expect(page.locator('html')).toHaveAttribute('lang', 'zh-CN');
});

test('workspace title bar strings follow the selected language', async ({ page }) => {
  await page.goto('/projects');
  const switcher = page.locator('select.lang-switch');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // default zh: breadcrumb + action labels
  const bar = page.locator('.title-bar');
  await expect(bar.locator('.breadcrumb span').first()).toHaveText('工作区');
  await expect(bar.getByRole('button', { name: '保存', exact: true })).toBeVisible();
  await expect(bar.getByRole('button', { name: '另存为' })).toBeVisible();

  // switch to en — instant (no reload)
  await switcher.selectOption('en-US');
  await expect(bar.locator('.breadcrumb span').first()).toHaveText('Workspace');
  await expect(bar.getByRole('button', { name: 'Save', exact: true })).toBeVisible();
  await expect(bar.getByRole('button', { name: 'Save as' })).toBeVisible();
  await expect(page.locator('html')).toHaveAttribute('lang', 'en-US');
});
