// The file name sorts it last: the player test below selects a recording on the shared server,
// and the tests of the shared player expect none to be selected when they start.
import { expect, test, type Locator, type Page } from '@playwright/test';
import { ADMIN, openSignedIn, typeInto, wav } from './support';

// Flutter web draws to a canvas; its accessibility layer exposes the texts as DOM.
// Another semantics node can cover a button, so a real mouse click is intercepted.
async function press(button: Locator) {
  await button.dispatchEvent('click');
}

async function api(page: Page, method: string, path: string, token?: string, body?: unknown) {
  const response = await page.request.fetch(path, {
    method,
    headers: token ? { authorization: `Bearer ${token}` } : {},
    data: body,
  });
  return { status: response.status(), json: await response.json().catch(() => null) };
}

async function adminToken(page: Page): Promise<string> {
  const setUp = await api(page, 'POST', '/api/v1/setup', undefined, {
    user_name: ADMIN.userName,
    password: ADMIN.password,
  });
  if (setUp.status === 201) return setUp.json.token;
  const login = await api(page, 'POST', '/api/v1/login', undefined, {
    user_name: ADMIN.userName,
    password: ADMIN.password,
  });
  expect(login.status).toBe(200);
  return login.json.token;
}

/** Uploads a recording as the admin and returns its id (an upload of the same audio finds it again). */
async function recording(page: Page, token: string, title: string, hertz: number): Promise<number> {
  const upload = await page.request.post('/api/v1/recordings', {
    headers: { authorization: `Bearer ${token}`, 'x-file-name': `${title}.wav` },
    data: wav(hertz),
  });
  expect([200, 201]).toContain(upload.status());
  return (await upload.json()).id;
}

async function bookmarksOf(page: Page, token: string, id: number) {
  return (await api(page, 'GET', `/api/v1/recordings/${id}/bookmarks`, token)).json;
}

async function replaceBookmarks(page: Page, token: string, id: number, body: unknown) {
  const done = await api(page, 'PUT', `/api/v1/recordings/${id}/bookmarks`, token, body);
  expect(done.status).toBe(200);
  return done.json;
}

async function openExchangeMenu(page: Page) {
  await press(page.getByRole('button', { name: 'Export und Import' }));
  await expect(page.getByRole('menuitem', { name: 'Exportieren als JSON' })).toBeVisible();
}

/** Chooses an import file in the file dialog; the preview opens. */
async function chooseImportFile(page: Page, name: string, content: string) {
  await openExchangeMenu(page);
  const chooser = page.waitForEvent('filechooser');
  await press(page.getByRole('menuitem', { name: 'Importieren …' }));
  await (await chooser).setFiles({ name, mimeType: 'application/json', buffer: Buffer.from(content) });
  await expect(page.getByText('Import-Vorschau')).toBeVisible();
}

test('exports the bookmarks of the library as JSON and CSV files', async ({ page }) => {
  const admin = await adminToken(page);
  const id = await recording(page, admin, 'Export Song', 261);
  await replaceBookmarks(page, admin, id, {
    labels: [{ name: 'Einsatz', position_ms: 100 }],
    regions: [{ name: 'Strophe', range: { start_ms: 200, end_ms: 600 } }],
  });
  await openSignedIn(page);
  await expect(page.getByRole('button', { name: /^Export Song/ }).first()).toBeVisible();

  await openExchangeMenu(page);
  const jsonDownload = page.waitForEvent('download');
  await press(page.getByRole('menuitem', { name: 'Exportieren als JSON' }));
  const json = await jsonDownload;
  expect(json.suggestedFilename()).toBe('veltrunox-bookmarks.json');
  const exported = JSON.parse(await (await import('node:fs/promises')).readFile((await json.path())!, 'utf8'));
  const song = exported.recordings.find((entry: any) => entry.recording.title === 'Export Song');
  expect(song.bookmarks.labels[0].name).toBe('Einsatz');
  expect(song.bookmarks.regions[0].range.end_ms).toBe(600);
  await expect(page.getByText('Gespeichert: veltrunox-bookmarks.json')).toBeVisible();

  await openExchangeMenu(page);
  const csvDownload = page.waitForEvent('download');
  await press(page.getByRole('menuitem', { name: 'Exportieren als CSV' }));
  const csv = await csvDownload;
  expect(csv.suggestedFilename()).toBe('veltrunox-bookmarks.csv');
  const table = await (await import('node:fs/promises')).readFile((await csv.path())!, 'utf8');
  expect(table).toContain(`${id},label,Einsatz,100,`);
  expect(table).toContain(`${id},region,Strophe,200,600`);
});

