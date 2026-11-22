import { expect, test, type Page } from '@playwright/test';
import { ADMIN, enableAccessibility, openSignedIn, wav } from './support';

const MEMBER = { userName: 'member', password: 'another long password' };

async function api(page: Page, method: string, path: string, token?: string, body?: unknown) {
  const response = await page.request.fetch(path, {
    method,
    headers: token ? { authorization: `Bearer ${token}` } : {},
    data: body,
  });
  return { status: response.status(), json: await response.json().catch(() => null) };
}

/** Sets the server up (or signs in) as the admin and returns the admin's token. */
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

async function openAccountMenu(page: Page) {
  await page.getByRole('button', { name: 'Konto' }).click();
  // The menu is open once its sign-out entry shows; only then can a missing entry mean anything.
  await expect(page.getByRole('menuitem', { name: 'Abmelden' })).toBeVisible();
}

test('access follows the guest switch and the role', async ({ page }) => {
  const admin = await adminToken(page);
  await api(page, 'POST', '/api/v1/users', admin, {
    user_name: MEMBER.userName,
    password: MEMBER.password,
    role: 'user',
  });
  await api(page, 'PUT', '/api/v1/guest-access', admin, { enabled: false });
  const upload = await page.request.post('/api/v1/recordings', {
    headers: { authorization: `Bearer ${admin}`, 'x-file-name': 'Access Song.wav' },
    data: wav(440),
  });
  expect([200, 201]).toContain(upload.status());

  try {
    // Guest access off: the sign-in is all an anonymous visitor gets.
    await page.goto('/');
    await enableAccessibility(page);
    await expect(page.getByRole('button', { name: 'Anmelden', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Als Gast fortfahren' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Aufnahmen hochladen' })).toHaveCount(0);

    // Guest access on: the guest gets in and may upload, but not manage.
    await api(page, 'PUT', '/api/v1/guest-access', admin, { enabled: true });
    await page.reload();
    await enableAccessibility(page);
    await page.getByRole('button', { name: 'Als Gast fortfahren' }).dispatchEvent('click');
    await expect(page.getByRole('button', { name: 'Aufnahmen hochladen' })).toBeVisible();
    await page.getByRole('button', { name: /^Access Song/ }).first().dispatchEvent('click');
    await expect(page.getByRole('button', { name: 'Umbenennen' })).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Löschen', exact: true })).toHaveCount(0);
    await openAccountMenu(page);
    await expect(page.getByRole('menuitem', { name: 'Benutzer verwalten' })).toHaveCount(0);
  } finally {
    await api(page, 'PUT', '/api/v1/guest-access', admin, { enabled: false });
  }
});

test('a user uploads but neither manages recordings nor users', async ({ page }) => {
  await adminToken(page);
  await openSignedIn(page, MEMBER);
  await expect(page.getByRole('button', { name: 'Aufnahmen hochladen' })).toBeVisible();
  await page.getByRole('button', { name: /^Access Song/ }).first().dispatchEvent('click');
  await expect(page.getByRole('button', { name: 'Umbenennen' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Löschen', exact: true })).toHaveCount(0);
  await openAccountMenu(page);
  await expect(page.getByRole('menuitem', { name: 'Benutzer verwalten' })).toHaveCount(0);
});

test('the admin manages recordings and users', async ({ page }) => {
  await openSignedIn(page);
  await page.getByRole('button', { name: /^Access Song/ }).first().dispatchEvent('click');
  await expect(page.getByRole('button', { name: 'Umbenennen' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Löschen', exact: true })).toBeVisible();
  await openAccountMenu(page);
  await expect(page.getByRole('menuitem', { name: 'Benutzer verwalten' })).toBeVisible();
});

test('signing out returns to the sign-in', async ({ page }) => {
  await openSignedIn(page);
  await openAccountMenu(page);
  await page.getByRole('menuitem', { name: 'Abmelden' }).click();
  await expect(page.getByRole('button', { name: 'Anmelden', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Aufnahmen hochladen' })).toHaveCount(0);
});
