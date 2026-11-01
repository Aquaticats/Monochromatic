/**
 Tests for the reason a round aborts its stragglers with.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StragglerCutError, } from '../dist/final/node/index.mjs';

await describe({
  name: StragglerCutError.name,
  children: [
    it({
      name: 'IS AN ERROR THAT NAMES ITSELF, and is what an aborted signal reports as its reason, so the seat tally can '
        + 'tell the round\'s grace window from a caller leaving',
      fn: async () => {
        /**
         Controller aborted the way the round aborts its stragglers.
         */
        const controller = new AbortController();
        controller.abort(new StragglerCutError(),);
        expect(controller.signal.reason,).toBeInstanceOf(StragglerCutError,);
        expect((controller.signal.reason as StragglerCutError).name,).toBe('StragglerCutError',);
        expect((controller.signal.reason as StragglerCutError).messageNamesOnly,).toBe(true,);
      },
    },),

    it({
      name: 'IS NOT WHAT A PLAIN ABORT CARRIES, which is the bare AbortError a caller\'s stop leaves',
      fn: async () => {
        /**
         Controller aborted with no reason.
         */
        const controller = new AbortController();
        controller.abort();
        expect(controller.signal.reason,).not.toBeInstanceOf(StragglerCutError,);
      },
    },),
  ],
},);
