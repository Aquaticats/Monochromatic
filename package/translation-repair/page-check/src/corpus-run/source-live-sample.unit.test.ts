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
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  composeSourcePage,
  type ComposedPage,
  encodeAsLiteralMdx,
  holdsOpenComment,
  readPageGrammar,
  runsUnfitForCompany,
  runTextOf,
  sourceRunsOf,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';
import {
  liveConstructsOf,
  liveConstructsOfCarried,
} from './source-live-constructs.test-fixture.ts';
import {
  footnoteProductTexts,
  type RunPair,
  sampledPairs,
  sampledTexts,
} from './source-live-sample.test-fixture.ts';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

/**
 Page text standing before the encoded text in the parse cases, the way the note stands before an original.
 */
const BEFORE = '## Description\n\n> This passage has not been translated yet. The original text follows.\n\n';

/**
 Seed of the draws of the pairs, the first of the two seeds of the search.
 */
const PAIR_SEED = 31_415_926;

/**
 How many pairs the stream of draws holds, as many as the search read.
 */
const PAIR_TOTAL = 40_000;

/**
 What `indexOf` answers for a text it does not find.
 */
const NOT_FOUND = -1;

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

/**
 Logger that writes nothing, since the composition writes a warn line for each step it takes and the sample takes
 thousands.
 */
const QUIET: Logger = {
  ...tagged({ tag: 'source-live-sample-test', },),
  warn: function ignore(): void {},
};

/**
 A construct found where the original had none, with the text that put it there.
 */
type Finding = {
  /**
   What the text was and where it stood.
   */
  readonly where: string;

  /**
   Constructs found, as type and place.
   */
  readonly found: readonly string[];
};

/**
 Names the slices of a composed page that were written as literal text and are still on it.

 @param composed - the composition
 @param artifact - the artifact it was composed from

 @returns Slice indices, none for a page with every original as recorded

 @example
 ```ts
 const encoded = encodedSlicesOf({ composed, artifact, },);
 ```
 */
function encodedSlicesOf(
  {
    composed,
    artifact,
  }: {
    readonly composed: ComposedPage;
    readonly artifact: WouldShipSource;
  },
): readonly number[] {
  if (composed.step === 'as-written')
    return [];
  if (composed.step !== 'dropped')
    return composed.slices;

  /**
   What each slice ships, and the runs of the slices that ship their original.
   */
  const readings = wouldShipTextPerSlice({ artifact, },);
  const runs = sourceRunsOf({ slices: readings, },);
  /**
   Runs the first step writes as literal text, those that cannot stand beside another, hold a comment nothing closes or
   do not parse alone.
   */
  const unfit = runsUnfitForCompany({
    runs,
    slices: readings,
  },);
  const failing = runs
    .filter(function failsAlone(run,): boolean {
      return unfit.includes(run,) || holdsOpenComment({
        run,
        slices: readings,
      },) || (readPageGrammar({
        pageText: runTextOf({
          run,
          slices: readings,
        },),
      },)
        .kind
        === 'refused');
    },)
    .flat();
  return failing.filter(function stillOnPage(slice,): boolean {
    return !composed.slices.includes(slice,);
  },);
}

/**
 Composes the page of a pair of runs, with a paragraph the archive covers before them and one between.

 @param pair - the two runs and the English between them

 @returns The composition and the artifact it was composed from

 @example
 ```ts
 const { composed, artifact, } = composePair({ pair: { first: '{', middle: 'Hi.\n', second: '}', }, },);
 ```
 */
function composePair(
  { pair, }: { readonly pair: RunPair; },
): {
  readonly composed: ComposedPage;
  readonly artifact: WouldShipSource;
} {
  /**
   Slices of the entry: the archive's first paragraph, the first run, the English between, the second run.
   */
  const rows: readonly SourceShippedRow[] = [
    {
      incumbentText: '## Description\n\nA tabby.\n',
      sourceText: '## 简介\n\n猫。\n',
    },
    {
      incumbentText: '',
      sourceText: pair.first,
    },
    {
      incumbentText: `\n${pair.middle}`,
      sourceText: '\n她睡。\n',
    },
    {
      incumbentText: '',
      sourceText: pair.second,
    },
  ];
  const artifact = sourceShippedArtifact({ rows, },);
  return {
    artifact,
    composed: composeSourcePage({
      artifact,
      slices: sourceShippedSlices({ rows, },),
      archiveText: sourceShippedArchive({ rows, },),
      entryId: 'Cat',
      l: QUIET,
    },),
  };
}

