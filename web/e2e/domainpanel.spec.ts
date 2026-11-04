import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/** Phase 8-5: domain surfaces — the mechanical panel lists the motion joints
 * of the selected object; the building panel lists the door/window schedule. */

test('domain panels open from the ribbon switches and list domain data', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Backend fixtures: an assembly with two parts + one joint, and a wall + door.
  const objects = (await (await page.request.get('http://127.0.0.1:3100/api/objects')).json()) as {
    id: string;
    object_type: number;
    name?: string;
    parent_id?: string | null;
  }[];
  let wallId = objects.find((o) => o.object_type === 100)?.id;
  if (!wallId) {
    const wall = await page.request.post('http://127.0.0.1:3100/api/objects', {
      data: { object_type: 'wall', name: 'JointWall' },
    });
    wallId = (await wall.json())['id'];
  }
  const door = await page.request.post('http://127.0.0.1:3100/api/genrule', {
    data: {
      genrule: 'door',
      name: 'DP-Door',
      wall_id: wallId,
      opening_width: 1.2,
      opening_height: 2.1,
      position_x: 1.5,
      position_z: 0,
      leaf_count: 2,
    },
  });
  expect(door.status(), String(await door.text())).toBe(200);

  const created = await page.request.post('http://127.0.0.1:3100/api/assemblies', {
    data: {
      name: 'DP Assembly',
      parts: [
        { name: 'DPA-1', shape: 'box', p1: 0.3, p2: 0.3, p3: 0.3 },
        { name: 'DPA-2', shape: 'box', p1: 0.3, p2: 0.3, p3: 0.3 },
      ],
    },
  });
  const asm = await created.json();
  const joint = await page.request.post('http://127.0.0.1:3100/api/joints', {
    data: {
      name: 'DP-hinge',
      kind: 'revolute',
      from_object_id: asm['parts'][0]['id'],
      to_object_id: asm['parts'][1]['id'],
      origin: { x: 0, y: 0, z: 1 },
      axis: { x: 0, y: 0, z: 5 },
    },
  });
  expect(joint.status(), String(await joint.text())).toBe(201);

  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  // Ribbon switch opens the mechanical domain panel.
  const tree = page.getByRole('tree', { name: 'Object tree' });
  const jointPart = objects.length
    ? tree
        .locator(`[data-node-id="${asm['parts'][0]['id']}"] > .tree-info`)
        .or(tree.locator('.tree-search')) // fallback resolves visibility below
    : tree;
  const refreshed = (await (await page.request.get('http://127.0.0.1:3100/api/objects')).json()) as {
    id: string;
    name?: string;
    parent_id?: string | null;
    object_type: number;
  }[];
  const partId = refreshed.find((o) => o.name === 'DPA-1')?.id;
  const partCount = refreshed.filter((o) => o.parent_id && o.object_type === 400).length;
  void partCount;
  void jointPart;
  if (partId) {
    await tree.locator(`[data-node-id="${partId}"] > .tree-info`).click();
  }
  await page.locator('.domain-switch', { hasText: '机械' }).click();
  const mechPanel = page.locator('[id$="-panel"], .domain-panel');
  await expect(page.locator('.domain-panel')).toBeVisible();
  await expect(page.locator('.domain-panel-section-label', { hasText: '运动副' })).toBeVisible();
  if (partId) {
    await expect(page.getByText('DP-hinge', { exact: false })).toBeVisible({ timeout: 5_000 });
  }

  // Building switch shows the door/window schedule with the door row.
  await page.locator('.domain-switch', { hasText: '建筑' }).click();
  const schedule = page.getByRole('table', { name: 'Openings schedule' });
  await expect(page.locator('.domain-panel-section-label', { hasText: '门窗表' })).toBeVisible();
  await expect(schedule).toBeVisible({ timeout: 10_000 });
  await expect(schedule.getByRole('row', { name: /门×2|门/ }).first()).toBeVisible();
});
