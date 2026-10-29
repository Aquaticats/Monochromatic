/**
 Tests for the reader of the JSON value an answer opens with, read through
 `parseAnswerJson`, which asks it last: an answer with nothing in it has no
 opening value, and an opening container that balances but is not JSON is
 refused as the whole answer is.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseAnswerJson, } from '../dist/final/node/index.mjs';

/**
 What the whole-answer parse says of any text it refuses.
 */
const REFUSED = {
  parsed: false,
  detail: 'refused by SyntaxError',
};

await describe({
  name: parseAnswerJson.name,
  children: [
    it({
      name: 'REFUSES AN ANSWER WITH NOTHING IN IT, whether empty or only whitespace, since it opens with no container',
      fn: async () => {
        expect([
          parseAnswerJson({ text: '', },),
          parseAnswerJson({ text: ' \n\t', },),
        ],).toEqual([
          REFUSED,
          REFUSED,
        ],);
      },
    },),
    it({
      name: 'REFUSES AN OPENING CONTAINER THAT BALANCES BUT IS NOT JSON as the whole answer is refused, rather than '
        + 'reading prose after it as a value',
      fn: async () => {
        expect(parseAnswerJson({ text: '{best: 2} That is my pick.', },),).toEqual(REFUSED,);
      },
    },),
  ],
},);
