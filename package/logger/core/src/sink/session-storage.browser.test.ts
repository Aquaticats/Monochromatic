// oxlint-disable typescript/no-unsafe-assignment, typescript/no-unsafe-member-access, typescript/strict-boolean-expressions -- browser evaluate callbacks lose type info across page boundary

import {
  expect,
  test,
} from '@playwright/test';

declare global {
  // oxlint-disable-next-line typescript/consistent-type-imports -- typeof import() cannot use import type syntax
  var moduleLogger: typeof import('@monochromatic-dev/logger-core');
}

test.describe('sessionStorage sink', () => {
  test.beforeEach(async ({ page, },) => {
    await page.goto('/',);
    await page.waitForFunction(() => globalThis.moduleLogger !== undefined);
  },);

  test('createSessionStorageSink exposes a callable verify', async ({ page, },) => {
    const typeofVerify = await page.evaluate(() => {
      const { createSessionStorageSink, } = globalThis.moduleLogger.sinks;
      return typeof createSessionStorageSink().verify;
    },);
    expect(typeofVerify,).toBe('function',);
  });

  test('verify resolves a boolean', async ({ page, },) => {
    const resultType = await page.evaluate(async () => {
      const { createSessionStorageSink, } = globalThis.moduleLogger.sinks;
      return typeof (await createSessionStorageSink().verify());
    },);
    expect(resultType,).toBe('boolean',);
  });

  test('verify detects availability', async ({ page, },) => {
    const result = await page.evaluate(async () => {
      const { createSessionStorageSink, } = globalThis.moduleLogger.sinks;
      return createSessionStorageSink().verify();
    },);
    expect(result,).toBe(true,);
  });

  test('sink write method exists', async ({ page, },) => {
    const typeofSink = await page.evaluate(() => {
      const { createSessionStorageSink, } = globalThis.moduleLogger.sinks;
      return typeof createSessionStorageSink().write;
    },);
    expect(typeofSink,).toBe('function',);
  });

  test('a verified sink writes records across levels and message shapes', async ({ page, },) => {
    const allSucceeded = await page.evaluate(async () => {
      const { createSessionStorageSink, } = globalThis.moduleLogger.sinks;
      const sink = createSessionStorageSink();
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
            console.warn('sessionStorage sink browser test write failed', error,);
            return false;
          }
        }
      }
      return true;
    },);
    expect(allSucceeded,).toBe(true,);
  });

  test('written records can be retrieved from sessionStorage', async ({ page, },) => {
    const result = await page.evaluate(async () => {
      const { createSessionStorageSink, } = globalThis.moduleLogger.sinks;

      // Clear any existing logs first.
      const keysToRemove: string[] = [];
      const storageLength = globalThis.sessionStorage.length;
      for (let storageIndex = 0; storageIndex < storageLength; storageIndex++) {
        const key = globalThis.sessionStorage.key(storageIndex,);
        if (key?.startsWith('monochromatic.log',))
          keysToRemove.push(key,);
      }
      keysToRemove.forEach(key => {
        globalThis.sessionStorage.removeItem(key,);
      },);

      const sink = createSessionStorageSink();
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

      // Find the written record.
      const currentStorageLength = globalThis.sessionStorage.length;
      for (let storageIndex = 0; storageIndex < currentStorageLength; storageIndex++) {
        const key = globalThis.sessionStorage.key(storageIndex,);
        if (key?.startsWith('monochromatic.log',)) {
          const value = globalThis.sessionStorage.getItem(key,);
          if (value?.includes(testMessage,)) {
            const parsed = JSON.parse(value,);
            return {
              found: true,
              message: parsed.message,
              level: parsed.level,
              testMessage,
            };
          }
        }
      }

      return { found: false, message: null, level: null, testMessage, };
    },);

    expect(result.found,).toBe(true,);
    expect(result.message,).toBe(result.testMessage,);
    expect(result.level,).toBe('info',);
  });
});
