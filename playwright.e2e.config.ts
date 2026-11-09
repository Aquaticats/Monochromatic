import {
  defineConfig,
  devices,
  type PlaywrightTestConfig,
} from '@playwright/test';

/**
 End-to-end Playwright configuration with an explicit public export type for isolated declarations.
 */
const config: PlaywrightTestConfig = defineConfig({
  // Only the installed workspace packages. Without it the test directory is this file's own directory and the
  // `**/*.e2e.test.ts` match reaches `package-paused/`, whose packages are outside the `package/*/*` pnpm workspace
  // glob, so their imports do not resolve and collection failed before any browser started (measured
  // 2026-11-09T22:26:00-05:00 inside the test container: `Cannot find package '@monochromatic-dev/or-throw-core'
  // imported from /work/package-paused/webapp-edu/paper2vn/src/paper2vn.e2e.test.ts`). The browser config names its
  // test directory the same way.
  testDir: './package',
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
