/**
 Guards issue #493: evaluating the root entry in a runtime that forbids
 timers in global scope (Cloudflare Workers throw from `setTimeout` there)
 must produce no `logger internal error` output, because the default logger
 is built on first use, not at import. Once a handler runs, the first log
 call builds it and the console sink verifies normally.

 The built artifact is imported dynamically inside the test so the throwing
 `setTimeout` is in place during module evaluation; this file therefore
 imports nothing from the logger statically.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Message the first log call inside the "handler" sends, distinct enough to
 find in the console stub's calls.
 */
const HANDLER_MESSAGE = 'first log inside a handler after a restricted import';

await describe({
  name: 'default logger under a global-scope-restricted runtime',
  // Both tests stub console methods, so they run one at a time.
  concurrency: 1,
  children: [
    it({
      name: 'importing the root entry while setTimeout throws writes no breadcrumb, and the first log inside a handler works',
      fn: async ({ sinon, },) => {
        const warn = sinon.stub(
          console,
          'warn',
        );
        const info = sinon.stub(
          console,
          'info',
        );
        /**
         Timer stub that behaves like a Workers global scope: any timer is a
         disallowed operation.
         */
        const forbidTimers = sinon.stub(
          globalThis,
          'setTimeout',
        )
          .throws(new Error('Disallowed operation called within global scope',),);
        /**
         Root entry evaluated with timers forbidden, as a Worker isolate does.
         */
        const entry = await import('@monochromatic-dev/logger-core');
        /**
         `tagged` reaches the singleton through its default parameter; wrapping
         must not build it either.
         */
        const l = entry.tagged({ tag: 'restricted', },);
        expect(warn.callCount,)
          .toBe(0,);
        forbidTimers.restore();

        // Inside a handler, timers are allowed again: the first log builds the
        // default logger, its sinks verify, and the console sink writes.
        l.info(HANDLER_MESSAGE,);
        await entry.logger.flush();
        expect(warn.callCount,)
          .toBe(0,);
        /**
         Console lines that carried the handler message.
         */
        const landed = info.getCalls()
          .filter(function carriesMessage(call,) {
            return call.args
              .some(function mentions(argument,) {
                return String(argument,)
                  .includes(HANDLER_MESSAGE,);
              },);
          },);
        expect(landed.length,)
          .toBe(1,);
      },
    },),

    it({
      name: 'flush before any log still builds the default logger and resolves',
      fn: async ({ sinon, },) => {
        const warn = sinon.stub(
          console,
          'warn',
        );
        const entry = await import('@monochromatic-dev/logger-core');
        await entry.logger.flush();
        expect(warn.callCount,)
          .toBe(0,);
      },
    },),
  ],
},);
