/**
 Tests for how the pair fixture judges a page that ships without the original of a run: the grammar refuses no page
 the composition could write without it for any reason but its depth, and a run dropped for depth does not excuse a
 run dropped for another reason beside it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type PageGrammarReading,
  readPageGrammar,
} from '../../dist/final/node/index.mjs';
import { readPair, } from './source-live-pairs.test-fixture.ts';

/**
 A run the grammar refuses for its depth however it is written.
 */
const DEEP_RUN = `${'> '.repeat(300,)}x\n`;

/**
 A run the stand-in grammar refuses for a reason that is not depth, even written as literal text.
 */
const FORCED_RUN = 'The cat purred.\n';

/**
 The strict grammar, except that it refuses any text holding the forced run, with a refusal that does not name depth.

 @param pageText - text read

 @returns What the grammar says of it

 @example
 ```ts
 const reading = refusingForcedRun({ pageText: 'The cat purred.\n', },); // refused
 ```
 */
function refusingForcedRun({ pageText, }: { readonly pageText: string; },): PageGrammarReading {
  return pageText.includes(FORCED_RUN.trim(),)
    ? {
      kind: 'refused',
      refusal: 'at 1:1 (forced by the case)',
    }
    : readPageGrammar({ pageText, },);
}

await describe({
  name: readPair.name,
  children: [
    it({
      name: 'EXCUSES THE DROP OF A RUN THE GRAMMAR REFUSES FOR ITS DEPTH',
      fn: async () => {
        const { finding, step, } = readPair({
          pair: {
            first: DEEP_RUN,
            middle: 'She slept on the counter.\n',
            second: 'ok\n',
          },
        },);
        expect(step,).toBe('dropped',);
        expect(finding.found,).toEqual([],);
      },
    },),

    it({
      name: 'NOTES THE DROP OF A RUN REFUSED FOR ANOTHER REASON THAT STANDS BESIDE ONE REFUSED FOR ITS DEPTH, which '
        + 'the refusal for depth does not excuse',
      fn: async () => {
        const { finding, step, } = readPair({
          pair: {
            first: DEEP_RUN,
            middle: 'She slept on the counter.\n',
            second: FORCED_RUN,
          },
          readGrammar: refusingForcedRun,
        },);
        expect(step,).toBe('dropped',);
        expect(finding.found.length,).toBe(1,);
        expect(finding.found[0],).toContain('slices 3 ',);
      },
    },),
  ],
},);