/**
 Reads the constructs of a page that stand in the text of the slices written as literal text.

 @param composed - the composition
 @param encoded - slices written as literal text that are still on the page

 @returns Constructs inside those slices' text, as type and place; a note for a slice whose text is not found on the
 page and one for a page the grammar refuses to read

 @example
 ```ts
 const found = liveInEncodedSlices({ composed, encoded: [1, 3,], },);
 ```
 */
function liveInEncodedSlices(
  {
    composed,
    encoded,
  }: {
    readonly composed: ComposedPage;
    readonly encoded: readonly number[];
  },
): readonly string[] {
  /**
   Where the text of each such slice stands on the page, at every place it occurs, and the slices not found at all.
   */
  const ranges = encoded.flatMap(function rangesOf(slice,): readonly {
    readonly slice: number;
    readonly start: number;
    readonly end: number;
  }[] {
    /**
     What the composition spliced in for the slice.
     */
    const text = composed.replacements.find(function isSlice({ sliceIndex, },): boolean {
      return sliceIndex === slice;
    },)?.replacementText ?? '';
    if (text === '')
      return [{
        slice,
        start: NOT_FOUND,
        end: NOT_FOUND,
      },];
    /**
     Offsets the text starts at.
     */
    const starts: number[] = [];
    for (
      let from = composed.pageText.indexOf(text,);
      from !== NOT_FOUND;
      from = composed.pageText.indexOf(text, from + 1,)
    )
      starts.push(from,);
    return (starts.length === 0)
      ? [{
        slice,
        start: NOT_FOUND,
        end: NOT_FOUND,
      },]
      : starts.map(function toRange(start,): {
        readonly slice: number;
        readonly start: number;
        readonly end: number;
      } {
        return {
          slice,
          start,
          end: start + text.length,
        };
      },);
  },);
  /**
   Slices whose text the page does not hold, which no construct can be placed in.
   */
  const unlocated = ranges
    .filter(function isUnlocated({ start, },): boolean {
      return start === NOT_FOUND;
    },)
    .map(function toNote({ slice, },): string {
      return `slice ${String(slice,)} not found on the page`;
    },);
  try {
    return [
      ...unlocated,
      ...liveConstructsOf({ pageText: composed.pageText, },)
        .filter(function standsInEncoded({ start, },): boolean {
          return ranges.some(function holds(range,): boolean {
            return (start >= range.start) && (start < range.end);
          },);
        },)
        .map(function toNote({
          type,
          start,
          end,
        },): string {
          return `${type}@${String(start,)}-${String(end,)}`;
        },),
    ];
  }
  catch (error) {
    return [...unlocated, `the page cannot be read: ${Error.isError(error,) ? error.name : String(error,)}`,];
  }
}

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
        const read = sample.map(function readText(text,): {
          readonly parses: boolean;
          readonly finding: Finding;
        } {
          /**
           The text as the encoder writes it.
           */
          const written = encodeAsLiteralMdx({ text, },);
          if (readPageGrammar({ pageText: `${BEFORE}${written}\n`, },).kind !== 'parses')
            return {
              parses: false,
              finding: {
                where: '',
                found: [],
              },
            };
          return {
            parses: true,
            finding: {
              where: JSON.stringify(text,),
              found: liveConstructsOfCarried({ carried: written, },)
                .map(function toNote({
                  type,
                  start,
                  end,
                },): string {
                  return `${type}@${String(start,)}-${String(end,)}`;
                },),
            },
          };
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
        const read = pairs.map(function readPair(pair,): {
          readonly encoded: number;
          readonly finding: Finding;
        } {
          const { composed, artifact, } = composePair({ pair, },);
          const encoded = encodedSlicesOf({
            composed,
            artifact,
          },);
          return {
            encoded: encoded.length,
            finding: {
              where: `${String(pair.at,)} ${JSON.stringify(pair,)} ${composed.step}`,
              found: (encoded.length === 0)
                ? []
                : liveInEncodedSlices({
                  composed,
                  encoded,
                },),
            },
          };
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
