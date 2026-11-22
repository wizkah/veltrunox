import { expect, test } from '@playwright/test';
import { openSignedIn } from './support';

test('lists the recordings and shows the connection status', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });

  await openSignedIn(page);

  // A tile is one button named after the recording; it holds the re-analyse button.
  await expect(page.getByRole('button', { name: 'Soundcheck', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Verse and chorus run-through', exact: true })).toBeVisible();
  await expect(page.getByText('Verbunden')).toBeVisible();
  expect(errors).toEqual([]);
});
