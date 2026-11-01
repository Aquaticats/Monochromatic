import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createLogger,
  sinks,
  type LogRecord,
} from '../dist/final/neutral/index.mjs';

/**
 Provides a fixed record for checking fallback console output.

 @param level - Severity to route.
 @returns Record with deterministic display timestamp.
 @example
 ```ts
 makeRecord({ level: 'info' });
 ```
 */
function makeRecord({ level, }: { readonly level: 'error' | 'fatal' | 'info' | 'warn'; },): LogRecord {
  return {
    level,
    message: 'QuickJS console fallback',
    timestamp: 0,
  };
}

await describe({
  name: 'QuickJS-compatible console and timerless logger',
  concurrency: 1,
  children: [
    it({
      name: 'console.log alone verifies and receives every unsuppressed severity',
      fn: async ({ sinon, },) => {
        const log = sinon.stub(console, 'log');
        sinon.stub(console, 'info').value(undefined);
        sinon.stub(console, 'warn').value(undefined);
        sinon.stub(console, 'error').value(undefined);
        sinon.stub(console, 'trace').value(undefined);
        sinon.stub(console, 'debug').value(undefined);
        const sink = sinks.createConsoleSink();
        expect(await sink.verify(),).toBe(true,);
        await Promise.all(
          (['info', 'warn', 'error', 'fatal',] as const).map(function writeLevel(level,) {
            return sink.write(makeRecord({ level, },),);
          },),
        );
        await sink.flush?.();
        expect(log.callCount,).toBe(4,);
        expect(log.getCalls().map(function firstArg(call,) {
          return String(call.args[0],);
        },),).toEqual([
          '[info] [1970-01-01T00:00:00.000Z] QuickJS console fallback',
          '[warn] [1970-01-01T00:00:00.000Z] QuickJS console fallback',
          '[error] [1970-01-01T00:00:00.000Z] QuickJS console fallback',
          '[fatal] [1970-01-01T00:00:00.000Z] QuickJS console fallback',
        ],);
      },
    },),
    it({
      name: 'logging and flushing work without a global timer',
      fn: async ({ sinon, },) => {
        const log = sinon.stub(console, 'log');
        sinon.stub(console, 'info').value(undefined);
        sinon.stub(console, 'debug').value(undefined);
        sinon.stub(globalThis, 'setTimeout').value(undefined);
        const { logger, initPromise, } = createLogger({
          sinks: [sinks.createConsoleSink(),],
        },);
        logger.info('timerless log',);
        await initPromise;
        await logger.flush();
        expect(log.callCount,).toBe(1,);
        expect(String(log.firstCall.args[0],),).toContain('timerless log',);
      },
    },),
    it({
      name: 'without console.log, a partial console does not verify',
      fn: async ({ sinon, },) => {
        sinon.stub(console, 'log').value(undefined);
        sinon.stub(console, 'warn').value(undefined);
        expect(await sinks.createConsoleSink().verify(),).toBe(false,);
      },
    },),
    it({
      name: 'without console.log, complete level methods still verify',
      fn: async ({ sinon, },) => {
        sinon.stub(console, 'log').value(undefined);
        expect(await sinks.createConsoleSink().verify(),).toBe(true,);
      },
    },),
    it({
      name: 'internal failures use console.log when console.warn is missing',
      fn: async ({ sinon, },) => {
        const log = sinon.stub(console, 'log');
        sinon.stub(console, 'warn').value(undefined);
        sinon.stub(console, 'error').value(undefined);
        const { initPromise, } = createLogger({
          sinks: [{
            verify: function failedVerify(): Promise<boolean> {
              throw new Error('probe failed',);
            },
            write: function unusedWrite(): Promise<void> {
              return Promise.resolve();
            },
          },],
        },);
        await initPromise;
        expect(log.callCount,).toBe(1,);
        expect(String(log.firstCall.args[0],),).toContain('probe failed',);
      },
    },),
    it({
      name: 'internal failures use console.error when warn is missing',
      fn: async ({ sinon, },) => {
        const error = sinon.stub(console, 'error');
        sinon.stub(console, 'warn').value(undefined);
        const { initPromise, } = createLogger({
          sinks: [{
            verify: function failedVerify(): Promise<boolean> {
              throw new Error('error channel probe failed',);
            },
            write: function unusedWrite(): Promise<void> {
              return Promise.resolve();
            },
          },],
        },);
        await initPromise;
        expect(error.callCount,).toBe(1,);
        expect(String(error.firstCall.args[0],),).toContain('error channel probe failed',);
      },
    },),
    it({
      name: 'absent console methods do not throw during internal reporting',
      fn: async ({ sinon, },) => {
        sinon.stub(console, 'log').value(undefined);
        sinon.stub(console, 'warn').value(undefined);
        sinon.stub(console, 'error').value(undefined);
        const { initPromise, logger, } = createLogger({
          sinks: [{
            verify: function failedVerify(): Promise<boolean> {
              throw new Error('no diagnostic channel',);
            },
            write: function unusedWrite(): Promise<void> {
              return Promise.resolve();
            },
          },],
        },);
        await initPromise;
        expect(function logWithoutBackend(): void {
          logger.info('no backend',);
        },).toThrow('No logging backends available',);
      },
    },),
  ],
},);
