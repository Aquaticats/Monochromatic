/**
 Tests that no page shipped with the original of an untranslated passage holds code the original lacks, and loses an
 original only where nothing can be done for it, over seeded samples of source texts.

 THE PAGE IS COMPILED AS MDX AND ITS EXPRESSIONS, JSX ELEMENTS AND ES MODULES RUN. The original is read as Markdown with
 GFM and the site's math, which builds none of them, so every one of those the strict grammar builds in the text of an
 original the encoder wrote as literal text is code the original lacks (`source-live-constructs.test-fixture.ts`). A
 page that ships without the original of a run is reported too, unless the grammar refuses that run, written as literal
 text, for its depth: that is the one thing no writing of the text can cure, so a pair that takes the last step of the
 composition for any other reason has met a shape the encoder does not make parse. The cases of
 `source-ships-encoding.unit.test.ts` and `source-shipped-page.unit.test.ts` read the shapes found one by one; these
 read a sample of the generated sets for the shape nobody has named.

 THE SAMPLE IS ONE IN FOUR OF EACH OF THE TWO SEEDS, AND WHY. A class of violators the pool reaches and the cases do not
 name is found when the sample keeps one of them. The two seeds are the two step searches' (`PAIR_SEEDS`), each a stream
 of 40000 draws, and a case reads one in four of each, 9929 pairs of each at the picking seed this file uses. Chances
 were measured by simulation against those kept sets (20000 random sets of violators among the 80000 draws, a
 generator seeded apart from the draws; two runs of the sets gave the figures within 0.004): a class of nine pairs is
 caught with chance 0.92 and one of seventeen with chance 0.99, where the case read the first seed alone it caught nine
 with chance 0.70 and seventeen with chance 0.89, the second seed being unread. A smaller class is the full pass over
 every pair and text (`source-live-full.expensive.unit.test.ts`, the `test:live-full` task), which is not part of
 `test:unit`; the README of the package says when it must be run. The file takes 119 s in the one measurement made of it
 after this change, under a host load average of 36 (a case that read the first seed alone took 40 s to 51 s), against
 293 s to 402 s for the full task. The pool holds lone carriage returns for line endings, footnote labels other than
 `1`, tilde fences at the first column, a footnote in a list item and in a block quote, a definition line after a
 footnote that is a tag alone, and a code span across a footnote line in a quote or a list item, beside the shapes
 the earlier searches found.

 SCOPE: the slices the composition wrote as literal text. A slice written as recorded is the text the strict grammar
 accepts as it stands, and the site reads it as the corpus page it came from.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */


import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

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
 The seeds of the two step searches, whose streams of draws the case reads one in four of each.
 */
const PAIR_SEEDS: readonly number[] = [
  31_415_926,
  27_182_818,
];

/**
 How many pairs the stream of draws holds, as many as the search read.
 */
const PAIR_TOTAL = 40_000;

/**
 The least the sample keeps of the generated footnote texts, below which it proves nothing.
 */
const LEAST_TEXTS = 6_000;

/**
 The least the sample keeps of the pairs of runs of one seed, below which it proves nothing.
 */
const LEAST_PAIRS = 8_000;

/**
 The least pairs of one seed's sample whose page wrote an original as literal text, below which it proves nothing.
 */
const LEAST_LITERAL_PAGES = 4_000;

/**
 The least pairs of one seed's sample holding a lone carriage return, below which the sample does not reach that class.
 */
const LEAST_CARRIAGE_RETURN_PAIRS = 1_000;

/**
 The share of the sampled footnote texts the page grammar must read once written, since a text it refuses is not
 checked.
 */
const LEAST_READ_SHARE = 0.9;

/**
 Seed of the stream that picks the pairs to keep. Any seed serves: the guarantee is the rate, not the draws it keeps.
 */
const PAIR_PICK_SEED = 23;

/**
 Whether a text holds a carriage return that no line feed follows, found by a scan of the guard's own and not by the code
 under test, so the guard cannot agree with a fault in it.

 @param text - text read

 @returns Whether it holds one

 @example
 ```ts
 const lone = holdsLoneCarriageReturn({ text: 'a\rb', },); // true
 ```
 */
function holdsLoneCarriageReturn({ text, }: { readonly text: string; },): boolean {
  return text
    .split('',)
    .some(function isLone(character, at,): boolean {
      return (character === '\r') && (text.charAt(at + 1,) !== '\n');
    },);
}

/**
 The pairs kept of each seed are one in this many, on average: a class of nine pairs among the 80000 draws of both seeds
 is caught with chance 0.92.
 */
const PAIR_ONE_IN = 4;

/**
 Seed of the stream that picks the generated footnote texts to keep.
 */
const TEXT_PICK_SEED = 1;

/**
 The generated footnote texts kept are one in this many, on average.
 */
const TEXT_ONE_IN = 4;

await describe({
  name: 'live constructs over seeded samples of source texts',
  children: [
    it({
      name: 'WRITES NO LIVE CONSTRUCT FOR A SEEDED SAMPLE OF THE GENERATED FOOTNOTE TEXTS: each text the encoder '
        + 'writes parses after the note and holds no expression, JSX element or ES module',
      fn: async () => {
        /**
         The texts kept of the 32256 generated.
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

    ...PAIR_SEEDS.map(function pairsOf(seed,) {
      return it({
        name: `SHIPS NO LIVE CONSTRUCT IN THE TEXT OF A SLICE WRITTEN AS LITERAL TEXT FOR A SEEDED SAMPLE OF PAIRS OF `
          + `RUNS OF THE SEED ${String(seed,)}, AND DROPS NO ORIGINAL BUT FOR A NESTING PAST THE BOUND: the page the `
          + 'composition makes holds no expression, JSX element or ES module where it wrote an original as literal '
          + 'text, and ships without the original of a run only where the grammar refuses that run for its depth',
        fn: async () => {
          /**
           The pairs kept of the draws.
           */
          const pairs = sampledPairs({
            seed,
            total: PAIR_TOTAL,
            sampleSeed: PAIR_PICK_SEED,
            oneIn: PAIR_ONE_IN,
          },);
          expect(pairs.length > LEAST_PAIRS,).toBe(true,);
          // A sample that holds next to no lone carriage return does not reach that class. The scan is the guard's own,
          // which the code under test is not asked.
          expect(pairs.filter(function holdsLoneReturn({
            first,
            second,
          },): boolean {
            return holdsLoneCarriageReturn({ text: `${first}${second}`, },);
          },).length > LEAST_CARRIAGE_RETURN_PAIRS,).toBe(true,);

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
      },);
    },),
  ],
},);
