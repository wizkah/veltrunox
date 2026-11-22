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

/** Opens the app and gets past the sign-in (or the first-start setup) as the test admin. */
export async function openSignedIn(page: Page) {
  await page.goto('/');
  await enableAccessibility(page);
  const setupTitle = page.getByText('Ersten Admin einrichten');
  const signIn = page.getByRole('button', { name: 'Anmelden', exact: true });
  await setupTitle.or(signIn).first().waitFor();
  await typeInto(page, 'Benutzername', ADMIN.userName);
  await typeInto(page, 'Passwort', ADMIN.password, true);
  if (await setupTitle.isVisible()) {
    await typeInto(page, 'Passwort wiederholen', ADMIN.password);
    await page.getByRole('button', { name: 'Einrichten' }).dispatchEvent('click');
  } else {
    await signIn.dispatchEvent('click');
  }
  await expect(page.getByRole('button', { name: 'Konto' })).toBeVisible();
}
