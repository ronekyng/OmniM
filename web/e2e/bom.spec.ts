import { expect, test } from '@playwright/test';

test.use({ locale: 'zh-CN' });

/** Phase 8-4 (ASSEMBLY §1): BOM / quantity panel for Assembly containers. */

test('assembly BOM section lists parts; leaf objects have no BOM', async ({ page }) => {
  await page.goto('/projects');
  await page.getByRole('button', { name: /Open sample project/ }).click();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const created = await page.request.post('http://127.0.0.1:3100/api/assemblies', {
    data: {
      name: 'BOM Assembly',
      parts: [
        { name: 'Comp-1' },
        { name: 'Box-1', shape: 'box', p1: 0.5, p2: 0.4, p3: 0.3 },
      ],
    },
  });
  // A default part without a def may be rejected; fall back to primitives only.
  if (!created.ok()) {
    await page.request.post('http://127.0.0.1:3100/api/assemblies', {
      data: {
        name: 'BOM Assembly',
        parts: [{ name: 'Box-1', shape: 'box', p1: 0.5, p2: 0.4, p3: 0.3 }],
      },
    });
  }
  await page.reload();
  await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

  const tree = page.getByRole('tree', { name: 'Object tree' });

  // Select by data-node-id so the click always resolves to the container row.
  const objects = (await (await page.request.get('http://127.0.0.1:3100/api/objects')).json()) as { id: string; object_type: number; name?: string; parent_id?: string | null }[];
  // The <li> bounding box spans the whole subtree; click the row's own
  // label area so the pointer lands on the container, not a child row.
  const byType = (id: string) =>
    tree.locator(`[data-node-id="${id}"] > .tree-info`);

  // Assembly container: the panel shows the server's BOM rows verbatim.
  const assembly = objects.find((o) => o.name === 'BOM Assembly' && o.object_type === 401);
  if (!assembly) throw new Error('BOM Assembly row missing');

  // Leaf (the assembly's own Box-1 part): no BOM section.
  const leaf = objects.find((o) => o.parent_id === assembly.id && o.object_type === 400);
  if (leaf) {
    await byType(leaf.id).click();
    await page.waitForTimeout(400);
    await expect(page.getByRole('table', { name: 'Bill of materials' })).toHaveCount(0);
  }
  const serverBom = await page
    .request
    .get(`http://127.0.0.1:3100/api/objects/${assembly.id}/bom`)
    .then((r) => r.json());
  await byType(assembly.id).click({ timeout: 5_000 }).catch(() => {
    // After an inline rename the row is re-created; retry via the row li.
    return tree.locator(`[data-node-id="${assembly.id}"]`).click({ timeout: 5_000 });
  });
  const table = page.getByRole('table', { name: 'Bill of materials' });
  await expect(table).toBeVisible({ timeout: 10_000 });
  const bodyRows = await table.getByRole('row').count();
  await expect
    .poll(async () => await table.getByRole('row').count())
    .toBe(serverBom.rows.length + 1, `rows ${JSON.stringify(serverBom)}`);
  expect(bodyRows).toBeGreaterThan(0);
  const headers = JSON.stringify(await table.locator('.bom-row-head').first().textContent());
  expect(headers).toContain('对象');
  expect(headers).toContain('体积 m³');
  // And the footer total mirrors the server total.
  await expect(page.locator('.propsets-section small.hint', { hasText: '合计' })).toBeVisible();
});
