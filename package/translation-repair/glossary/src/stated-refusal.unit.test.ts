/**
 Tests the refusal a command states in its own words: what it says, the
 marker that lets a boundary repeat it, and the failure it may carry.

 Fixtures are invented.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '../dist/final/node/index.mjs';

await describe({
  name: StatedRefusalError.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'SAYS its message whole under its own name, marked as safe to forward, and carries no cause where '
        + 'none is handed',
      fn: async () => {
        /** A usage line, the shape the class exists for. */
        const refusal = new StatedRefusalError({ says: 'name one cat: feed-cat <name>', },);
        expect({
          text: String(refusal,),
          marked: refusal.messageNamesOnly,
          carriesCause: 'cause' in refusal,
        },)
          .toEqual({
            text: 'StatedRefusalError: name one cat: feed-cat <name>',
            marked: true,
            carriesCause: false,
          },);
      },
    },),

    it({
      name: 'KEEPS the failure it is handed as its cause, the very object, and repeats none of that failure\'s '
        + 'words in what it says',
      fn: async () => {
        /** A failure whose own words a boundary must not print. */
        const failure = new Error('the bowl at /tmp/purr is cracked',);
        /** The refusal answering that failure in authored words. */
        const refusal = new StatedRefusalError({
          says: 'the bowl could not be read',
          cause: failure,
        },);
        expect(refusal.cause,).toBe(failure,);
        expect(String(refusal,),).toBe('StatedRefusalError: the bowl could not be read',);
      },
    },),
  ],
},);
