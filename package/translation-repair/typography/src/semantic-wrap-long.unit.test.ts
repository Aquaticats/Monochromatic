/**
 Tests for a passage longer than any buffer a child process is given by
 default.

 WHY THIS IS A FILE OF ITS OWN: the program that wraps a passage returns the
 wrapped text on a pipe, and a pipe read into a buffer of fixed size fails
 with a truncated or refused reply once the text outgrows it. Node's default
 for a synchronous child is one mebibyte, so the passage here is a little
 over that on the way in and a little over on the way out. The in-process wrap
 this one replaced took 564 seconds on 1.54 million characters of this shape
 (measured once, outside this suite), which is why no case here asks it for
 the same wrap beside the new one.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { wrapReplacementText, } from '../dist/final/node/index.mjs';

/**
 Sentences in the long passage; at fourteen characters each, over a mebibyte.
 */
const SENTENCES = 80_000;

await describe({
  name: wrapReplacementText.name,
  children: [
    it({
      name: 'WRAPS A PASSAGE OF OVER A MEBIBYTE, one sentence to a line and none lost, so the reply buffer '
        + 'is sized for the text and not for a typical page',
      fn: async () => {
        /**
         One run-on line of identical sentences, a space after each.
         */
        const passage = 'The cat naps. '.repeat(SENTENCES,);
        expect(passage.length,).toBeGreaterThan(1_048_576,);

        /**
         The wrapped passage.
         */
        const wrapped = wrapReplacementText({ text: passage, },);

        /**
         Its lines.
         */
        const lines = wrapped.split('\n',);
        expect(lines.length,).toBe(SENTENCES,);
        expect(lines.every(function isSentence(line,): boolean {
          return line.trimEnd() === 'The cat naps.';
        },),).toBe(true,);
      },
    },),
  ],
},);
