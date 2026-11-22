import { expect, test } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { openSignedIn } from './support';

const port = process.env.E2E_PORT ?? '8181';

// Runs with playwright.reconnect.config.ts: this test starts and stops the server itself.
const serverBinary = join(__dirname, '../../target/debug/veltrunox-server');
const dataDir = mkdtempSync(join(tmpdir(), 'veltrunox-e2e-reconnect-'));
const configFile = join(dataDir, 'veltrunox.toml');
writeFileSync(
  configFile,
  [
    `address = "127.0.0.1:${port}"`,
    `data_dir = ${JSON.stringify(join(dataDir, 'data'))}`,
    `web_dir = ${JSON.stringify(join(__dirname, '../../app/build/web'))}`,
  ].join('\n'),
);
let server: ChildProcess | undefined;

async function startServer() {
  server = spawn(serverBinary, ['--config', configFile], {
    stdio: 'ignore',
  });
  await expect
    .poll(() => fetch(`http://127.0.0.1:${port}/api/v1/status`).then((r) => r.ok, () => false), {
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
  await openSignedIn(page);
  await expect(page.getByText('Verbunden')).toBeVisible();

  await stopServer();
  await expect(page.getByText('Getrennt')).toBeVisible({ timeout: 15_000 });

  await startServer();
  await expect(page.getByText('Verbunden')).toBeVisible({ timeout: 60_000 });
});
