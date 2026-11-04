import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/** UI-FBK-03 (FEEDBACK_ERRORS §4): an import visible for >1 s shows an
 * indeterminate progress indicator with elapsed seconds and stays cancellable. */

test('slow import shows progress and can be cancelled', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // The import request is intercepted and delayed: deterministic >1 s window.
  const objectsBefore = (await page.request.get('http://127.0.0.1:3100/api/objects').then((r) => r.json())) as unknown[];
  await page.route(/\/api\/import\/ifc/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 4000));
    // Reply from the real backend: the import actually completes server-side.
    void route.continue();
  });

  const content = [
    'ISO-10303-21;',
    ...Array.from({ length: 40 }, (_, i) => `#${i + 1}=IFCBUILDINGELEMENTPROXY('0',$,'Px-${i}',$,$,$,$,$,.ELEMENT.);`),
    'END-ISO-10303-21;',
  ].join('\r\n');
  await page.setInputFiles('input[type=file][accept=".ifc"]', {
    name: 'slow.ifc',
    mimeType: 'text/plain',
    buffer: Buffer.from(content, 'utf-8'),
  });

  // The import flow asks two confirm dialogs first; answer them so the
  // fetch starts (progress gate runs on the fetch window).
  await page.getByRole('button', { name: 'Move to origin' }).click();
  await page.getByRole('button', { name: 'Replace', exact: true }).click();

  // Progress appears (>1 s) with a live elapsed counter and a cancel button.
  const overlay = page.getByRole('alertdialog', { name: 'Importing model' });
  await expect(overlay).toBeVisible({ timeout: 6_000 });
  await expect(overlay.getByRole('progressbar')).toBeVisible();
  await expect(overlay.getByText(/秒|second/)).toBeVisible();

  // Cancel: overlay closes and the UI reports cancellation.
  await overlay.getByRole('button', { name: '取消导入' }).click();
  await expect(overlay).not.toBeVisible({ timeout: 5_000 });
  await expect(page.getByText('取消导入')).toBeVisible({ timeout: 5_000 });

  void objectsBefore;
});
