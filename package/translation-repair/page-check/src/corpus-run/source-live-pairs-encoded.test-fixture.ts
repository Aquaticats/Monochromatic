import {
  type ComposedPage,
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
// the text may have left live. Split from `source-live-pairs.test-fixture.ts` for its length.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

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
