import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { createOpfsSink, } from '@monochromatic-dev/logger-core/browser';

// Node/Bun has no `navigator.storage`, so this file exercises the
// unavailable-backend fallback that the browser test (which runs where OPFS
// exists) never reaches: `getDirectory` throws and is caught, and drained
// batches hit the unset-stream guard. The available path lives in
// `opfs.browser.test.ts`; the shared buffering policy is covered in
// `record-buffer.unit.test.ts`. The factory is reached through the
// `./browser` subpath because the root entry no longer exports it.
await describe({
  name: 'OPFS sink (node fallback)',
  children: [
    it({
      name: 'verify resolves false when OPFS is absent',
      fn: async () => {
        // `navigator.storage.getDirectory()` dereferences an undefined member
        // and throws; verify catches it and reports the backend unavailable.
        const sink = createOpfsSink();
        expect(await sink.verify(),)
          .toBe(false,);
      },
    },),

    it({
      name: 'write buffers without throwing when OPFS is absent',
      fn: async () => {
        // The record buffers; nothing touches the missing backend until a
        // flush trigger fires.
        const sink = createOpfsSink();
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
      name: 'flush drains the buffer into the unset-stream guard and resolves',
      fn: async () => {
        // A warn record drains synchronously on add, an info record drains on
        // the flush hook; both batches hit the guard and are dropped silently.
        const sink = createOpfsSink();
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
         Resolved flush result; must settle even with no stream to write to.
         */
        const result = await sink.flush?.();
        expect(result,)
          .toBeUndefined();
      },
    },),
  ],
},);