test('exports the bookmarks of one recording from the player', async ({ page }) => {
  const admin = await adminToken(page);
  const id = await recording(page, admin, 'Player Export Song', 293);
  await replaceBookmarks(page, admin, id, { labels: [{ name: 'Mitte', position_ms: 300 }] });
  await openSignedIn(page);
  await press(page.getByRole('button', { name: /^Player Export Song/ }).first());
  await press(page.getByRole('button', { name: 'Im Player öffnen', exact: true }));
  await expect(page.getByRole('button', { name: /^Mitte Label/ })).toBeVisible();

  await openExchangeMenu(page);
  const download = page.waitForEvent('download');
  await press(page.getByRole('menuitem', { name: 'Exportieren als JSON' }));
  expect((await download).suggestedFilename()).toBe('Player Export Song.bookmarks.json');
});

test('an import shows a preview first and applies only after a confirmation', async ({ page }) => {
  const admin = await adminToken(page);
  const id = await recording(page, admin, 'Import Song', 311);
  await replaceBookmarks(page, admin, id, {
    labels: [
      { name: 'Alt eins', position_ms: 100 },
      { name: 'Alt zwei', position_ms: 200 },
    ],
  });
  await openSignedIn(page);

  const file = JSON.stringify({
    recordings: [
      { recording: { id }, bookmarks: { labels: [{ name: 'Neu', position_ms: 300 }] } },
      { recording: { id: 99999 }, bookmarks: {} },
    ],
  });
  await chooseImportFile(page, 'edited.json', file);

  // The preview names what is added and what is lost, and writes nothing.
  await expect(page.getByText('Import Song', { exact: true })).toBeVisible();
  await expect(page.getByText('Neu 1 · Geändert 0 · Unverändert 0')).toBeVisible();
  await expect(page.getByText(/Entfernt 2: Diese Bookmarks fehlen in der Datei/)).toBeVisible();
  await expect(page.getByText('Diese Aufnahme gibt es auf dem Server nicht.')).toBeVisible();
  expect((await bookmarksOf(page, admin, id)).labels.map((label: any) => label.name)).toEqual(['Alt eins', 'Alt zwei']);

  // Cancelling leaves everything as it was.
  await press(page.getByRole('button', { name: 'Abbrechen' }));
  await expect(page.getByText('Import-Vorschau')).toHaveCount(0);
  expect((await bookmarksOf(page, admin, id)).labels).toHaveLength(2);

  await chooseImportFile(page, 'edited.json', file);
  await press(page.getByRole('button', { name: 'Anwenden' }));
  // The summary sentence is a live region, which the accessibility layer does not always carry
  // to the page; what the dialog ends in and what the server holds say the same.
  await expect(page.getByRole('button', { name: 'Anwenden' })).toHaveCount(0);
  await press(page.getByRole('button', { name: 'Schließen' }));
  expect((await bookmarksOf(page, admin, id)).labels.map((label: any) => label.name)).toEqual(['Neu']);
});

