/**
 E2e tests for `onLoadRedirectingTo`.
 
 The match path navigates to `test/fixture/redirect-target.html` so
 `page.waitForURL` can confirm `location.replace` fired with the anchor's href.
 
 @module
 */
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  expect,
  test,
} from '@playwright/test';

import { loadHarness, } from './test-setup.ts';

/** Absolute file URL of the redirect target used by the match-path test. */
const TARGET_URL = pathToFileURL(
  join(import.meta.dirname, '..', 'test', 'fixture', 'redirect-target.html',),
)
  .href;

test.describe('onLoadRedirectingTo', () => {
  test('is a no-op when no a.redirectingTo element exists', async ({ page, },) => {
    await loadHarness({ page, },);

    const urlBefore = page.url();
    await page.evaluate(function callWithoutAnchor() {
      globalThis.moduleDom.onLoadRedirectingTo(20,);
    },);

    // Wait long enough for the redirect timer to fire if it were going to.
    await page.waitForTimeout(100,);

    expect(page.url(),).toBe(urlBefore,);
  });

  test('replaces location with the anchor href after the delay', async ({ page, },) => {
    await loadHarness({ page, },);

    await page.evaluate(function seedAnchorAndCall(targetUrl,) {
      const anchor = document.createElement('a',);
      anchor.className = 'redirectingTo';
      anchor.href = targetUrl;
      anchor.textContent = 'redirecting';
      document.body.append(anchor,);
      globalThis.moduleDom.onLoadRedirectingTo(20,);
    }, TARGET_URL,);

    await page.waitForURL(TARGET_URL,);
    const marker = await page.locator('#marker',).textContent();
    expect(marker,).toBe('redirect-target-reached',);
  });
});
