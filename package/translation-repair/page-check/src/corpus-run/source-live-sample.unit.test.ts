/**
 Tests that no page shipped with the original of an untranslated passage holds code the original lacks, over seeded
 samples of source texts.

 THE PAGE IS COMPILED AS MDX AND ITS EXPRESSIONS, JSX ELEMENTS AND ES MODULES RUN. The original is read as Markdown with
 GFM and the site's math, which builds none of them, so every one of those the strict grammar builds in the text of an
 original the encoder wrote as literal text is code the original lacks (`source-live-constructs.test-fixture.ts`). The
 cases of `source-ships-encoding.unit.test.ts` and `source-shipped-page.unit.test.ts` read the shapes found one by one;
 these read a sample of the generated sets for the shape nobody has named. A seeded search of forty thousand pairs of
 runs found five pages with such code before the fix that closes a fence or display math a text leaves open and ends a
 footnote before a definition line: four at index 4676, 17099, 18547 and 27936 of the draws of the seed 31415926, one at
 index 35447 of the draws of the seed 27182818. The pairs here are a fixed fraction of the draws of the first seed and
 hold the pairs at index 4676, 17099 and 27936, so the check fails without that fix.

 SCOPE: the slices the composition wrote as literal text. A slice written as recorded is the text the strict grammar
 accepts as it stands, and the site reads it as the corpus page it came from.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */


import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { readPageGrammar, } from '../../dist/final/node/index.mjs';
import {
  type Finding,
  readPair,
  readText,
} from './source-live-pairs.test-fixture.ts';
import {
  footnoteProductTexts,
  sampledPairs,
  sampledTexts,
} from './source-live-sample.test-fixture.ts';

/**
 Seed of the draws of the pairs, the first of the two seeds of the search.
 */
const PAIR_SEED = 31_415_926;

/**
 How many pairs the stream of draws holds, as many as the search read.
 */
const PAIR_TOTAL = 40_000;

/**
 The least the sample keeps of the generated footnote texts, below which it proves nothing.
 */
const LEAST_TEXTS = 1_000;

/**
 The least the sample keeps of the pairs of runs, below which it proves nothing.
 */
const LEAST_PAIRS = 2_000;

/**
 The least pairs of the sample whose page wrote an original as literal text, below which it proves nothing.
 */
const LEAST_LITERAL_PAGES = 1_000;

/**
 The share of the sampled footnote texts the page grammar must read once written, since a text it refuses is not
 checked.
 */
const LEAST_READ_SHARE = 0.9;

/**
 Seed of the stream that picks the pairs to keep, one that keeps three of the four draws that held code before the fix.
 */
const PAIR_PICK_SEED = 23;

/**
 The pairs kept are one in this many, on average.
 */
const PAIR_ONE_IN = 18;

/**
 Seed of the stream that picks the generated footnote texts to keep.
 */
const TEXT_PICK_SEED = 1;

/**
 The generated footnote texts kept are one in this many, on average.
 */
const TEXT_ONE_IN = 8;

await describe({
  name: 'live constructs over seeded samples of source texts',
  children: [
    it({
      name: 'WRITES NO LIVE CONSTRUCT FOR A SEEDED SAMPLE OF THE GENERATED FOOTNOTE TEXTS: each text the encoder writes '
        + 'parses after the note and holds no expression, JSX element or ES module',
      fn: async () => {
        /**
         The texts kept of the 10800 generated.
         */
        const sample = sampledTexts({
          texts: footnoteProductTexts(),
          sampleSeed: TEXT_PICK_SEED,
          oneIn: TEXT_ONE_IN,
        },);
        // A sample that kept next to nothing proves nothing.
        expect(sample.length > LEAST_TEXTS,).toBe(true,);

        /**
         Texts the page grammar reads once written, and the constructs found in them.
         */
        const read = sample.map(function readSampled(text,): ReturnType<typeof readText> {
          return readText({ text, },);
        },);
        expect(read.filter(function isRead({ parses, },): boolean {
          return parses;
        },).length > (sample.length * LEAST_READ_SHARE),).toBe(true,);
        expect(read
          .map(function toFinding({ finding, },): Finding {
            return finding;
          },)
          .filter(function holdsCode({ found, },): boolean {
            return found.length > 0;
          },),).toEqual([],);
      },
    },),

    it({
      name: 'SHIPS NO LIVE CONSTRUCT IN THE TEXT OF A SLICE WRITTEN AS LITERAL TEXT FOR A SEEDED SAMPLE OF PAIRS OF RUNS: '
        + 'the page the composition makes holds no expression, JSX element or ES module where it wrote an original '
        + 'as literal text',
      fn: async () => {
        /**
         The pairs kept of the draws, three of which held code before the fix.
         */
        const pairs = sampledPairs({
          seed: PAIR_SEED,
          total: PAIR_TOTAL,
          sampleSeed: PAIR_PICK_SEED,
          oneIn: PAIR_ONE_IN,
        },);
        expect(pairs.length > LEAST_PAIRS,).toBe(true,);

        /**
         Pairs whose page wrote an original as literal text, with the constructs found in that text.
         */
        const read = pairs.map(function readSampled(pair,): ReturnType<typeof readPair> {
          return readPair({ pair, },);
        },);
        // A sample whose pages wrote nothing as literal text proves nothing.
        expect(read.filter(function wroteLiteral({ encoded, },): boolean {
          return encoded > 0;
        },).length > LEAST_LITERAL_PAGES,).toBe(true,);
        expect(read
          .map(function toFinding({ finding, },): Finding {
            return finding;
          },)
          .filter(function holdsCode({ found, },): boolean {
            return found.length > 0;
          },),).toEqual([],);
      },
    },),
  ],
},);