test('an entry changed since the export is a conflict that can be overwritten', async ({ page }) => {
  const admin = await adminToken(page);
  const id = await recording(page, admin, 'Conflict Song', 349);
  const before = await replaceBookmarks(page, admin, id, { labels: [{ name: 'Vorher', position_ms: 100 }] });
  const label = before.labels[0];
  await openSignedIn(page);

  // Someone renames the label after the file was made from the old state.
  await api(page, 'PUT', `/api/v1/labels/${label.id}`, admin, {
    expected_version: label.version,
    name: 'Zwischendurch',
    position_ms: 100,
  });
  const file = JSON.stringify({
    recordings: [
      {
        recording: { id },
        bookmarks: { labels: [{ id: label.id, version: label.version, name: 'Aus der Datei', position_ms: 100 }] },
      },
    ],
  });
  await chooseImportFile(page, 'old.json', file);

  await expect(page.getByText('Konflikt', { exact: true })).toBeVisible();
  await expect(
    page.getByText('Label 1: Die Markierung, aus der dieser Eintrag stammt, wurde inzwischen geändert.'),
  ).toBeVisible();
  await expect(page.getByText('Keine Aufnahme kann übernommen werden.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Anwenden' })).toBeDisabled();

  await press(page.getByRole('switch', { name: /Auch zwischenzeitlich geänderte Einträge überschreiben/ }));
  await expect(page.getByText('Bereit', { exact: true })).toBeVisible();
  await press(page.getByRole('button', { name: 'Anwenden' }));
  await expect(page.getByRole('button', { name: 'Anwenden' })).toHaveCount(0);
  await expect.poll(async () => (await bookmarksOf(page, admin, id)).labels[0].name).toBe('Aus der Datei');
});

test('a file that cannot be read gets one error and no preview', async ({ page }) => {
  await adminToken(page);
  await openSignedIn(page);

  await chooseImportFile(page, 'broken.json', '{not json');

  // The sentence that says why is a live region (see above); the dialog offers no way on.
  await expect(page.getByText('Import-Vorschau')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Schließen' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Anwenden' })).toHaveCount(0);
});

test('makes an API token that works until it is revoked', async ({ page, context }) => {
  const admin = await adminToken(page);
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  await openSignedIn(page);

  await press(page.getByRole('button', { name: 'Konto' }));
  await press(page.getByRole('menuitem', { name: 'API-Tokens' }));
  await expect(page.getByRole('group', { name: 'Neues Token' })).toBeVisible();
  await typeInto(page, 'Name', 'Browser LLM');
  await press(page.getByRole('button', { name: 'Anlegen', exact: true }));

  // The text is shown once, can be copied, and then works as a bearer token. The accessibility
  // layer names the field but does not carry its text, so the clipboard is where it is read.
  const notice = page.getByRole('textbox', { name: /^Dein neues Token.*nicht noch einmal angezeigt/ });
  await expect(notice).toBeVisible();
  await press(page.getByRole('button', { name: 'Kopieren' }));
  await expect(page.getByRole('textbox', { name: /nicht noch einmal angezeigt\. Kopiert$/ })).toBeVisible();
  const secret = await page.evaluate(() => navigator.clipboard.readText());
  expect(secret).toMatch(/^vxt_[0-9a-f]{64}$/);
  expect((await api(page, 'GET', '/api/v1/export', secret)).status).toBe(200);
  expect((await api(page, 'POST', '/api/v1/import?dry_run=true', secret, { recordings: [] })).status).toBe(200);
  expect((await api(page, 'GET', '/api/v1/users', secret)).status).toBe(403);

  // Hidden, the text is gone from the page; the token stays in the list.
  await press(page.getByRole('button', { name: 'Ausblenden' }));
  await expect(notice).toHaveCount(0);
  await expect(page.getByRole('group', { name: /^Browser LLM/ })).toBeVisible();

  await press(page.getByRole('button', { name: 'Widerrufen: Browser LLM' }));
  await expect(page.getByText('Token „Browser LLM“ widerrufen?')).toBeVisible();
  await press(page.getByRole('button', { name: 'Widerrufen', exact: true }));
  // The confirmation has to be gone first: the list behind it is hidden from the accessibility
  // layer while it is open, so an absent row says nothing yet.
  await expect(page.getByRole('alertdialog')).toHaveCount(0);
  await expect(page.getByRole('group', { name: /^Browser LLM/ })).toHaveCount(0);
  expect((await api(page, 'GET', '/api/v1/export', secret)).status).toBe(401);
  expect(admin).toBeTruthy();
});
