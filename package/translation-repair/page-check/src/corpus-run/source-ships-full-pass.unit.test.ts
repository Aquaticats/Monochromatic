/**
 Tests that the pairs the full pass found, and the shapes they stand for, ship no live construct in a run written as
 literal text and drop no original.

 THE STRICT GRAMMAR HAS NO RAW HTML, so a tag alone on a line is a line of the paragraph above it on the page, where the
 plain reading ends the footnote's paragraph at the tag. A footnote line in a block quote ends the quote at the first
 lazy line that the strict grammar reads as a fence, which the plain reading takes for text of the paragraph, and what
 stands after the footnote at the top is read as it stands. A footnote that reaches the top of the text puts a blank
 line before a definition that follows it directly, in a container too. The pairs in
 `source-live-full-pass-pairs.test-fixture.ts` are the ones the pass over all the draws found; the cases of this file
 name each shape on its own.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { FULL_PASS_PAIRS, } from './source-live-full-pass-pairs.test-fixture.ts';
import { readPair, } from './source-live-pairs.test-fixture.ts';
import type { RunPair, } from './source-live-sample.test-fixture.ts';

/**
 English between the two runs.
 */
const MIDDLE = 'She slept on the counter.\n';

/**
 A run that parses alone, beside the first.
 */
const SECOND_RUN = 'ok\n';

/**
 A block the strict grammar refuses, which follows a run and makes it fail alone, so that the composition writes the run
 as literal text.
 */
const REFUSED_BLOCK = '\n<b>\n';

/**
 The shapes, each as a pair with a run that fails alone, so that the composition writes it as literal text.
 */
const SHAPE_PAIRS: readonly (readonly [string, RunPair])[] = [
  [
    'A TAG ALONE AND A CODE SPAN WITH A BRACE ON THE LINES AFTER A FOOTNOTE LINE IN A BLOCK QUOTE',
    {
      first: '> a `x\n>  [^1]: <b>` {d}\n<q>\n`{`\n',
      middle: MIDDLE,
      second: SECOND_RUN,
    },
  ],
  [
    'A LAZY FOOTNOTE LINE IN A BLOCK QUOTE, CODE INDENTED UNDER IT AND A DEFINITION WITH A BRACE DIRECTLY AFTER',
    {
      first: `> a\n[^1]: x\n\n        c\n[r]: u{b}\n${REFUSED_BLOCK}`,
      middle: MIDDLE,
      second: SECOND_RUN,
    },
  ],
  [
    'A TAG ALONE IN A FOOTNOTE LINE OF A BLOCK QUOTE, THEN A FOOTNOTE WITH INDENTED CODE AND A DEFINITION WITH A BRACE',
    {
      first: '> [^1]: x\n> <a href="x">\n[^1]: x\n\n        c\n[r]: u{b}\n',
      middle: MIDDLE,
      second: SECOND_RUN,
    },
  ],
  [
    'A LAZY LINE HOLDING A FENCE MARK INDENTED FOUR SPACES AFTER A FOOTNOTE LINE IN A BLOCK QUOTE, BESIDE A FENCE',
    {
      first: '> a `x\n>  [^1]: <b>` {d}\nabc\n    ```\n',
      middle: MIDDLE,
      second: '<b>\n\n```\n{a}\n```\n',
    },
  ],
  [
    'A LAZY LINE OF CODE AFTER A FENCE MARK INDENTED FOUR SPACES IN A QUOTE, LINES ENDING IN A LONE CARRIAGE RETURN',
    {
      first: `> a \`x\n>  [^1]: {c}\` {d}\r> a\n>     \`\`\`\r    {y}\n${REFUSED_BLOCK}`,
      middle: MIDDLE,
      second: SECOND_RUN,
    },
  ],
];

await describe({
  name: 'the shapes the full pass found, in a pair of runs',
  children: [
    ...SHAPE_PAIRS.map(function shapeCase([name, pair,],) {
      return it({
        name: `SHIPS NO LIVE CONSTRUCT AND KEEPS EVERY ORIGINAL IN THE SHAPE: ${name}`,
        fn: async () => {
          /**
           What the composition made of the pair.
           */
          const { finding, } = readPair({ pair, },);
          expect(finding.found,).toEqual([],);
        },
      },);
    },),

    ...FULL_PASS_PAIRS.map(function foundCase([name, pair,],) {
      return it({
        name: `SHIPS NO LIVE CONSTRUCT AND KEEPS EVERY ORIGINAL FOR THE PAIR THE FULL PASS FOUND: ${name}`,
        fn: async () => {
          /**
           What the composition made of the pair.
           */
          const { finding, } = readPair({ pair, },);
          expect(finding.found,).toEqual([],);
        },
      },);
    },),
  ],
},);
