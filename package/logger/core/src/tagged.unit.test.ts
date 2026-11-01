import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  tagged,
  type Level,
  type Logger,
} from '@monochromatic-dev/logger-core';

/**
 Every log level paired with the message body the all-levels test sends
 through each, so the assertion can be built from one source of truth.
 */
const LEVEL_MESSAGES: readonly { readonly level: Level; readonly message: string; }[] = [
  {
    level: 'trace',
    message: 'a',
  },
  {
    level: 'debug',
    message: 'b',
  },
  {
    level: 'info',
    message: 'c',
  },
  {
    level: 'warn',
    message: 'd',
  },
  {
    level: 'error',
    message: 'e',
  },
  {
    level: 'fatal',
    message: 'f',
  },
];

/**
 Builds a stub logger whose methods record the messages they receive
 and whose `flush` is a trivial resolved promise. Used to verify the
 tagged wrapper's message prefixing and `flush` forwarding without
 touching the default multi-sink singleton.
 
 @returns Object containing the stub logger and the recorded message list.
 */
function createStubLogger(): {
  l: Logger;
  messages: { level: string; message: string; }[];
  flushCalls: number;
} {
  const messages: { level: string; message: string; }[] = [];
  const counters: { flushCalls: number; } = { flushCalls: 0, };
  const l: Logger = {
    debug: function debug(message: string,): void {
      messages.push({ level: 'debug', message, },);
    },
    error: function error(message: string,): void {
      messages.push({ level: 'error', message, },);
    },
    fatal: function fatal(message: string,): void {
      messages.push({ level: 'fatal', message, },);
    },
    flush: function flush(): Promise<void> {
      counters.flushCalls++;
      return Promise.resolve();
    },
    info: function info(message: string,): void {
      messages.push({ level: 'info', message, },);
    },
    trace: function trace(message: string,): void {
      messages.push({ level: 'trace', message, },);
    },
    warn: function warn(message: string,): void {
      messages.push({ level: 'warn', message, },);
    },
  };
  return {
    get flushCalls() {
      return counters.flushCalls;
    },
    l,
    messages,
  };
}

await describe({
  name: 'tagged logger wrapper',
  children: [
    it({
      name: 'prepends [tag] to each level method',
      fn: async () => {
        const stub = createStubLogger();
        const wrapped = tagged({
          tag: 'scope',
          l: stub.l,
        },);

        wrapped.info('hello',);
        wrapped.warn('careful',);

        expect(stub.messages,)
          .toEqual([
            { level: 'info', message: '[scope] hello', },
            { level: 'warn', message: '[scope] careful', },
          ],);
      },
    },),

    it({
      name: 'flush delegates to the inner logger without tagging',
      fn: async () => {
        const stub = createStubLogger();
        const wrapped = tagged({
          tag: 'scope',
          l: stub.l,
        },);

        expect(stub.flushCalls,)
          .toBe(0,);
        await wrapped.flush();
        expect(stub.flushCalls,)
          .toBe(1,);
        // flush must not leak into the message stream
        expect(stub.messages.length,)
          .toBe(0,);
      },
    },),

    it({
      name: 'prepends [tag] across all six level methods',
      fn: async () => {
        const stub = createStubLogger();
        const wrapped = tagged({
          tag: 'svc',
          l: stub.l,
        },);

        LEVEL_MESSAGES.forEach(function logAtLevel({
          level,
          message,
        },) {
          wrapped[level](message,);
        },);

        expect(stub.messages,)
          .toEqual(LEVEL_MESSAGES.map(function toTagged({
            level,
            message,
          },) {
            return {
              level,
              message: `[svc] ${message}`,
            };
          },),);
      },
    },),

    it({
      name: 'composes nested tags root-first',
      fn: async () => {
        const stub = createStubLogger();
        // Inner wrap hits the underlying logger first (leftmost tag); outer
        // wrap prepends last (rightmost tag), so the stub sees them root-first.
        const inner = tagged({
          tag: 'http',
          l: stub.l,
        },);
        const outer = tagged({
          tag: 'retry',
          l: inner,
        },);

        outer.info('attempt 3',);

        expect(stub.messages,)
          .toEqual([
            {
              level: 'info',
              message: '[http] [retry] attempt 3',
            },
          ],);
      },
    },),

    it({
      name: 'wraps the default singleton logger when no parent is given',
      fn: async () => {
        // Exercises the `l = defaultLogger` parameter default: a tag with no
        // explicit parent returns a usable wrapper whose calls reach the
        // singleton without throwing and whose flush delegates to it.
        const wrapped = tagged({ tag: 'standalone', },);

        expect(typeof wrapped.info,)
          .toBe('function',);
        expect(function logThroughDefault() {
          wrapped.info('via default logger',);
        },)
          .not
          .toThrow();
        await expect(wrapped.flush(),)
          .resolves
          .toBeUndefined();
      },
    },),
  ],
},);
