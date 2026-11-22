import { defineConfig } from '@playwright/test';

// The reconnect test controls the server itself; build it first: cargo build -p veltrunox-server
export default defineConfig({
  testDir: './tests',
  testMatch: 'reconnect.spec.ts',
  use: { baseURL: `http://127.0.0.1:${process.env.E2E_PORT ?? '8181'}`, locale: 'de-DE' },
});
