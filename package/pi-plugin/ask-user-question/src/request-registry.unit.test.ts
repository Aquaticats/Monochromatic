import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createRequestRegistry, } from '../dist/final/node/index.mjs';

await describe({
  name: createRequestRegistry.name,
  children: [
    it({
      name: 'aborts every active request',
      fn: async () => {
        /**
         Registry under test.
         */
        const registry = createRequestRegistry();
        /**
         First active request scope.
         */
        using first = registry.open();
        /**
         Second active request scope.
         */
        using second = registry.open();
        registry.abortAll();
        expect(first.signal.aborted,)
          .toBe(true,);
        expect(second.signal.aborted,)
          .toBe(true,);
      },
    },),
    it({
      name: 'manual abort affects selected request',
      fn: async () => {
        /**
         Registry under test.
         */
        const registry = createRequestRegistry();
        /**
         Manually cancelled request scope.
         */
        using request = registry.open();
        request.abort();
        expect(request.signal.aborted,)
          .toBe(true,);
      },
    },),
    it({
      name: 'disposed request leaves registry before global abort',
      fn: async () => {
        /**
         Registry under test.
         */
        const registry = createRequestRegistry();
        /**
         Disposed request retained for signal inspection.
         */
        const request = registry.open();
        request[Symbol.dispose]();
        registry.abortAll();
        expect(request.signal.aborted,)
          .toBe(false,);
      },
    },),
  ],
},);
