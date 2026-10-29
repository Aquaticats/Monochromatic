/**
 E2e tests for `onLoadSetCssFromUrlParams`.
 
 Drives the test harness in `test/harness.html` via Playwright;
 `loadHarness` injects the built bundle inline so `globalThis.moduleDom`
 is available regardless of browser-specific `file://` module-script restrictions.
 
 @module
 */
import {
  expect,
  test,
} from '@playwright/test';

import { loadHarness, } from '../test-setup.ts';

test.describe('onLoadSetCssFromUrlParams', () => {
  test('applies every URL param when no allowlist is supplied', async ({ page, },) => {
    await loadHarness({ page, query: '--brand=red&font-size=16px', },);

    const result = await page.evaluate(function callUnfiltered() {
      globalThis.moduleDom.onLoadSetCssFromUrlParams();
      return {
        brand: document.documentElement.style.getPropertyValue('--brand',),
        fontSize: document.documentElement.style.getPropertyValue('font-size',),
      };
    },);

    expect(result.brand,).toBe('red',);
    expect(result.fontSize,).toBe('16px',);
  });

  test('applies only allow-listed keys when an allowlist is supplied', async ({ page, },) => {
    await loadHarness({ page, query: '--brand=red&font-size=16px', },);

    const result = await page.evaluate(function callFiltered() {
      globalThis.moduleDom.onLoadSetCssFromUrlParams(['--brand',],);
      return {
        brand: document.documentElement.style.getPropertyValue('--brand',),
        fontSize: document.documentElement.style.getPropertyValue('font-size',),
      };
    },);

    expect(result.brand,).toBe('red',);
    expect(result.fontSize,).toBe('',);
  });

  test('is a no-op when the query string has no matching keys', async ({ page, },) => {
    await loadHarness({ page, query: '--brand=red', },);

    const result = await page.evaluate(function callMismatch() {
      globalThis.moduleDom.onLoadSetCssFromUrlParams(['--nonexistent',],);
      return {
        brand: document.documentElement.style.getPropertyValue('--brand',),
        nonexistent: document.documentElement.style.getPropertyValue('--nonexistent',),
      };
    },);

    expect(result.brand,).toBe('',);
    expect(result.nonexistent,).toBe('',);
  });

  test('silently drops unknown standard CSS keys via setProperty', async ({ page, },) => {
    await loadHarness({ page, query: 'notacssprop=anything', },);

    const result = await page.evaluate(function callUnknown() {
      globalThis.moduleDom.onLoadSetCssFromUrlParams();
      return {
        notacssprop: document.documentElement.style.getPropertyValue('notacssprop',),
      };
    },);

    expect(result.notacssprop,).toBe('',);
  });
});
