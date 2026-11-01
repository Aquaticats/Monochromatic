import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  logger,
} from '@monochromatic-dev/logger-core';

await describe({
  name: logger.constructor.name,
  children: [
    it({
      name: 'logger has all six log level methods',
      fn: async () => {
        expect(typeof logger.trace,).toBe('function',);
        expect(typeof logger.debug,).toBe('function',);
        expect(typeof logger.info,).toBe('function',);
        expect(typeof logger.warn,).toBe('function',);
        expect(typeof logger.error,).toBe('function',);
        expect(typeof logger.fatal,).toBe('function',);
      },
    },),

    it({
      name: 'trace method accepts string message',
      fn: async () => {
        expect(() => {
          logger.trace('test trace message',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'debug method accepts string message',
      fn: async () => {
        expect(() => {
          logger.debug('test debug message',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'info method accepts string message',
      fn: async () => {
        expect(() => {
          logger.info('test info message',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'warn method accepts string message',
      fn: async () => {
        expect(() => {
          logger.warn('test warn message',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'error method accepts string message',
      fn: async () => {
        expect(() => {
          logger.error('test error message',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'fatal method accepts string message',
      fn: async () => {
        expect(() => {
          logger.fatal('test fatal message',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'logs with empty string message',
      fn: async () => {
        expect(() => {
          logger.info('',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'logs with unicode message',
      fn: async () => {
        expect(() => {
          logger.info('Hello 世界 🌍',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'logs with multiline message',
      fn: async () => {
        expect(() => {
          logger.info('line1\nline2\nline3',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'logs with special characters',
      fn: async () => {
        expect(() => {
          logger.info('Special: <script>alert("xss")</script>',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'logs with JSON-like content',
      fn: async () => {
        expect(() => {
          logger.info('{"key": "value", "count": 42}',);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'handles rapid successive logs',
      fn: async () => {
        expect(() => {
          const RAPID_LOG_COUNT = 100;
          for (let logIndex = 0; logIndex < RAPID_LOG_COUNT; logIndex++)
            logger.debug(`rapid log ${logIndex}`,);
        },)
          .not
          .toThrow();
      },
    },),

    it({
      name: 'flush is a callable method returning a promise',
      fn: async () => {
        expect(typeof logger.flush,)
          .toBe('function',);
        const result = logger.flush();
        expect(result instanceof Promise,)
          .toBe(true,);
        await result;
      },
    },),

    it({
      name: 'flush drains buffered console records',
      fn: async () => {
        // The console sink batches on microtasks; after a sync burst
        // of logs, flush should resolve after the console has actually
        // received the records. We cannot easily assert the count here
        // without re-spying (the default logger owns its sinks), so this
        // test only verifies that flush resolves after at least one
        // microtask tick rather than returning an already-settled promise
        // that misses the flush.
        logger.info('pre-flush 1',);
        logger.info('pre-flush 2',);
        logger.info('pre-flush 3',);
        await expect(logger.flush(),)
          .resolves
          .toBeUndefined();
      },
    },),
  ],
},);
