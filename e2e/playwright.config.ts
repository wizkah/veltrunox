import { defineConfig } from '@playwright/test';
import { mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// The server serves the web app, so page and API share one origin.
const serverUrl = 'http://127.0.0.1:8080';

// Needs the web app built first: scripts/build-web.sh
export default defineConfig({
  testDir: './tests',
  testIgnore: 'reconnect.spec.ts',
  use: { baseURL: serverUrl },
  webServer: {
    command: 'cargo run -q -p veltrunox-server',
    cwd: '..',
    url: `${serverUrl}/api/v1/recordings`,
    env: {
      VELTRUNOX_ADDRESS: '127.0.0.1:8080',
      VELTRUNOX_DATA_DIR: mkdtempSync(join(tmpdir(), 'veltrunox-e2e-')),
      VELTRUNOX_WEB_DIR: 'app/build/web',
    },
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
