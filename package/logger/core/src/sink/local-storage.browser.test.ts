// oxlint-disable typescript/no-unsafe-assignment, typescript/no-unsafe-member-access, typescript/strict-boolean-expressions -- browser evaluate callbacks lose type info across page boundary

import {
  expect,
  test,
} from '@playwright/test';

declare global {
  // oxlint-disable-next-line typescript/consistent-type-imports -- typeof import() cannot use import type syntax
  var moduleLogger: typeof import('@monochromatic-dev/logger-core');
}

test.describe('localStorage sink', () => {
  test.beforeEach(async ({ page, },) => {
    await page.goto('/',);
    await page.waitForFunction(() => globalThis.moduleLogger !== undefined);
  },);

  test('createLocalStorageSink exposes a callable verify', async ({ page, },) => {
    const typeofVerify = await page.evaluate(() => {
      const { createLocalStorageSink, } = globalThis.moduleLogger.sinks;
      return typeof createLocalStorageSink().verify;
    },);
    expect(typeofVerify,).toBe('function',);
  });

  test('verify detects availability', async ({ page, },) => {
    const result = await page.evaluate(async () => {
      const { createLocalStorageSink, } = globalThis.moduleLogger.sinks;
      return createLocalStorageSink().verify();
    },);
    expect(result,).toBe(true,);
  });

  test('a partial process shim does not prevent browser storage detection', async ({ page, },): Promise<void> => {
    /** This synthetic process belongs only to Playwright's disposable page. */
    const available = await page.evaluate(async (): Promise<boolean> => {
      Object.defineProperty(globalThis, 'process', { configurable: true, value: { env: {}, }, },);
      return await globalThis.moduleLogger.sinks.createLocalStorageSink().verify();
    },);
    expect(available,).toBe(true,);
  },);

  test('a verified sink writes records across levels and message shapes', async ({ page, },) => {
    const allSucceeded = await page.evaluate(async () => {
      const { createLocalStorageSink, } = globalThis.moduleLogger.sinks;
      const sink = createLocalStorageSink();
      await sink.verify();
      const levels = ['trace', 'debug', 'info', 'warn', 'error', 'fatal',] as const;
      const messages = [
        'test message',
        'Hello 世界 🌍',
        '',
        '{"key": "value", "nested": {"a": 1}}',
      ];
      for (const level of levels) {
        for (const message of messages) {
          try {
            void sink.write({
              level,
              message,
              timestamp: Date.now(),
            },);
          }
          catch (error: unknown) {
            console.warn('localStorage sink browser test write failed', error,);
            return false;
          }
        }
      }
      return true;
    },);
    expect(allSucceeded,).toBe(true,);
  });

  test('written records can be retrieved from localStorage under a run-scoped key', async ({ page, },) => {
    const result = await page.evaluate(async () => {
      const { createLocalStorageSink, } = globalThis.moduleLogger.sinks;

      // Clear any existing logs first.
      const keysToRemove: string[] = [];
      const storageLength = globalThis.localStorage.length;
      for (let storageIndex = 0; storageIndex < storageLength; storageIndex++) {
        const key = globalThis.localStorage.key(storageIndex,);
        if (key?.startsWith('monochromatic.log',))
          keysToRemove.push(key,);
      }
      keysToRemove.forEach(key => {
        globalThis.localStorage.removeItem(key,);
      },);

      const sink = createLocalStorageSink();
      await sink.verify();

      const testMessage = `unique-test-${Date.now()}`;
      await sink.write({
        level: 'info' as const,
        message: testMessage,
        timestamp: Date.now(),
      },);
      // Routine severity buffers; the flush hook forces the batch out so the
      // read-back below observes it deterministically.
      await sink.flush?.();

      // Find the written record under a run-scoped key
      // (`monochromatic.log.{stamp}.{nonce}.{index}`).
      const currentStorageLength = globalThis.localStorage.length;
      for (let storageIndex = 0; storageIndex < currentStorageLength; storageIndex++) {
        const key = globalThis.localStorage.key(storageIndex,);
        if (key?.startsWith('monochromatic.log',)) {
          const value = globalThis.localStorage.getItem(key,);
          if (value?.includes(testMessage,)) {
            const parsed = JSON.parse(value,);
            return {
              found: true,
              message: parsed.message,
              level: parsed.level,
              keySegments: key.split('.',).length,
              testMessage,
            };
          }
        }
      }

      return { found: false, message: null, level: null, keySegments: 0, testMessage, };
    },);

    expect(result.found,).toBe(true,);
    expect(result.message,).toBe(result.testMessage,);
    expect(result.level,).toBe('info',);
    // Prefix `monochromatic.log` plus stamp, nonce, and index.
    expect(result.keySegments,).toBe(5,);
  });
});
