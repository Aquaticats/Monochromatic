/**
 Tests for the router's no-provider refusal and the mark that says it was raised after an exchange (B1230).

 The mark must not lose what the refusal it replaces carried: its stack and its cause are what a person reads when the
 refusal ends a run (B1248).

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  markedAfterExchange,
  NoProviderForModelError,
  refusedBeforeAnyExchange,
} from '../dist/final/node/index.mjs';

await describe({
  name: markedAfterExchange.name,
  concurrency: 1,
  children: [
    it({
      name: 'MARKS A REFUSAL AS RAISED AFTER AN EXCHANGE and keeps the refusal it replaces as the cause, so its stack '
        + 'and its own cause stay reachable (B1248)',
      fn: async () => {
        /**
         Refusal the router raised before the mark.
         */
        const refusal = new NoProviderForModelError({
          modelId: 'cat-model',
          reason: 'no provider serves this model',
        },);
        /**
         The same refusal after the mark.
         */
        const marked = markedAfterExchange({ error: refusal, },);
        expect(marked instanceof NoProviderForModelError ? marked.afterExchange : undefined,).toBe(true,);
        expect(Error.isError(marked,) ? marked.cause : undefined,).toBe(refusal,);
        expect(refusedBeforeAnyExchange({ error: refusal, },),).toBe(true,);
        expect(refusedBeforeAnyExchange({ error: marked, },),).toBe(false,);
      },
    },),

    it({
      name: 'LEAVES A MARKED REFUSAL AND ANY OTHER ERROR AS THEY ARE, the control',
      fn: async () => {
        /**
         Refusal already marked.
         */
        const marked = new NoProviderForModelError({
          modelId: 'cat-model',
          reason: 'no provider serves this model',
          afterExchange: true,
        },);
        /**
         Error that is no refusal.
         */
        const other = new RangeError('out of range',);
        expect(markedAfterExchange({ error: marked, },),).toBe(marked,);
        expect(markedAfterExchange({ error: other, },),).toBe(other,);
      },
    },),
  ],
},);
