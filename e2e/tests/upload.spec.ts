import { expect, test } from '@playwright/test';
import { openSignedIn, wav } from './support';

test('uploads a file, which appears in the library, and notices a repeat', async ({ page }) => {
  await openSignedIn(page);

  async function upload(name: string, content: Buffer) {
    const chooser = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: 'Aufnahme hochladen' }).dispatchEvent('click');
    await (await chooser).setFiles({ name, mimeType: 'audio/wav', buffer: content });
  }

  await upload('Browser Upload.wav', wav(330));
  await expect(page.getByText('Hochgeladen: Browser Upload', { exact: true }).first()).toBeVisible({ timeout: 30_000 });
  await expect(page.getByRole('button', { name: /^Browser Upload/ }).first()).toBeVisible();

  await upload('Copy of it.wav', wav(330));
  await expect(page.getByText('Schon vorhanden: Browser Upload (nicht noch einmal angelegt)', { exact: true }).first()).toBeVisible({
    timeout: 30_000,
  });
});
