import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 30000,
  use: {
    // Tauri launches as a desktop app, not a browser
    // The app opens at app://localhost internally
    launchOptions: {
      args: ['http://localhost:1420'],
    },
  },
});
