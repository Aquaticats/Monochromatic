import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  sinks,
} from '@monochromatic-dev/logger-core';

/**
 Sink factories under test, read from the built artifact's `sinks` namespace.
 */
const {
  createNoopSink,
} = sinks;

await describe({
  name: 'noop sink',
  children: [
    it({
      name: 'verifies as available',
      fn: async () => {
        const sink = createNoopSink();
        expect(await sink.verify(),)
          .toBe(true,);
      },
    },),

    it({
      name: 'discards writes without throwing and exposes no flush hook',
      fn: async () => {
        const sink = createNoopSink();
        await expect(
          sink.write({
            level: 'info',
            message: 'discarded',
            timestamp: 0,
          },),
        )
          .resolves
          .toBeUndefined();
        expect(sink.flush,)
          .toBeUndefined();
      },
    },),
  ],
},);
