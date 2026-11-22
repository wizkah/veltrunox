import { defineConfig } from '@playwright/test';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// The server reads its settings from a configuration file (TOML strings accept JSON quoting).
// The tests use port 8181 (not the development server's 8080); E2E_PORT picks another one.
const port = process.env.E2E_PORT ?? '8181';
const testDirectory = mkdtempSync(join(tmpdir(), 'veltrunox-e2e-'));
const configFile = join(testDirectory, 'veltrunox.toml');
writeFileSync(
  configFile,
  [
    `address = "127.0.0.1:${port}"`,
    `data_dir = ${JSON.stringify(join(testDirectory, 'data'))}`,
    `web_dir = ${JSON.stringify(join(__dirname, '../app/build/web'))}`,
  ].join('\n'),
);

// The server serves the web app, so page and API share one origin.
const serverUrl = `http://127.0.0.1:${port}`;

// Needs the web app built first: scripts/build-web.sh
export default defineConfig({
  testDir: './tests',
  testIgnore: 'reconnect.spec.ts',
  // The first test sets the admin up on a fresh server; the others sign in with it.
  workers: 1,
  // The interface follows the browser's language; the tests read the German texts.
  use: { baseURL: serverUrl, locale: 'de-DE' },
  webServer: {
    command: `cargo run -q -p veltrunox-server -- --config ${configFile}`,
    cwd: '..',
    url: `${serverUrl}/api/v1/status`,
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
