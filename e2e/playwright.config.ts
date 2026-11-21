import { defineConfig } from '@playwright/test';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// The server reads its settings from a configuration file (TOML strings accept JSON quoting).
const testDirectory = mkdtempSync(join(tmpdir(), 'veltrunox-e2e-'));
const configFile = join(testDirectory, 'veltrunox.toml');
writeFileSync(
  configFile,
  [
    'address = "127.0.0.1:8080"',
    `data_dir = ${JSON.stringify(join(testDirectory, 'data'))}`,
    `web_dir = ${JSON.stringify(join(__dirname, '../app/build/web'))}`,
  ].join('\n'),
);

// The server serves the web app, so page and API share one origin.
const serverUrl = 'http://127.0.0.1:8080';

// Needs the web app built first: scripts/build-web.sh
export default defineConfig({
  testDir: './tests',
  testIgnore: 'reconnect.spec.ts',
  use: { baseURL: serverUrl },
  webServer: {
    command: `cargo run -q -p veltrunox-server -- --config ${configFile}`,
    cwd: '..',
    url: `${serverUrl}/api/v1/recordings`,
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
