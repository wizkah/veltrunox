import { expect, test } from '@playwright/test';

// Flutter web draws to a canvas; its accessibility layer exposes the texts as DOM.
async function enableAccessibility(page: import('@playwright/test').Page) {
  await page.locator('flt-semantics-placeholder').evaluate((element) => (element as HTMLElement).click());
}

test('lists the recordings and shows the connection status', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });

  await page.goto('/');
  await enableAccessibility(page);

  await expect(page.getByText('Soundcheck')).toBeVisible();
  await expect(page.getByText('Verse and chorus run-through')).toBeVisible();
  await expect(page.getByText('Verbunden')).toBeVisible();
  expect(errors).toEqual([]);
});
