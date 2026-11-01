/** Actual-browser acceptance for the consumer bundle built from the neutral harness artifact. @module */
import { expect, test, } from '@playwright/test';

for (const processShim of [false, true,]) {
  test(`ordinary browser sandbox with process shim ${String(processShim,)}`, async ({ page, },): Promise<void> => {
    /** A fresh page contains no logger scripts or unrelated fixture modules. */
    await page.route('http://localhost:3005/sinon-fixture', async route => {
      await route.fulfill({ contentType: 'text/html', body: '<!doctype html><title>Sinon browser fixture</title>', },);
    },);
    /** Any uncaught browser exception fails acceptance independently of the returned evidence. */
    const errors: string[] = [];
    page.on('pageerror', error => {
      errors.push(error.message,);
    },);
    await page.goto('/sinon-fixture',);
    /** Module import occurs in the actual browser, not in Playwright's Node test process. */
    const result: unknown = await page.evaluate(async ({ url, shim, },): Promise<unknown> => {
      /** Validate the dynamic browser export before invoking it across the page boundary. */
      const fixture: unknown = await import(url);
      if (((typeof fixture) !== 'object') || (fixture === null) || (!('runBrowserSandboxProbe' in fixture))
        || ((typeof fixture.runBrowserSandboxProbe) !== 'function'))
        throw new Error('Browser fixture did not export runBrowserSandboxProbe',);
      return await Reflect.apply(fixture.runBrowserSandboxProbe, fixture, [{ processShim: shim, },],) as unknown;
    }, { url: '/dist/module-test/client/sinon-browser-fixture.js', shim: processShim, },);
    expect(result,).toEqual({ restored: true, contextCount: 2, nodeObserverInstalled: false, processShim, },);
    expect(errors,).toEqual([],);
  },);
}
