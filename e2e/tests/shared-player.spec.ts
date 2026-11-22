import { expect, test, type Locator, type Page } from '@playwright/test';
import { openSignedIn } from './support';

// Flutter web draws to a canvas; its accessibility layer exposes the texts as DOM.
// Another semantics node can cover a button, so a real mouse click is intercepted.
async function press(button: Locator) {
  await button.dispatchEvent('click');
}

// A selection resets the shared state; the controls look the same before and after,
// so give it a moment to arrive before the next command. The library shows the details
// of a recording first; opening it there leads to the player.
async function select(page: Page, title: string) {
  // The tile of the loaded recording also says so in its name.
  await press(page.getByRole('button', { name: new RegExp(`^${title}( Im Player geladen)?$`) }));
  await press(page.getByRole('button', { name: 'Im Player öffnen', exact: true }));
  await page.waitForTimeout(500);
}

async function showPlayer(page: Page) {
  await press(page.getByRole('button', { name: 'Player', exact: true }));
}

async function open(page: Page) {
  await openSignedIn(page);
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
  await showPlayer(second);
  await expect(second.getByText('Keine Aufnahme gewählt')).toBeVisible();

  await select(first, 'Soundcheck');
  await expect(second.getByRole('button', { name: 'Abspielen', exact: true })).toBeVisible();

  await press(first.getByRole('button', { name: 'Abspielen', exact: true }));
  await expect(second.getByRole('button', { name: 'Pause', exact: true })).toBeVisible();

  await press(second.getByRole('button', { name: 'Pause', exact: true }));
  await expect(first.getByRole('button', { name: 'Abspielen', exact: true })).toBeVisible();
  expect(errors).toEqual([]);
});

test('a client joining during playback shows the running state at once', async ({ browser }) => {
  const first = await (await browser.newContext()).newPage();
  await open(first);
  await select(first, 'Soundcheck');
  await press(first.getByRole('button', { name: 'Abspielen', exact: true }));
  await expect(first.getByRole('button', { name: 'Pause', exact: true })).toBeVisible();

  const late = await (await browser.newContext()).newPage();
  await open(late);
  await showPlayer(late);
  await expect(late.getByRole('button', { name: 'Pause', exact: true })).toBeVisible();
  await press(first.getByRole('button', { name: 'Pause', exact: true }));
});

test('the play mode is shared between the clients', async ({ browser }) => {
  const first = await (await browser.newContext()).newPage();
  const second = await (await browser.newContext()).newPage();
  await open(first);
  await open(second);
  await select(first, 'Soundcheck');
  await showPlayer(second);

  await expect(first.getByRole('button', { name: /Abspielmodus: Bis Ende/ })).toBeVisible();
  await press(first.getByRole('button', { name: /Abspielmodus: Bis Ende/ }));
  await expect(second.getByRole('button', { name: /Abspielmodus: Regionsende/ })).toBeVisible();
  await press(second.getByRole('button', { name: /Abspielmodus: Regionsende/ }));
  await press(second.getByRole('button', { name: /Abspielmodus: Schleife/ }));
  await expect(first.getByRole('button', { name: /Abspielmodus: Bis Ende/ })).toBeVisible();
});
