import {
  defineConfig,
  devices,
  type PlaywrightTestConfig,
} from '@playwright/test';

/**
 End-to-end Playwright configuration with an explicit public export type for isolated declarations.
 */
const config: PlaywrightTestConfig = defineConfig({
  testMatch: '**/*.e2e.test.ts',
  fullyParallel: true,
  retries: process.env
    .CI ? 2 : 0,
  reporter: [
    ['dot',],
    [
      'html',
      { open: 'never', },
    ],
  ],

  use: {
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'], },
    },
  ],
},);

export default config;
