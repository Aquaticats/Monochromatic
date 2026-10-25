import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { commentRegionsOf, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type PageGrammarReading,
  readPageGrammar,
} from './page-grammar.ts';
import {
  textOnPage,
  UNTRANSLATED_PASSAGE_NOTE,
} from './source-ships.ts';
import {
  type WouldShipSlice,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from './would-ship-text.ts';

//region Source ships runs
// THE RUNS OF SLICES SHIPPING THEIR ORIGINAL, AND WHICH OF THEM THE PAGE CANNOT CARRY. A run is a contiguous run of
// slices shipping their original, the unit the note stands before (`source-ships.ts`). The composition of the page
// (`source-ships-compose.ts`) and the judgement of a page on disk (`page-agreement.ts`) both need to say which runs a
// page is shipped without, and they ask here, so that the page written and the page expected are one answer.

/**
 What the grammar says of a text, handed in so a case can stand in for it.
 */
export type GrammarReader = (arguments_: { readonly pageText: string; },) => PageGrammarReading;

/**
 Groups the slices that ship their original into runs.

 @param slices - every slice's reading, in comparison-row order

 @returns Slice indices of each run, in page order; none when no slice ships its original

 @example
 ```ts
 const runs = sourceRunsOf({ slices: wouldShipTextPerSlice({ artifact, },), },);
 ```
 */
export function sourceRunsOf(
  { slices, }: { readonly slices: readonly WouldShipSlice[]; },
): readonly (readonly number[])[] {
  /**
   Runs found so far, the last of which a continuing slice joins.
   */
  const runs: number[][] = [];
  for (const slice of slices) {
    /**
     What the slice ships.
     */
    const { reading, } = slice;
    if (reading.kind !== 'source-ships')
      continue;
    if (reading.opensRun) {
      runs.push([slice.sliceIndex,],);
      continue;
    }
    /**
     The run this slice continues.
     */
    const run = nonNullishOrThrow(runs.at(-1,),);
    run.push(slice.sliceIndex,);
  }
  return runs;
}

/**
 Text a run of slices would make of a page of its own, as the page carries it.

 @param run - slice indices of the run

 @param slices - every slice's reading, in comparison-row order

 @param encoding - how the originals are written, as recorded unless a case asks for literal text

 @returns Note and originals, one blank line apart

 @example
 ```ts
 const text = runTextOf({ run: [88, 89,], slices, },);
 ```
 */
export function runTextOf(
  {
    run,
    slices,
    encoding = 'as-written',
  }: {
    readonly run: readonly number[];
    readonly slices: readonly WouldShipSlice[];
    readonly encoding?: 'as-written' | 'literal';
  },
): string {
  /**
   What each slice of the run puts on the page.
   */
  const texts = slices.flatMap(function textOf(slice,): readonly string[] {
    /**
     What the slice ships.
     */
    const { reading, } = slice;
    return (run.includes(slice.sliceIndex,) && (reading.kind === 'source-ships'))
      ? [
        textOnPage({
          reading,
          encoding,
        },),
      ]
      : [];
  },);
  return `${texts.join('\n\n',)}\n`;
}

/**
 Text a run of slices puts on the page after the note, the slices one blank line apart.

 @param run - slice indices of the run, which opens with a slice that opens a run

 @param slices - every slice's reading, in comparison-row order

 @param encoding - how the originals are written

 @returns The originals alone, without the note and the line ending the page text ends with

 @throws {@link Error} when the text of the run does not open with the note, which a run of slices does by its definition

 @example
 ```ts
 const body = runBodyOf({ run: [88, 89,], slices, encoding: 'literal', },);
 ```
 */
export function runBodyOf(
  {
    run,
    slices,
    encoding,
  }: {
    readonly run: readonly number[];
    readonly slices: readonly WouldShipSlice[];
    readonly encoding: 'as-written' | 'literal';
  },
): string {
  /**
   The note and the blank line after it.
   */
  const opening = `${UNTRANSLATED_PASSAGE_NOTE}\n\n`;
  /**
   The text the run would make of a page of its own.
   */
  const text = runTextOf({
    run,
    slices,
    encoding,
  },);
  if (!text.startsWith(opening,))
    throw new Error('a run of slices shipping their original does not open with the note',);
  return text.slice(
    opening.length,
    -1,
  );
}

/**
 Whether the text a run puts on the page opens an HTML comment that never closes.

 @param run - slice indices of the run

 @param slices - every slice's reading, in comparison-row order

 @returns Whether the run's own text has such an opener

 @example
 ```ts
 const open = holdsOpenComment({ run: [88, 89,], slices, },);
 ```
 */
export function holdsOpenComment(
  {
    run,
    slices,
  }: {
    readonly run: readonly number[];
    readonly slices: readonly WouldShipSlice[];
  },
): boolean {
  return commentRegionsOf({
    text: runTextOf({
      run,
      slices,
    },),
  },)
    .some(function neverCloses({ terminated, },): boolean {
      return !terminated;
    },);
}

/**
 Names slices the way every line of the composition prints them.

 @param indices - slice indices

 @returns `[1, 3]`

 @example
 ```ts
 const named = namedSlices({ indices: [1, 3,], },);
 ```
 */
export function namedSlices({ indices, }: { readonly indices: readonly number[]; },): string {
  return `[${indices.join(', ',)}]`;
}

/**
 Slices of the runs the grammar refuses even written as literal text, which no writing of the original makes parse.

 @param artifact - settled entry, read for what each slice would carry

 @param readGrammar - what the grammar says of a text, the strict MDX grammar the site compiles with by default

 @returns Slice indices in page order, none where every run parses as literal text

 @example
 ```ts
 const unwritable = slicesRefusedAsLiteral({ artifact, },);
 ```
 */
export function slicesRefusedAsLiteral(
  {
    artifact,
    readGrammar = readPageGrammar,
  }: {
    readonly artifact: WouldShipSource;
    readonly readGrammar?: GrammarReader;
  },
): readonly number[] {
  /**
   What each slice would carry.
   */
  const readings = wouldShipTextPerSlice({ artifact, },);
  return sourceRunsOf({ slices: readings, },)
    .filter(function refusedAsLiteral(run,): boolean {
      return readGrammar({
        pageText: runTextOf({
          run,
          slices: readings,
          encoding: 'literal',
        },),
      },)
        .kind
        === 'refused';
    },)
    .flat();
}

/**
 The sets of slices a page may be shipped without, in the order the composition tries them.

 THE RUNS THE GRAMMAR REFUSES EVEN WRITTEN AS LITERAL TEXT COME FIRST: no writing of such a run makes a page parse, and
 the runs beside it are not to blame. EVERY RUN COMES LAST, for a page the runs refuse together (or one the English
 refuses) and for the pages shipped before this order existed. The first set is left out where it would be the last.

 @param artifact - settled entry, read for what each slice would carry

 @param readGrammar - what the grammar says of a text, the strict MDX grammar the site compiles with by default

 @returns Slice indices of each set, none where no slice ships its original

 @example
 ```ts
 const sets = droppedRunSets({ artifact, },);
 ```
 */
export function droppedRunSets(
  {
    artifact,
    readGrammar = readPageGrammar,
  }: {
    readonly artifact: WouldShipSource;
    readonly readGrammar?: GrammarReader;
  },
): readonly (readonly number[])[] {
  /**
   Slices of each run.
   */
  const runs = sourceRunsOf({ slices: wouldShipTextPerSlice({ artifact, },), },);
  /**
   Every slice shipping its original.
   */
  const everySlice = runs.flat();
  if (everySlice.length === 0)
    return [];
  /**
   Slices of the runs no writing makes parse.
   */
  const refused = slicesRefusedAsLiteral({
    artifact,
    readGrammar,
  },);
  return ((refused.length === 0) || (refused.length === everySlice.length))
    ? [everySlice,]
    : [
      refused,
      everySlice,
    ];
}

//endregion Source ships runs
