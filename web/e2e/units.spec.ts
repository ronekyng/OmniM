import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/** INTERACTION §9.2 (UI-INT-10): every numeric input in the creation panel
 * carries its unit suffix — m for world-space lengths, mm for section
 * thicknesses; catalog params echo `{value} {unit}` (mm). */

test('creation forms show unit suffixes on numeric inputs', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Storey: the elevation input carries the m suffix.
  await page.keyboard.press('ControlOrMeta+K');
  await page.keyboard.type('create storey');
  await page.keyboard.press('Enter');
  const panel = page.locator('.create-panel');
  await expect(panel).toBeVisible();
  const elevation = panel.getByLabel('Elevation');
  await expect(elevation.locator('xpath=following-sibling::span[@class="field-unit"][1]')).toHaveText('m');

  // Door fields need a host wall — create one through the API first.
  await page.request.post('http://127.0.0.1:3100/api/objects', {
    data: { object_type: 'wall', name: 'UnitWall' },
  });
  await page.waitForTimeout(500);

  // Escape out and open the door form: opening width/height/offsets are m.
  await page.keyboard.press('Escape');
  await page.getByRole('button', { name: '门', exact: true }).click();
  await expect(panel).toBeVisible();
  for (const label of ['Opening width', 'Opening height', 'Horizontal offset', 'Vertical offset']) {
    const field = panel.getByLabel(label);
    await expect(field.locator('xpath=following-sibling::span[@class="field-unit"][1]')).toHaveText('m');
  }

  // Catalog parameters echo `{value} {unit}` (mm for modelled params);
  // AssemblyPanel carries its own mm/m trap notice (F4 audit) — the
  // ComponentParamPanel value cell format is locked by that component's tests.
  await page.keyboard.press('Escape');
});
