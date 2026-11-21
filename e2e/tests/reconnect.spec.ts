import { expect, test } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// Runs with playwright.reconnect.config.ts: this test starts and stops the server itself.
const serverBinary = join(__dirname, '../../target/debug/veltrunox-server');
const dataDir = mkdtempSync(join(tmpdir(), 'veltrunox-e2e-reconnect-'));
let server: ChildProcess | undefined;

async function startServer() {
  server = spawn(serverBinary, [], {
    env: {
      ...process.env,
      VELTRUNOX_ADDRESS: '127.0.0.1:8080',
      VELTRUNOX_DATA_DIR: dataDir,
      VELTRUNOX_ALLOWED_ORIGIN: 'http://127.0.0.1:8081',
    },
    stdio: 'ignore',
  });
  await expect
    .poll(() => fetch('http://127.0.0.1:8080/api/v1/recordings').then((r) => r.ok, () => false), {
      timeout: 30_000,
    })
    .toBe(true);
}

async function stopServer() {
  const process = server;
  server = undefined;
  if (!process) return;
  const exited = new Promise((resolve) => process.once('exit', resolve));
  process.kill();
  await exited;
}

test.afterEach(stopServer);

test('the browser client reconnects after the server restarts', async ({ page }) => {
  await startServer();
  await page.goto('/');
  await page.locator('flt-semantics-placeholder').evaluate((element) => (element as HTMLElement).click());
  await expect(page.getByText('Verbunden')).toBeVisible();

  await stopServer();
  await expect(page.getByText('Getrennt')).toBeVisible({ timeout: 15_000 });

  await startServer();
  await expect(page.getByText('Verbunden')).toBeVisible({ timeout: 60_000 });
});
