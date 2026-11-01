import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SliceRounds, } from './editor-calibrate-slice.ts';
import { roundWasJudged, } from './editor-calibrate-standing.ts';

//region Editor calibrate closing
// THE TWO CLOSING PARAGRAPHS of the editor calibration's report, after both
// standings: how many slices the naturalness lane could reach at all, and what
// shipped with no vote behind it.

/**
 Prints how many slices the naturalness lane could reach at all.

 THE REFINER STANDING'S DENOMINATOR, and it is not the slice count. A
 paragraph under the eligibility floor is never offered to a rewriter, so a
 slice can buy the whole accuracy lane and reach no refiner. Without this an
 empty refiner standing reads as a rewriter roster that answered nothing,
 which is a different and much worse fact.

 THE CLAUSE ABOUT THE OTHER SLICES IS PRINTED ONLY WHEN THERE ARE ANY, and
 counts them: until 2026-10-06 it said "the rest" carried nothing to rewrite
 when every slice had reached a rewriter.

 @param perSlice - what every slice produced

 @example
 ```ts
 printEditorCalibrateRefineReach({ perSlice, },);
 ```
 */
export function printEditorCalibrateRefineReach(
  { perSlice, }: { readonly perSlice: readonly SliceRounds[]; },
): void {
  /**
   Slices carrying a paragraph the lane was willing to offer a rewriter.
   */
  const asked = perSlice.filter(function eligible(slice,): boolean {
    return slice.refineAsked;
  },);

  /**
   Slices that carried no such paragraph.
   */
  const unreached = perSlice.length - asked.length;

  /**
   Clause naming those slices, empty when there are none.
   */
  const restClause = (unreached === 0)
    ? ''
    : `; ${String(unreached,)} ${
      wordForCount({
        count: unreached,
        one: 'slice',
        many: 'slices',
      },)
    } carried no paragraph over the eligibility floor, so no refiner was asked there and ${
      wordForCount({
        count: unreached,
        one: 'its',
        many: 'their',
      },)
    } silence is not evidence about any model`;

  console.log(
    `  reached a rewriter on ${String(asked.length,)} of ${String(perSlice.length,)} ${
      wordForCount({
        count: perSlice.length,
        one: 'slice',
        many: 'slices',
      },)
    }${restClause}`,
  );
}

/**
 Prints what shipped, and how much of it no vote ever touched.

 THE STANDING'S BLIND SPOT, MEASURED. `selectChunkPatch` ships outright when
 every proposal is identical, recording no round because there was nothing to
 choose between. A slice like that repairs and contributes nothing to a
 standing, so without this line a converged run reads as a lane that did no
 work. It was found live: a slice whose panel adjudicated seven issues
 repaired one of them and reported zero editor rounds.

 SHIPPING IS NOT WINNING. Nobody preferred this text to anything, so these
 counts must never be read as a rate against the standing this report prints.

 @param perSlice - what every slice produced

 @example
 ```ts
 printEditorCalibrateShipped({ perSlice, },);
 ```
 */
export function printEditorCalibrateShipped(
  { perSlice, }: { readonly perSlice: readonly SliceRounds[]; },
): void {
  /**
   Slices that shipped a repair without any editor round being judged, a
   round with no ballot counting as none (`roundWasJudged`), as the standing
   counts it.
   */
  const unvoted = perSlice.filter(function converged(slice,): boolean {
    /**
     Whether any editor round on this slice drew a ballot.
     */
    const judged = slice.editor
      .some(function drewBallot(round,): boolean {
        return roundWasJudged({ round, },);
      },);
    return (!judged) && (slice.editorShipped
      .length
      > 0);
  },);

  /**
   Slices that shipped a repair at all.
   */
  const shipping = perSlice.filter(function repaired(slice,): boolean {
    return slice.editorShipped
      .length
      > 0;
  },);

  console.log(
    `\nEDITORS SHIPPED on ${String(shipping.length,)} of ${String(perSlice.length,)} ${
      wordForCount({
        count: perSlice.length,
        one: 'slice',
        many: 'slices',
      },)
    }, `
      + `${String(unvoted.length,)} of them with no editor round judged at all`,
  );

  if (shipping.length === 0) {
    console.log('  NOTHING SHIPPED. No slice in this sample carried an accepted issue.',);
    return;
  }

  /**
   How many slices each model wrote shipping text on.
   */
  const credits = new Map<RosterModelId, number>();

  for (const slice of shipping) {
    for (const modelId of slice.editorShipped) {
      credits.set(
        modelId,
        (credits.get(modelId,) ?? 0) + 1,
      );
    }
  }

  for (
    const [modelId, count,] of [...credits.entries(),].toSorted(function byCount(
      left: ReadonlyEntry<RosterModelId, number>,
      right: ReadonlyEntry<RosterModelId, number>,
    ): number {
      return right[1] - left[1];
    },)
  ) {
    console.log(
      `  ${modelId}: wrote shipping text on ${String(count,)} of ${String(shipping.length,)} ${
        wordForCount({
          count: shipping.length,
          one: 'slice',
          many: 'slices',
        },)
      }`,
    );
  }

  console.log(
    '  THIS IS NOT A PREFERENCE. A model ships here by writing text that survived, including '
      + 'text every other editor proposed identically, which no judge was ever asked about.',
  );
}

//endregion Editor calibrate closing
