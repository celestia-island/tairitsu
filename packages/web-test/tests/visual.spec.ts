import { test, expect } from '@playwright/test';
import { PAGES } from '../page-registry';

test.describe('Visual Regression - Screenshot Capture', () => {
  for (const pageSpec of PAGES) {
    test(`${pageSpec.category}/${pageSpec.name} renders correctly`, async ({ page }) => {
      await page.goto(pageSpec.url);
      await page.waitForLoadState('domcontentloaded');
      await page.locator('#hikari-app').waitFor({ timeout: 15000 });

      const target = pageSpec.selector
        ? page.locator(pageSpec.selector)
        : page;

      await expect(target).toHaveScreenshot(`${pageSpec.name}.png`, {
        maxDiffPixelRatio: 0.01,
      });
    });
  }
});

test.describe('Visual Regression - Interactive States', () => {
  for (const pageSpec of PAGES.filter(p => p.interactions && p.interactions.length > 0)) {
    for (const interaction of pageSpec.interactions!) {
      test(`${pageSpec.name}${interaction.suffix}`, async ({ page }) => {
        await page.goto(pageSpec.url);
        await page.waitForLoadState('domcontentloaded');
        await page.locator('#hikari-app').waitFor({ timeout: 15000 });

        const el = page.locator(interaction.selector);
        await expect(el.first()).toBeVisible();

        switch (interaction.action) {
          case 'click':
            await el.first().click();
            break;
          case 'hover':
            await el.first().hover();
            break;
          case 'focus':
            await el.first().focus();
            break;
        }

        await page.waitForTimeout(300);

        await expect(page).toHaveScreenshot(
          `${pageSpec.name}${interaction.suffix}.png`,
          { maxDiffPixelRatio: 0.01 },
        );
      });
    }
  }
});
