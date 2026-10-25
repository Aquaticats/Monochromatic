/**
 Tests that a pair of runs holding a code span across a footnote definition line inside a block quote or a list item
 ships its original and holds no live construct in it.

 THE STRICT GRAMMAR STARTS THE FOOTNOTE DEFINITION AT THE LINE, behind the mark of a quote, at the content column of a
 list item, or after up to three spaces on a lazy line, and the span the plain reading keeps across the line ends
 there. The page then holds what follows the colon as text of the definition: a tag in it must be written as text so
 that the page parses (a run that does not parse is dropped), and a brace in it must be written as text so that it is
 no expression. The composition reads the pair as the sampled and the full checks do (`readPair`), which reports a
 page that ships without an original though no page was refused for its depth, and a construct found in a run written
 as literal text. The text of every shape is in `source-live-container-footnotes.test-fixture.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { containerFootnoteTexts, } from './source-live-container-footnotes.test-fixture.ts';
import { readPair, } from './source-live-pairs.test-fixture.ts';

/**
 A blank line and a block the strict grammar refuses, which follow a run and make it fail alone, so that the
 composition writes the run as literal text.
 */
const REFUSED_BLOCK = '\n<b>\n';

/**
 English between the two runs.
 */
const MIDDLE = 'She slept on the counter.\n';

/**
 A run that parses alone, beside the first.
 */
const SECOND_RUN = 'ok\n';

await describe({
  name: 'a code span across a footnote definition line inside a container, in a pair of runs',
  children: [
    ...containerFootnoteTexts({ body: '<b>', },).map(function tagCase([name, text,],) {
      return it({
        name: `WRITES ${name.toUpperCase()} AS LITERAL TEXT and keeps the original, so the page is not shipped without it`,
        fn: async () => {
          /**
           What the composition made of the pair.
           */
          const { step, finding, } = readPair({
            pair: {
              first: text,
              middle: MIDDLE,
              second: SECOND_RUN,
            },
          },);
          expect(finding.found,).toEqual([],);
          expect(step,).toBe('encoded-failing-runs',);
        },
      },);
    },),

    ...containerFootnoteTexts({ body: '{c}', },).map(function braceCase([name, text,],) {
      return it({
        name: `LEAVES NO LIVE EXPRESSION IN THE LITERAL TEXT WRITTEN FOR ${name.toUpperCase()}, beside a block the `
          + 'grammar refuses',
        fn: async () => {
          /**
           What the composition made of the pair.
           */
          const { step, finding, } = readPair({
            pair: {
              first: `${text}${REFUSED_BLOCK}`,
              middle: MIDDLE,
              second: SECOND_RUN,
            },
          },);
          expect(finding.found,).toEqual([],);
          expect(step,).toBe('encoded-failing-runs',);
        },
      },);
    },),
  ],
},);
