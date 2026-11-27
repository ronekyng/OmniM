import { expect, test } from '@playwright/test';

/** UI-INT-01: tree click/replace, Ctrl/Cmd toggle, Shift range, Esc clears,
 *  Ctrl/Cmd+A selects all visible objects. Canvas box-select/empty-click stay
 *  pending until a calibrated camera test. */
test('tree gestures: click replaces, ctrl toggles, Esc clears', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const first = page.locator('.tree-node').first();
  const second = page.locator('.tree-node').nth(1);

  await first.click();
  await expect(page.locator('.status-selection')).not.toHaveText('No selection');

  await second.click({ modifiers: ['ControlOrMeta'] });
  await expect(page.locator('.status-selection')).toContainText('2 selected');

  await second.click({ modifiers: ['ControlOrMeta'] });
  await expect(page.locator('.status-selection')).not.toContainText('2 selected');

  await page.keyboard.press('Escape');
  await expect(page.locator('.status-selection')).toHaveText('No selection');
});

test('tree gestures: shift range select and Ctrl/Cmd+A select all', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Seed one wall into the open project, then reload so the scene/tree pick it up.
  const seeded = await page.request.post('http://127.0.0.1:3100/api/objects', {
    data: { object_type: 'wall', name: 'GestureWall' },
  });
  expect(seeded.status(), String(await seeded.text())).toBe(200);
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const rows = page.locator('.tree-node');
  const count = await rows.count();
  expect(count).toBeGreaterThanOrEqual(2);

  await rows.nth(0).click();
  await rows.nth(count - 1).click({ modifiers: ['Shift'] });
  await expect(page.locator('.status-selection')).toContainText(`${count} selected`);

  await page.keyboard.press('Escape');
  await expect(page.locator('.status-selection')).toHaveText('No selection');

  await page.keyboard.press('ControlOrMeta+A');
  await expect(page.locator('.status-selection')).toContainText(/\d+ selected/);
});

test('viewport gestures: click select, empty click clears, box select replace/add', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Seed two walls with real geometry so viewport picking has meshes and a
  // box select can prove a multi-object result.
  for (const name of ['GestureWall3D', 'GestureWall3D-2']) {
    const wall = await page.request.post('http://127.0.0.1:3100/api/genrule', {
      data: { genrule: 'wall', name, length: 5, height: 3, core_thickness: 0.2, finish_thickness: 0.02 },
    });
    expect(wall.status(), String(await wall.text())).toBe(200);
  }
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const canvas = page.locator('.viewer-content canvas').first();
  await expect(canvas).toBeVisible();
  await page.keyboard.press('f'); // fit view centers the wall
  const box = (await canvas.boundingBox())!;
  const cx = box.x + box.width / 2;
  const cy = box.y + box.height / 2;

  // Left click on an object selects it (single selection shows its name).
  await page.mouse.click(cx, cy);
  await expect(page.locator('.status-selection')).not.toHaveText('No selection');

  // Left click on empty space clears (top-left corner is outside the wall).
  await page.mouse.click(box.x + 8, box.y + 8);
  await expect(page.locator('.status-selection')).toHaveText('No selection');

  // Ctrl/Cmd+drag = additive box select (shipped nav scheme).
  await page.keyboard.down('Control');
  await page.mouse.move(box.x + 4, box.y + 4);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width - 4, box.y + box.height - 4, { steps: 6 });
  await page.mouse.up();
  await page.keyboard.up('Control');
  await expect(page.locator('.status-selection')).toContainText('2 selected');

  // Contract nav scheme: plain empty-space drag replaces the selection.
  await page.keyboard.press('ControlOrMeta+K');
  const palette = page.getByRole('dialog', { name: 'Command palette' });
  await palette.getByRole('combobox', { name: 'Search commands' }).fill('navigation scheme');
  await page.keyboard.press('Enter');
  await page.keyboard.press('Escape');
  await page.mouse.move(box.x + 4, box.y + 4);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width - 4, box.y + box.height - 4, { steps: 6 });
  await page.mouse.up();
  await expect(page.locator('.status-selection')).toContainText('2 selected');
});
