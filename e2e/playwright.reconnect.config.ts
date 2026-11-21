import { defineConfig } from '@playwright/test';

const webUrl = 'http://127.0.0.1:8081';

// The reconnect test controls the server itself; build it first: cargo build -p veltrunox-server
export default defineConfig({
  testDir: './tests',
  testMatch: 'reconnect.spec.ts',
  use: { baseURL: webUrl },
  webServer: {
    command: 'python3 -m http.server 8081 --bind 127.0.0.1',
    cwd: '../app/build/web',
    url: webUrl,
    reuseExistingServer: !process.env.CI,
  },
});
