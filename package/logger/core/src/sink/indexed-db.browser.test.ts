// oxlint-disable typescript/no-unsafe-assignment, typescript/no-unsafe-member-access -- browser evaluate callbacks lose type info across page boundary

import {
  expect,
  test,
} from '@playwright/test';

declare global {
  // oxlint-disable-next-line typescript/consistent-type-imports -- typeof import() cannot use import type syntax
  var moduleLoggerBrowser: typeof import('@monochromatic-dev/logger-core/browser');
}

test.describe('IndexedDB sink', () => {
  test.beforeEach(async ({ page, },) => {
    await page.goto('/',);
    await page.waitForFunction(() => globalThis.moduleLoggerBrowser !== undefined);
  },);

  test('createIndexedDbSink exposes a callable verify', async ({ page, },) => {
    const typeofVerify = await page.evaluate(() => {
      const { createIndexedDbSink, } = globalThis.moduleLoggerBrowser;
      return typeof createIndexedDbSink().verify;
    },);
    expect(typeofVerify,).toBe('function',);
  });

  test('verify detects availability', async ({ page, },) => {
    const result = await page.evaluate(async () => {
      const { createIndexedDbSink, } = globalThis.moduleLoggerBrowser;
      return createIndexedDbSink().verify();
    },);
    expect(result,).toBe(true,);
  });

  test('a verified sink writes records across levels and message shapes', async ({ page, },) => {
    const allSucceeded = await page.evaluate(async () => {
      const { createIndexedDbSink, } = globalThis.moduleLoggerBrowser;
      const sink = createIndexedDbSink();
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
            console.warn('IndexedDB sink browser test write failed', error,);
            return false;
          }
        }
      }
      await sink.flush?.();
      return true;
    },);
    expect(allSucceeded,).toBe(true,);
  });

  test('a flushed batch is readable back out of the database as JSONL', async ({ page, },) => {
    const result = await page.evaluate(async () => {
      const { createIndexedDbSink, } = globalThis.moduleLoggerBrowser;
      const sink = createIndexedDbSink();
      await sink.verify();

      const testMessage = `unique-test-${Date.now()}`;
      await sink.write({
        level: 'info' as const,
        message: testMessage,
        timestamp: Date.now(),
      },);
      // Routine severity buffers; the flush hook forces the batch out and
      // resolves only after its transaction settles, so the read below is
      // deterministic.
      await sink.flush?.();

      const database: IDBDatabase = await new Promise((resolve, reject,) => {
        const request = globalThis.indexedDB.open('monochromatic.log', 1,);
        request.addEventListener('success', () => {
          resolve(request.result,);
        },);
        request.addEventListener('error', () => {
          reject(request.error ?? new Error('open failed',),);
        },);
      },);
      const batches: string[] = await new Promise((resolve, reject,) => {
        const request = database
          .transaction('batch', 'readonly',)
          .objectStore('batch',)
          .getAll();
        request.addEventListener('success', () => {
          resolve(request.result,);
        },);
        request.addEventListener('error', () => {
          reject(request.error ?? new Error('getAll failed',),);
        },);
      },);
      database.close();

      const holding = batches.find(batch => batch.includes(testMessage,),);
      if (holding === undefined)
        return { found: false, message: null, level: null, testMessage, };
      const lines = holding.split('\n',);
      const parsed = JSON.parse(lines.find(line => line.includes(testMessage,),) ?? 'null',);
      return {
        found: true,
        message: parsed.message,
        level: parsed.level,
        testMessage,
      };
    },);

    expect(result.found,).toBe(true,);
    expect(result.message,).toBe(result.testMessage,);
    expect(result.level,).toBe('info',);
  });

  test('retention trims the store back to the cap, oldest first', async ({ page, },) => {
    test.setTimeout(120_000,);
    const result = await page.evaluate(async () => {
      const { createIndexedDbSink, } = globalThis.moduleLoggerBrowser;
      const sink = createIndexedDbSink();
      await sink.verify();

      // Every warn record flushes its own batch, so this issues one batch
      // transaction per record and must cross the 2048-batch retention cap
      // regardless of what earlier tests left in the store.
      const BATCHES_PAST_CAP = 2_049;
      for (let index = 0; index < BATCHES_PAST_CAP; index++) {
        void sink.write({
          level: 'warn' as const,
          message: `retention-${index}`,
          timestamp: Date.now(),
        },);
      }
      await sink.flush?.();

      const database: IDBDatabase = await new Promise((resolve, reject,) => {
        const request = globalThis.indexedDB.open('monochromatic.log', 1,);
        request.addEventListener('success', () => {
          resolve(request.result,);
        },);
        request.addEventListener('error', () => {
          reject(request.error ?? new Error('open failed',),);
        },);
      },);
      const store = database
        .transaction('batch', 'readonly',)
        .objectStore('batch',);
      const count: number = await new Promise((resolve, reject,) => {
        const request = store.count();
        request.addEventListener('success', () => {
          resolve(request.result,);
        },);
        request.addEventListener('error', () => {
          reject(request.error ?? new Error('count failed',),);
        },);
      },);
      // The newest write must have survived the trim.
      const newestPresent: boolean = await new Promise((resolve, reject,) => {
        const request = store.getAll();
        request.addEventListener('success', () => {
          resolve(request.result.some((batch: string,) => batch.includes(`retention-${BATCHES_PAST_CAP - 1}`,),),);
        },);
        request.addEventListener('error', () => {
          reject(request.error ?? new Error('getAll failed',),);
        },);
      },);
      database.close();
      return { count, newestPresent, };
    },);

    expect(result.count,).toBe(2_048,);
    expect(result.newestPresent,).toBe(true,);
  });
});
