/**
 The full pass of the whole-page check of live constructs over every generated source text and every pair of runs of
 the step searches, which the sampled case (`source-live-sample.unit.test.ts`) reads one in four of.

 NOT PART OF `test:unit`: the file name carries `.expensive.`, which the task leaves out unless asked for it. Run it
 with `mise run //package/translation-repair/page-check:test:live-full`; the README of the package says when it must
 be run. It reads 80000 pairs and 32256 footnote texts in one process: 293 s to 402 s across the runs made so far,
 under the load of the other sessions of the host (no load figure was recorded for them), and a peak of 535 MB to
 559 MB of memory. It is the check that holds what
 a sample of the generated sets can miss: a class of a few pairs among the 80000, such as the one pair of the seed
 31415926 that a code span across a footnote line in a block quote once took to the last step of the composition, or
 the three pairs of each seed that a footnote that is a block of its own, then a definition line, once took there.

 WHAT IT GUARDS: no live construct in text written as literal text, and no drop that the grammar does not force (a run
 refused for its depth). WHAT IT DOES NOT SEE: a change that only escapes more than needed, which leaves the page
 parsing and free of live constructs while the text written differs (a tab no longer ending a footnote label, so a tag
 after it is written with backslashes; a join that starts at the line feed after a carriage return and keeps a stray
 one). Both are fail-safe, so a live-construct check cannot fail on them, and unit cases pin them instead
 (`source-ships-encoding.unit.test.ts`, `source-ships-lines.unit.test.ts`, `footnote-label.unit.test.ts` of the
 markup package).

 EVERY PAIR AND EVERY TEXT IS READ THE WAY THE SAMPLED CASE READS ONE (`source-live-pairs.test-fixture.ts`): the page is
 composed, the text written as literal text is found on it, and no expression, JSX element or ES module may stand in
 it; a page that ships without the original of a run is reported unless the grammar refuses that run, written as
 literal text, for its depth.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  type Finding,
  readPair,
  readText,
} from './source-live-pairs.test-fixture.ts';
import {
  footnoteProductTexts,
  sampledPairs,
} from './source-live-sample.test-fixture.ts';

/**
 The seeds of the two step searches, whose draws the sampled case also takes its pairs from.
 */
const PAIR_SEEDS: readonly number[] = [
  31_415_926,
  27_182_818,
];

/**
 How many pairs each seed's stream of draws holds, as many as the searches read.
 */
const PAIR_TOTAL = 40_000;

/**
 The least pairs of one stream whose page wrote an original as literal text, below which the pass proves nothing.
 */
const LEAST_LITERAL_PAGES = 15_000;

/**
 The share of the generated footnote texts the page grammar must read once written, since a text it refuses is not
 checked.
 */
const LEAST_READ_SHARE = 0.9;

await describe({
  name: 'live constructs over every generated source text and every pair of runs',
  children: [
    ...PAIR_SEEDS.map(function pairsOf(seed,) {
      return it({
        name: `SHIPS NO LIVE CONSTRUCT AND DROPS NO ORIGINAL BUT OF A RUN NESTING PAST THE BOUND, OVER ALL ${
          String(PAIR_TOTAL,)
        } PAIRS OF THE SEED ${String(seed,)}`,
        fn: async () => {
          /**
           Every pair of the stream.
           */
          const pairs = sampledPairs({
            seed,
            total: PAIR_TOTAL,
            sampleSeed: 1,
            oneIn: 1,
          },);
          expect(pairs.length,).toBe(PAIR_TOTAL,);

          /**
           What each pair's page wrote as literal text, and what was found in it.
           */
          const read = pairs.map(function readEach(pair,): ReturnType<typeof readPair> {
            return readPair({ pair, },);
          },);
          // A pass whose pages wrote nothing as literal text proves nothing.
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
      },);
    },),

    it({
      name: 'WRITES NO LIVE CONSTRUCT FOR ANY OF THE GENERATED FOOTNOTE TEXTS: each text the encoder writes parses '
        + 'after the note and holds no expression, JSX element or ES module',
      fn: async () => {
        /**
         Every generated text.
         */
        const texts = footnoteProductTexts();
        /**
         Texts the page grammar reads once written, and the constructs found in them.
         */
        const read = texts.map(function readEach(text,): ReturnType<typeof readText> {
          return readText({ text, },);
        },);
        expect(read.filter(function isRead({ parses, },): boolean {
          return parses;
        },).length > (texts.length * LEAST_READ_SHARE),).toBe(true,);
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
