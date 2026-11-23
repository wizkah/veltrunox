import { expect, test } from '@playwright/test';
import { openSignedIn, silentWav, wav } from './support';

test('uploads a file, which appears in the library, and notices a repeat', async ({ page }) => {
  await openSignedIn(page);

  async function upload(name: string, content: Buffer) {
    const chooser = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: 'Aufnahmen hochladen' }).dispatchEvent('click');
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

test('shows how much of a slow upload has arrived', async ({ page }) => {
  await openSignedIn(page);
  // A throttled connection makes the 20 MB file take a few seconds to send.
  const session = await page.context().newCDPSession(page);
  await session.send('Network.emulateNetworkConditions', {
    offline: false,
    latency: 0,
    downloadThroughput: -1,
    uploadThroughput: 5_000_000,
  });

  const chooser = page.waitForEvent('filechooser');
  await page.getByRole('button', { name: 'Aufnahmen hochladen' }).dispatchEvent('click');
  await (await chooser).setFiles({ name: 'Slow One.wav', mimeType: 'audio/wav', buffer: silentWav(10_000_000) });

  await expect(page.getByText(/^Wird hochgeladen: Slow One\.wav – \d+\u00a0%$/).first()).toBeVisible({ timeout: 30_000 });
  await session.send('Network.emulateNetworkConditions', {
    offline: false,
    latency: 0,
    downloadThroughput: -1,
    uploadThroughput: -1,
  });
  await expect(page.getByText('Hochgeladen: Slow One', { exact: true }).first()).toBeVisible({ timeout: 60_000 });
});
