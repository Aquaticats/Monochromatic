import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { createIndexedDbSink, } from '@monochromatic-dev/logger-core/browser';

// Node, Deno, and Bun expose no `indexedDB` (probed on Node 26, Deno 2.9,
// Bun 1.3), so this file exercises the unavailable-backend fallback that the
// browser test (which runs where IndexedDB exists) never reaches: `verify`
// short-circuits on the missing global, and drained batches hit the
// unset-connection guard. The available path lives in
// `indexed-db.browser.test.ts`; the shared buffering policy is covered in
// `record-buffer.unit.test.ts`. The factory is reached through the
// `./browser` subpath because the root entry no longer exports it.
await describe({
  name: 'IndexedDB sink (node fallback)',
  children: [
    it({
      name: 'verify resolves false when IndexedDB is absent',
      fn: async () => {
        const sink = createIndexedDbSink();
        expect(await sink.verify(),)
          .toBe(false,);
      },
    },),

    it({
      name: 'write buffers without throwing when IndexedDB is absent',
      fn: async () => {
        // The record buffers; nothing touches the missing backend until a
        // flush trigger fires.
        const sink = createIndexedDbSink();
        /**
         Resolved write result; the sink write contract is `Promise<void>`.
         */
        const result = await sink.write({
          level: 'info',
          message: 'dropped',
          timestamp: 0,
        },);
        expect(result,)
          .toBeUndefined();
      },
    },),

    it({
      name: 'flush drains the buffer into the unset-connection guard and resolves',
      fn: async () => {
        // A warn record drains synchronously on add, an info record drains on
        // the flush hook; both batches hit the guard and are dropped silently.
        const sink = createIndexedDbSink();
        await sink.write({
          level: 'warn',
          message: 'urgent but backendless',
          timestamp: 0,
        },);
        await sink.write({
          level: 'info',
          message: 'buffered but backendless',
          timestamp: 1,
        },);
        /**
         Resolved flush result; must settle even with no connection to write to.
         */
        const result = await sink.flush?.();
        expect(result,)
          .toBeUndefined();
      },
    },),
  ],
},);
