import { defineConfig } from '@playwright/test';

const webUrl = 'http://127.0.0.1:8081';
const serverUrl = 'http://127.0.0.1:8080';

// Needs the web app built first: scripts/build-web.sh
export default defineConfig({
  testDir: './tests',
  use: { baseURL: webUrl },
  webServer: [
    {
      command: 'cargo run -q -p veltrunox-server',
      cwd: '..',
      url: `${serverUrl}/api/v1/recordings`,
      env: {
        VELTRUNOX_ADDRESS: '127.0.0.1:8080',
        VELTRUNOX_ALLOWED_ORIGIN: webUrl,
      },
      reuseExistingServer: !process.env.CI,
      timeout: 300_000,
    },
    {
      command: 'python3 -m http.server 8081 --bind 127.0.0.1',
      cwd: '../app/build/web',
      url: webUrl,
      reuseExistingServer: !process.env.CI,
    },
  ],
});
