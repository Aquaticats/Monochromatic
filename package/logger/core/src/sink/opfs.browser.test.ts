// oxlint-disable eslint/no-await-in-loop -- browser evaluate callbacks lose type info across page boundary

import {
  expect,
  test,
} from '@playwright/test';

declare global {
  // oxlint-disable-next-line typescript/consistent-type-imports -- typeof import() cannot use import type syntax
  var moduleLoggerBrowser: typeof import('@monochromatic-dev/logger-core/browser');
}

test.describe('OPFS sink', () => {
  test.beforeEach(async ({ page, },) => {
    await page.goto('/',);
    await page.waitForFunction(() => globalThis.moduleLoggerBrowser !== undefined);
  },);

  test('createOpfsSink exposes a callable verify', async ({ page, },) => {
    const typeofVerify = await page.evaluate(() => {
      const { createOpfsSink, } = globalThis.moduleLoggerBrowser;
      return typeof createOpfsSink().verify;
    },);
    expect(typeofVerify,).toBe('function',);
  });

  test('verify resolves a boolean', async ({ page, },) => {
    const resultType = await page.evaluate(async () => {
      const { createOpfsSink, } = globalThis.moduleLoggerBrowser;
      const resolved = await createOpfsSink().verify();
      return typeof resolved;
    },);
    expect(resultType,).toBe('boolean',);
  });

  test('verify detects OPFS availability', async ({ page, },) => {
    const result = await page.evaluate(async () => {
      const { createOpfsSink, } = globalThis.moduleLoggerBrowser;
      return createOpfsSink().verify();
    },);
    expect(result,).toBe(true,);
  });

  test('sink write method exists', async ({ page, },) => {
    const typeofSink = await page.evaluate(() => {
      const { createOpfsSink, } = globalThis.moduleLoggerBrowser;
      return typeof createOpfsSink().write;
    },);
    expect(typeofSink,).toBe('function',);
  });

  test('a verified sink writes records across levels and message shapes', async ({ page, },) => {
    const allSucceeded = await page.evaluate(async () => {
      const { createOpfsSink, } = globalThis.moduleLoggerBrowser;
      const sink = createOpfsSink();
      await sink.verify();
      const levels = ['trace', 'debug', 'info', 'warn', 'error', 'fatal',] as const;
      const messages = [
        'test message',
        'Hello 世界 🌍',
        '',
        '{"key": "value", "nested": {"a": 1}}',
        'line1\nline2\nline3',
      ];
      for (const level of levels) {
        for (const message of messages) {
          try {
            await Promise.resolve(sink.write({
              level,
              message,
              timestamp: Date.now(),
            },),);
          }
          catch (error: unknown) {
            console.warn('OPFS sink browser test write failed', error,);
            return false;
          }
        }
      }
      return true;
    },);
    expect(allSucceeded,).toBe(true,);
  });
});
