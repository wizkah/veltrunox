import { expect, type Page } from '@playwright/test';

// The first test on a fresh server sets up this admin; later tests sign in with it.
export const ADMIN = { userName: 'tester', password: 'a long test password' };

// Flutter web draws to a canvas; its accessibility layer exposes the texts as DOM.
export async function enableAccessibility(page: Page) {
  await page.locator('flt-semantics-placeholder').evaluate((element) => (element as HTMLElement).click());
}

// Flutter reads text from real key presses; setting the value of the page's input element
// does not always reach the text field.
async function typeInto(page: Page, label: string, text: string, exact = false) {
  const field = page.getByRole('textbox', { name: label, exact });
  await field.dispatchEvent('click');
  await field.focus();
  // Keys typed right after the focus moved can get lost.
  await page.waitForTimeout(300);
  await page.keyboard.type(text, { delay: 10 });
}

/** Opens the app and gets past the sign-in (or the first-start setup) as the given account. */
export async function openSignedIn(page: Page, account = ADMIN) {
  await page.goto('/');
  await enableAccessibility(page);
  const setupTitle = page.getByText('Ersten Admin einrichten');
  const signIn = page.getByRole('button', { name: 'Anmelden', exact: true });
  await setupTitle.or(signIn).first().waitFor();
  await typeInto(page, 'Benutzername', account.userName);
  await typeInto(page, 'Passwort', account.password, true);
  if (await setupTitle.isVisible()) {
    await typeInto(page, 'Passwort wiederholen', account.password);
    await page.getByRole('button', { name: 'Einrichten' }).dispatchEvent('click');
  } else {
    await signIn.dispatchEvent('click');
  }
  await expect(page.getByRole('button', { name: 'Konto' })).toBeVisible();
}

/** A one-second mono 16-bit WAV of a tone. */
export function wav(hertz: number): Buffer {
  const rate = 8000;
  const samples = Buffer.alloc(rate * 2);
  for (let frame = 0; frame < rate; frame++) {
    samples.writeInt16LE(Math.round(0.3 * Math.sin((2 * Math.PI * hertz * frame) / rate) * 32767), frame * 2);
  }
  const header = Buffer.alloc(44);
  header.write('RIFF', 0);
  header.writeUInt32LE(36 + samples.length, 4);
  header.write('WAVEfmt ', 8);
  header.writeUInt32LE(16, 16);
  header.writeUInt16LE(1, 20);
  header.writeUInt16LE(1, 22);
  header.writeUInt32LE(rate, 24);
  header.writeUInt32LE(rate * 2, 28);
  header.writeUInt16LE(2, 32);
  header.writeUInt16LE(16, 34);
  header.write('data', 36);
  header.writeUInt32LE(samples.length, 40);
  return Buffer.concat([header, samples]);
}
