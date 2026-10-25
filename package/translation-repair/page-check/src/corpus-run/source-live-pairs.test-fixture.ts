import {
  capturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  composeSourcePage,
  type ComposedPage,
  encodeAsLiteralMdx,
  holdsOpenComment,
  type PageGrammarReading,
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
import type { RunPair, } from './source-live-sample.test-fixture.ts';
import {
  sourceShippedArchive,
  sourceShippedArtifact,
  type SourceShippedRow,
  sourceShippedSlices,
} from './source-shipped.test-fixture.ts';

//region Live constructs of a text and of a pair of runs
// THE WHOLE-PAGE CHECK OF LIVE CONSTRUCTS FOR ONE TEXT AND FOR ONE PAIR OF RUNS
// (`source-live-constructs.test-fixture.ts` finds the constructs of a page), shared by the sampled case and the full
// pass over the generated sets (`source-live-sample.unit.test.ts`, `source-live-full.expensive.unit.test.ts`), so that
// both read a text the same way.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 Page text standing before the encoded text in the parse cases, the way the note stands before an original.
 */
export const BEFORE = '## Description\n\n> This passage has not been translated yet. The original text follows.\n\n';

/**
 What `indexOf` answers for a text it does not find.
 */
const NOT_FOUND = -1;

/**
 A construct found where the original had none, with the text that put it there.
 */
export type Finding = {
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
 What the grammar says of a page that nests past the bound the readers keep, which no writing of the text makes it
 accept: the one reason a page ships without the original of a run.
 */
const DEPTH_REFUSAL = 'nested too deeply to read';

/**
 Where the text of a slice stands on a composed page.
 */
type Placed = {
  /**
   Index of the slice.
   */
  readonly slice: number;

  /**
   Offset the text starts at, `NOT_FOUND` for a text the page does not hold.
   */
  readonly start: number;

  /**
   Offset after the text.
   */
  readonly end: number;
};

/**
 Names the slices of a composed page that were written as literal text and are still on it.

 @param composed - composition

 @param artifact - artifact it was composed from

 @returns Slice indices, none for a page with every original as recorded

 @example
 ```ts
 const encoded = encodedSlicesOf({ composed, artifact, },);
 ```
 */
export function encodedSlicesOf(
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
   What each slice ships.
   */
  const readings = wouldShipTextPerSlice({ artifact, },);
  /**
   The runs of the slices that ship their original.
   */
  const runs = sourceRunsOf({ slices: readings, },);
  /**
   Runs the first step writes as literal text, those that cannot stand beside another, hold a comment nothing closes or
   do not parse alone.
   */
  const unfit = runsUnfitForCompany({
    runs,
    slices: readings,
  },);
  /**
   Slices of the runs the first step writes as literal text.
   */
  const failing = runs
    .filter(function failsAlone(run,): boolean {
      /**
       What the page grammar says of the run alone.
       */
      const alone = readPageGrammar({
        pageText: runTextOf({
          run,
          slices: readings,
        },),
      },);
      /**
       Whether the run holds a comment nothing closes.
       */
      const opensComment = holdsOpenComment({
        run,
        slices: readings,
      },);
      /**
       Whether the grammar refuses the run alone.
       */
      const refused = alone.kind === 'refused';
      /**
       Whether the run cannot stand beside another as it is written.
       */
      const unfitForCompany = unfit.includes(run,);
      return unfitForCompany
        || opensComment
        || refused;
    },)
    .flat();
  return failing.filter(function stillOnPage(slice,): boolean {
    return !composed
      .slices
      .includes(slice,);
  },);
}

/**
 Composes the page of a pair of runs, with a paragraph the archive covers before them and one between.

 @param pair - two runs and the English between them

 @returns Composition, the artifact it was composed from, and what the grammar refused of the pages the
 composition tried, as it worded each refusal

 @example
 ```ts
 const { composed, artifact, refusals, } = composePair({ pair: { first: '{', middle: 'Hi.\n', second: '}', }, },);
 ```
 */
export function composePair(
  { pair, }: { readonly pair: RunPair; },
): {
  readonly composed: ComposedPage;
  readonly artifact: WouldShipSource;
  readonly refusals: readonly string[];
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
  /**
   The settled artifact of the entry.
   */
  const artifact = sourceShippedArtifact({ rows, },);
  /**
   What the grammar refused, in the order the composition asked.
   */
  const refusals: string[] = [];
  return {
    artifact,
    refusals,
    composed: composeSourcePage({
      artifact,
      slices: sourceShippedSlices({ rows, },),
      archiveText: sourceShippedArchive({ rows, },),
      entryId: 'Cat',
      // The composition writes a line for each step it takes, which a full pass takes thousands of times.
      l: capturingLogger({ messages: [], },),
      readGrammar: function recording({ pageText, },): PageGrammarReading {
        /**
         What the page grammar says of the text.
         */
        const reading = readPageGrammar({ pageText, },);
        if (reading.kind === 'refused')
          refusals.push(reading.refusal,);
        return reading;
      },
    },),
  };
}

/**
 Finds every offset a text starts at in a page.

 @param page - page text

 @param text - text looked for, not empty

 @returns Offsets in page order, none for a text the page does not hold

 @example
 ```ts
 const starts = startsOf({ page: 'a b a', text: 'a', },);
 ```
 */
function startsOf(
  {
    page,
    text,
  }: {
    readonly page: string;
    readonly text: string;
  },
): readonly number[] {
  /**
   Offsets found.
   */
  const starts: number[] = [];
  for (
    let at = page.indexOf(
      text,
      0,
    );
    at !== NOT_FOUND;
    at = page.indexOf(
      text,
      at + 1,
    )
  )
    starts.push(at,);
  return starts;
}

/**
 Finds where the text of a slice stands on a composed page, at every place it occurs.

 @param composed - composition

 @param slice - index of the slice

 @returns One entry for each place, or one entry with `NOT_FOUND` for a slice whose text the page does not hold

 @example
 ```ts
 const placed = placesOf({ composed, slice: 1, },);
 ```
 */
function placesOf(
  {
    composed,
    slice,
  }: {
    readonly composed: ComposedPage;
    readonly slice: number;
  },
): readonly Placed[] {
  /**
   What the composition spliced in for the slice.
   */
  const replacement = composed
    .replacements
    .find(function isSlice({ sliceIndex, },): boolean {
      return sliceIndex === slice;
    },);
  /**
   Its text, empty for a slice nothing was spliced for.
   */
  const text = replacement?.replacementText ?? '';
  /**
   Offsets the text starts at.
   */
  const starts = (text === '') ? [] : startsOf({
    page: composed.pageText,
    text,
  },);
  if (starts.length === 0)
    return [
      {
        slice,
        start: NOT_FOUND,
        end: NOT_FOUND,
      },
    ];
  return starts.map(function toPlace(start,): Placed {
    return {
      slice,
      start,
      end: start + text.length,
    };
  },);
}

/**
 Reads the constructs of a page that stand in the text of the slices written as literal text.

 @param composed - composition

 @param encoded - slices written as literal text that are still on the page

 @returns Constructs inside those slices' text, as type and place; a note for a slice whose text is not found on the
 page and one for a page the grammar refuses to read

 @example
 ```ts
 const found = liveInEncodedSlices({ composed, encoded: [1, 3,], },);
 ```
 */
export function liveInEncodedSlices(
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
  const ranges = encoded.flatMap(function rangesOf(slice,): readonly Placed[] {
    return placesOf({
      composed,
      slice,
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
        .map(function toNote(
          {
            type,
            start,
            end,
          },
        ): string {
          return `${type}@${String(start,)}-${String(end,)}`;
        },),
    ];
  }
  catch (error) {
    return [
      ...unlocated,
      `the page cannot be read: ${Error.isError(error,) ? error.name : String(error,)}`,
    ];
  }
}

/**
 Reads a pair of runs: composes its page, finds the constructs in the text written as literal text, and notes a page
 that shipped without the original of a run though neither run nests past the bound.

 @param pair - pair, with the index of its draw when it has one

 @returns How many slices were written as literal text, what was found in them or in the dropping, and the step the
 composition took

 @example
 ```ts
 const { encoded, finding, } = readPair({ pair: { first: '{', middle: 'Hi.\n', second: '}', }, },);
 ```
 */
export function readPair(
  { pair, }: { readonly pair: RunPair & { readonly at?: number; }; },
): {
  readonly encoded: number;
  readonly finding: Finding;
  readonly step: ComposedPage['step'];
} {
  /**
   The page of the pair and the artifact it was composed from.
   */
  const {
    composed,
    artifact,
    refusals,
  } = composePair({ pair, },);
  /**
   Slices written as literal text that the page still holds.
   */
  const encoded = encodedSlicesOf({
    composed,
    artifact,
  },);
  /**
   Whether the grammar refused a page for its depth, the one refusal no writing of the text cures.
   */
  const refusedForDepth = refusals.some(function isDepth(refusal,): boolean {
    return refusal.includes(DEPTH_REFUSAL,);
  },);
  /**
   Whether the composition shipped the page without the original of a run.
   */
  const dropped = composed.step === 'dropped';
  /**
   The slices whose original is not on the page, as the note names them.
   */
  const droppedSlices = composed
    .slices
    .join(' ',);
  /**
   Whether the page lost an original for no reason a writing of the text cannot cure.
   */
  const droppedWithoutCause = dropped && (!refusedForDepth);
  return {
    encoded: encoded.length,
    step: composed.step,
    finding: {
      where: `${String(pair.at ?? NOT_FOUND,)} ${JSON.stringify(pair,)} ${composed.step}`,
      found: [
        ...(droppedWithoutCause
          ? [`shipped without the source of slices ${droppedSlices} though the grammar refused no page for its depth`,]
          : []),
        ...((encoded.length === 0)
          ? []
          : liveInEncodedSlices({
            composed,
            encoded,
          },)),
      ],
    },
  };
}

/**
 Reads a text: writes it as the encoder does after the note and finds the constructs in it.

 @param text - source text

 @returns Whether the page grammar reads the text once written, and what was found in it

 @example
 ```ts
 const { parses, finding, } = readText({ text: '[^1]: <b>\n', },);
 ```
 */
export function readText(
  { text, }: { readonly text: string; },
): {
  readonly parses: boolean;
  readonly finding: Finding;
} {
  /**
   The text as the encoder writes it.
   */
  const written = encodeAsLiteralMdx({ text, },);
  /**
   What the page grammar says of the page with the text after the note.
   */
  const reading = readPageGrammar({ pageText: `${BEFORE}${written}\n`, },);
  if (reading.kind !== 'parses')
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
        .map(function toNote(
          {
            type,
            start,
            end,
          },
        ): string {
          return `${type}@${String(start,)}-${String(end,)}`;
        },),
    },
  };
}

//endregion Live constructs of a text and of a pair of runs
