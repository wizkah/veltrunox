import { expect, test, type Locator, type Page } from '@playwright/test';

// Flutter web draws to a canvas; its accessibility layer exposes the texts as DOM.
// Another semantics node can cover a button, so a real mouse click is intercepted.
async function press(button: Locator) {
  await button.dispatchEvent('click');
}

// A selection resets the shared state; the controls look the same before and after,
// so give it a moment to arrive before the next command.
async function select(page: Page, title: string) {
  await press(page.getByRole('button', { name: title, exact: true }));
  await page.waitForTimeout(500);
}

async function open(page: Page) {
  await page.goto('/');
  await page.locator('flt-semantics-placeholder').evaluate((element) => (element as HTMLElement).click());
  await expect(page.getByText('Verbunden')).toBeVisible();
}

test('two browser clients share the playback state', async ({ browser }) => {
  const errors: string[] = [];
  const first = await (await browser.newContext()).newPage();
  const second = await (await browser.newContext()).newPage();
  for (const page of [first, second]) {
    page.on('console', (message) => {
      if (message.type() === 'error') errors.push(message.text());
    });
  }
  await open(first);
  await open(second);
  await expect(second.getByText('Keine Aufnahme gewählt')).toBeVisible();

  await select(first, 'Soundcheck');
  await expect(second.getByRole('button', { name: 'Abspielen' })).toBeVisible();

  await press(first.getByRole('button', { name: 'Abspielen' }));
  await expect(second.getByRole('button', { name: 'Pause' })).toBeVisible();

  await press(second.getByRole('button', { name: 'Pause' }));
  await expect(first.getByRole('button', { name: 'Abspielen' })).toBeVisible();
  expect(errors).toEqual([]);
});

test('a client joining during playback shows the running state at once', async ({ browser }) => {
  const first = await (await browser.newContext()).newPage();
  await open(first);
  await select(first, 'Soundcheck');
  await press(first.getByRole('button', { name: 'Abspielen' }));
  await expect(first.getByRole('button', { name: 'Pause' })).toBeVisible();

  const late = await (await browser.newContext()).newPage();
  await open(late);
  await expect(late.getByRole('button', { name: 'Pause' })).toBeVisible();
  await press(first.getByRole('button', { name: 'Pause' }));
});
