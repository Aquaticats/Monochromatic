import {
  type ComposedPage,
  type GrammarReader,
  holdsOpenComment,
  readPageGrammar,
  runsUnfitForCompany,
  runTextOf,
  sourceRunsOf,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from '../../dist/final/node/index.mjs';

//region Slices written as literal text
// WHICH SLICES OF A COMPOSED PAGE THE FIRST STEP WROTE AS LITERAL TEXT AND THE PAGE STILL HOLDS, read the way the first
// step reads them (`source-ships-compose.ts`), so that a case can ask where to look for the constructs a writing of
// the text may have left live; and which slices the page lost that no refusal for depth accounts for. Split from
// `source-live-pairs.test-fixture.ts` for its length.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 What the grammar says of a page that nests past the bound the readers keep, which no writing of the text makes it
 accept: the one reason a page ships without the original of a run.
 */
const DEPTH_REFUSAL = 'nested too deeply to read';

/**
 Names the slices of a composed page that were left off it though the grammar does not refuse their run for its depth.
 A run is excused by what the grammar says of that run's own text written as literal text, never by a refusal of
 another run's or of the page, so a run dropped for another reason beside one refused for its depth is named.

 @param composed - composition

 @param artifact - artifact it was composed from

 @param readGrammar - what the grammar says of a text

 @returns Slice indices in page order, none for a page that kept every original or lost only runs the grammar refuses
 for their depth

 @example
 ```ts
 const unexplained = dropsWithoutCause({ composed, artifact, readGrammar: readPageGrammar, },);
 ```
 */
export function dropsWithoutCause(
  {
    composed,
    artifact,
    readGrammar,
  }: {
    readonly composed: ComposedPage;
    readonly artifact: WouldShipSource;
    readonly readGrammar: GrammarReader;
  },
): readonly number[] {
  if (composed.step !== 'dropped')
    return [];
  /**
   What each slice ships.
   */
  const readings = wouldShipTextPerSlice({ artifact, },);
  /**
   The runs of the slices that ship their original.
   */
  const runs = sourceRunsOf({ slices: readings, },);
  return composed
    .slices
    .filter(function lacksDepthRefusal(slice,): boolean {
      /**
       The run the slice belongs to.
       */
      const run = runs.find(function holds(candidate,): boolean {
        return candidate.includes(slice,);
      },);
      if (run === undefined)
        return true;
      /**
       What the grammar says of the run alone, written as literal text.
       */
      const alone = readGrammar({
        pageText: runTextOf({
          run,
          slices: readings,
          encoding: 'literal',
        },),
      },);
      if (alone.kind !== 'refused')
        return true;
      /**
       Whether the refusal is the one for depth.
       */
      const forDepth = alone
        .refusal
        .includes(DEPTH_REFUSAL,);
      return !forDepth;
    },);
}

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

//endregion Slices written as literal text
