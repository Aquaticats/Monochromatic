import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { IssueAuthorship, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { BenchSlice, } from './bench-sample.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Editor calibrate slice
// WHAT ONE SLICE OF THE EDITOR CALIBRATION PRODUCES, and the one line the
// driver prints about it while the run is still going.
//
// SPLIT OUT OF `editor-calibrate.ts` when the overlap dial put that
// file over its line budget. The type moved with the line because the line is
// its only reader outside the driver, and the driver is an entry module:
// nothing an entry module declares may be exported through a barrel, since
// rolldown folds the export into a shared chunk and `import.meta.main` reads
// false in the built command. A line that is to be tested has to live beside
// the driver, not inside it.

/**
 Rounds one slice produced, split by the seat that produced them.
 */
export type SliceRounds = {
  /**
   Rounds the editors' candidates were judged in.
   */
  readonly editor: readonly SelectionRound[];

  /**
   Rounds the refiners' candidates were judged in.
   */
  readonly refiner: readonly SelectionRound[];

  /**
   Whether the naturalness lane found anything on this slice eligible to
   rewrite at all.

   THE REFINER STANDING'S DENOMINATOR. A paragraph under the eligibility
   floor is never offered to a rewriter, so a slice can run the whole lane
   and reach no refiner. Without this, an empty refiner standing cannot be
   told from a rewriter roster that answered nothing.
   */
  readonly refineAsked: boolean;

  /**
   Models that wrote the repair this slice shipped, BEFORE refinement.

   SEPARATE FROM THE ROUNDS, AND NOT A PREFERENCE. Nobody chose between
   alternatives on a slice where every editor proposed the same text, so
   shipping there says the ensemble agreed and says nothing about who would
   have won a vote. Counted anyway, because without it such a slice is
   invisible: it repaired, and the standing records nothing.

   READ OFF THE PRE-REFINEMENT OUTCOME ON PURPOSE. `collectRefinedAuthors`
   unions the editors with any refiner whose rewrite won, so the refined
   outcome's authorship credits both seats in one list and cannot be split
   back apart. Taking it from the accuracy lane's own outcome keeps this
   column about editors, which is what a reader of an EDITOR report assumes.
   */
  readonly editorShipped: readonly RosterModelId[];

  /**
   Models whose rewrite is in the text this slice shipped.

   THE REFINER SEAT'S EQUIVALENT, and the reason `settleRefinedSlice` returns
   `refinedBy` at all. Empty on every slice where no rewrite shipped, which
   includes a rewrite the recheck rolled back.
   */
  readonly refinerShipped: readonly RosterModelId[];

  /**
   Refiners heard with a usable answer on this slice, proposal or not.

   WHAT SEPARATES ANSWERED FROM SILENT in the refiner standing's coverage
   line. Empty on a slice that reached no rewriter.
   */
  readonly refinerHeard: readonly RosterModelId[];
};

/**
 Renders the progress line for one finished slice.

 NUMBERED BY POSITION IN THE SAMPLE, not by arrival. Above an overlap of one
 these lines arrive out of order, and numbering them by arrival would make two
 lines claim the same slice while none claimed the one still running.

 @param position - where the slice sits in the sample, counted from zero

 @param total - slices in the sample

 @param slice - passage the line is about, for its entry and chunk

 @param rounds - what running the slice produced

 @returns One line, indented for the report and without its newline

 @example
 ```ts
 console.log(sliceProgressLine({ position: 0, total: 4, slice, rounds, },),);
 ```
 */
export function sliceProgressLine(
  {
    position,
    total,
    slice,
    rounds,
  }: {
    readonly position: number;
    readonly total: number;
    readonly slice: Pick<BenchSlice, 'entryId' | 'index'>;
    readonly rounds: SliceRounds;
  },
): string {
  /**
   How many rounds the editors were judged in on this slice.
   */
  const editorCount = rounds
    .editor
    .length;

  /**
   How many the refiners were judged in.
   */
  const refinerCount = rounds
    .refiner
    .length;

  /**
   How many editors wrote text this slice shipped.
   */
  const shipping = rounds
    .editorShipped
    .length;

  /**
   Note for a slice where the naturalness lane had nothing to offer a rewriter.
   */
  const reachNote = rounds.refineAsked ? '' : ' (nothing eligible to rewrite)';

  return `  slice ${String(position + 1,)} of ${String(total,)} `
    + `(${slice.entryId} chunk ${String(slice.index,)}): `
    + `${String(editorCount,)} editor ${
      wordForCount({
        count: editorCount,
        one: 'round',
        many: 'rounds',
      },)
    }, `
    + `${String(refinerCount,)} refiner ${
      wordForCount({
        count: refinerCount,
        one: 'round',
        many: 'rounds',
      },)
    }${reachNote}, `
    + `${String(shipping,)} ${
      wordForCount({
        count: shipping,
        one: 'editor',
        many: 'editors',
      },)
    } shipping`;
}

/**
 Every model credited with writing text that shipped on one slice.

 BOTH HALVES OF THE AUTHORSHIP. A model can write a whole chunk or serve one
 issue inside it, and either is having written what shipped.

 @param authorship - what the lane recorded about who wrote the repair

 @returns Models credited, each once

 @example
 ```ts
 const authors = shippedAuthors({ authorship: outcome.authorship, },);
 ```
 */
export function shippedAuthors(
  { authorship, }: { readonly authorship: IssueAuthorship; },
): readonly RosterModelId[] {
  return [
    ...new Set([
      ...authorship.everyIssue,
      ...Object
        .values(authorship.perIssue,)
        .flat(),
    ],),
  ];
}

//endregion Editor calibrate slice
