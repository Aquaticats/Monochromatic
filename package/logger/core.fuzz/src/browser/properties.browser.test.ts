/**
 Drives the browser property bundle through the Playwright harness page:
 loads the bundle built by `bundle:browser-properties`, runs every browser property
 in the page, and fails on any property fast-check could falsify. A
 backend the browser lacks reports itself skipped, never failed.

 Local podman run only until a browser CI job exists:
 `mise run //package/logger/core.fuzz:bundle:browser-properties`, then
 `mise run test:browser:chromium -- package/logger/core.fuzz/src/browser/properties.browser.test.ts`.

 @module
 */

import {
  expect,
  test,
} from '@playwright/test';

import type { BrowserPropertyRunner, } from './report.ts';

declare global {
  /**
   Runner the property bundle installs on the page.
   */
  var loggerFuzz: BrowserPropertyRunner;
}

/**
 Bundle path the harness server exposes under the sidecar's dist route.
 */
const BUNDLE_URL = '/dist/module-logger.fuzz/client/properties.js';

/**
 fast-check runs per property; each run writes up to a dozen records and
 reparses a backend, so this stays well inside the test timeout.
 */
const NUM_RUNS = 25;

/**
 Whole-test timeout covering three properties over real browser storage.
 */
const TEST_TIMEOUT_MS = 180_000;

test.describe(
  'logger browser sinks under adversarial records',
  () => {
    test(
      'every browser backend reparses the exact records, or reports itself unavailable',
      async ({ page, },) => {
        test.setTimeout(TEST_TIMEOUT_MS,);
        await page.goto('/',);
        await page.addScriptTag({
          type: 'module',
          url: BUNDLE_URL,
        },);
        await page.waitForFunction(() => globalThis.loggerFuzz !== undefined);
        /**
         Report the page produced.
         */
        const report = await page.evaluate(
          (numRuns,) => globalThis.loggerFuzz.run({ numRuns, },),
          NUM_RUNS,
        );
        // Precise terminal output: the dot reporter shows nothing else about
        // which backend ran, passed, or was skipped in this browser.
        console.log(`${report.userAgent}\n${report.results
          .map((result,) => `  ${result.status.padEnd(7,)} ${result.name}${(result.detail === undefined) ? '' : `: ${result.detail}`}`)
          .join('\n',)}`,);
        for (const result of report.results)
          expect(
            result.status,
            `${result.name} (${report.userAgent}): ${result.detail ?? ''}`,
          )
            .not
            .toBe('failed',);
        expect(report.results.filter((result,) => result.status === 'passed').length,)
          .toBeGreaterThan(0,);
      },
    );
  },
);
