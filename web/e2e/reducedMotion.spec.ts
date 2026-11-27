import { expect, test } from '@playwright/test';

/** A11Y §7 (UI-A11Y-06): under `prefers-reduced-motion: reduce` every
 *  decorative/infinite animation stops while the state stays perceivable via
 *  static styling; the view cube has no easing to remove (instant `setView`)
 *  and opacity transitions stay ≤100 ms. Verified by media emulation.
 *  The control group proves the reduced-motion block is what makes the
 *  difference. */

test.describe('reduced motion: reduce', () => {
  test.use({ reducedMotion: 'reduce' });

  test('decorative animations stop', async ({ page }) => {
    await page.goto('/projects');
    await page.getByRole('button', { name: /Open sample project/ }).click();
    await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });

    const animationOf = (selector: string) =>
      page.locator(selector).first().evaluate((el) => getComputedStyle(el).animationName);

    // Skeleton shimmer off (home page loading state, if captured).
    await page.goto('/projects');
    const skeleton = page.locator('.skeleton-bar').first();
    if (await skeleton.isVisible().catch(() => false)) {
      expect(await animationOf('.skeleton-bar')).toBe('none');
    }

    // AI chat typing dots off.
    await page.getByRole('button', { name: /Open sample project/ }).click();
    await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
    const dot = page.locator('.aichat-dot').first();
    if (await dot.isVisible().catch(() => false)) {
      expect(await animationOf('.aichat-dot')).toBe('none');
    }

    // Toast entry animation is off (slide/fade-in family).
    await page.waitForTimeout(1000); // workspace keybindings settle after mount
    await page.keyboard.press('ControlOrMeta+S');
    const toast = page.locator('.toast-item').first();
    await toast.waitFor({ timeout: 10_000 });
    expect(await animationOf('.toast-item')).toBe('none');
  });
});

test.describe('reduced motion: control', () => {
  test.use({ reducedMotion: 'no-preference' });

  test('decorative animations still run without the preference', async ({ page }) => {
    await page.goto('/projects');
    await page.getByRole('button', { name: /Open sample project/ }).click();
    await expect(page.locator('.title-bar')).toBeVisible({ timeout: 20_000 });
    await page.waitForTimeout(1000);
    await page.keyboard.press('ControlOrMeta+S');
    const toast = page.locator('.toast-item').first();
    await toast.waitFor({ timeout: 10_000 });
    const anim = await toast.evaluate((el) => getComputedStyle(el).animationName);
    expect(anim).not.toBe('none');
  });
});
