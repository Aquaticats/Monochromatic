import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { ConsistencyPlan, } from '@monochromatic-dev/translation-repair-consistency/ts';
import type {
  ArtifactConsistencyCarry,
  ArtifactConsistencyOccurrence,
} from './artifact-two-lane-consistency.ts';
import { outcomesAfter, } from './artifact-two-lane-read-consistency-carry-recount.ts';
import { keptReasonExpected, } from './artifact-two-lane-read-consistency-occurrence.ts';

//region Artifact consistency shared record
// WHICH PLACES OF A GROUP WERE CARRIED UNDER ONE BENCH RECORD. The places of one
// block are put to the bench once, and the same candidates, ballots, usable count,
// quorum and decision are recorded under each of them (`pass-consistency-carry-choose.ts`).
// A reader of a section that records a unit carried in part cannot see the block, since
// the English of a slice the pass left alone is not in the file, but it can see the
// record: places whose carries are the same record were carried together, and the
// splice among their candidates was built from the places among them code could move
// the chosen words into.

/**
 A place the chosen words are carried to, with the plan recomputed for it and the carry recorded under it.
 */
export type CarriedPlace = {
  /**
   The place.
   */
  readonly occurrence: ArtifactConsistencyOccurrence;

  /**
   Where the place sits, for the parse error.
   */
  readonly here: string;

  /**
   What moving the chosen words over the place by code amounts to.
   */
  readonly plan: ConsistencyPlan['kind'];

  /**
   The carry recorded under the place.
   */
  readonly carry: ArtifactConsistencyCarry;
};

/**
 Whether a carry holds a splice among its candidates.

 @param carry - the carry

 @returns Whether a candidate is the chosen words moved in by code

 @example
 ```ts
 const spliced = carriesSplice({ carry, },);
 ```
 */
export function carriesSplice({ carry, }: { readonly carry: ArtifactConsistencyCarry; },): boolean {
  return carry.candidates
    .some(function isSplice(candidate,): boolean {
      return candidate.kind === 'splice';
    },);
}

/**
 The part of a carry that is one bench record: everything but the label of the place it is recorded under.

 @param carry - the carry

 @returns The record as text, equal for two carries exactly when they are the same record

 @example
 ```ts
 const record = recordOf({ carry, },);
 ```
 */
function recordOf({ carry, }: { readonly carry: ArtifactConsistencyCarry; },): string {
  return JSON.stringify({
    candidates: carry.candidates,
    ballots: carry.ballots,
    usable: carry.usable,
    quorum: carry.quorum,
    decision: carry.decision,
  },);
}

/**
 Whether a place that code can move the chosen words into stands under the same bench record as a carry.

 @param carry - the carry of a place whose plan is `unavailable` and that holds a splice candidate

 @param carriedTo - every place of the group that was carried to, with its plan and its carry

 @returns Whether another place whose plan is `replace` was recorded under the same candidates, ballots and decision, so
 the splice among them was built for it

 @example
 ```ts
 const built = spliceBuiltForSomePlace({ carry, carriedTo, },);
 ```
 */
export function spliceBuiltForSomePlace(
  {
    carry,
    carriedTo,
  }: {
    readonly carry: ArtifactConsistencyCarry;
    readonly carriedTo: readonly CarriedPlace[];
  },
): boolean {
  /**
   The record the carry belongs to.
   */
  const record = recordOf({ carry, },);
  return carriedTo.some(function movesAndShares(other,): boolean {
    return (other.plan === 'replace') && (recordOf({ carry: other.carry, },) === record);
  },);
}

/**
 What a refusal of a splice candidate says kept code from moving the words.

 @param label - the receiving place's label

 @param marks - whether the section recorded the marks beside the words, which
 adds two causes code could not move them for

 @returns The reason, for the parse error

 @example
 ```ts
 const reason = noSpliceReasonFor({ label: 'OCCURRENCE 2', marks: 'unrecorded', },);
 ```
 */
function noSpliceReasonFor(
  {
    label,
    marks,
  }: {
    readonly label: string;
    readonly marks: 'compared' | 'unrecorded';
  },
): string {
  if (marks === 'unrecorded')
    return `no splice candidate for ${label}, whose frame or the chosen frame has no slot to move the words by`;
  return `no splice candidate for ${label}, whose words code cannot move: the frame or the chosen frame has no slot to `
    + 'move them by, a single mark beside the words differs between the two places, the quotation marks inside '
    + 'the words differ, the two hold a different number of block breaks (a blank line, or a line break the Markdown '
    + 'parse reads as a new block), or a single mark stands beside the words and the chosen words end on that side in '
    + 'another class of character than the receiving words, a letter or a digit against anything else';
}

/**
 Whether the decision of a carry chose the splice among its candidates.

 @param carry - the carry

 @returns Whether the carry chose a candidate that is the chosen words moved in by code

 @example
 ```ts
 const chosen = choosesSplice({ carry, },);
 ```
 */
function choosesSplice({ carry, }: { readonly carry: ArtifactConsistencyCarry; },): boolean {
  if (carry.decision
    .kind
    !== 'chosen')
    return false;
  /**
   Label of the candidate chosen.
   */
  const { label, } = carry.decision;
  return carry.candidates
    .some(function isChosenSplice(candidate,): boolean {
      return (candidate.label === label) && (candidate.kind === 'splice');
    },);
}

/**
 Whether a place may end `kept-current` although the splice was chosen and the recorded words let code move them into it.
 The pass decides four things on the English it holds and the record does not: whether an apostrophe the splice carries to
 an edge where the place's words held none is still an apostrophe on the page every replacement makes (the gain case), whether
 the splice would write a second apostrophe beside a mark the place keeps past markup, whether it would change how a
 mark outside the words is read, and whether the page holds a character reference that is a quotation mark (`&#39;`), which
 refuses every gain on it. The record holds the words and the marks directly beside them and no more, so for a place
 whose recomputed plan is `replace` the reader accepts `kept-current` beside the outcomes a replacement ends in, and
 cannot check which of the four left the place alone. It does require the place to RECORD which (`keptBecause`, a reason
 of a closed set), so a place left alone with no stated reason is refused. A section that recorded no marks beside the
 words predates all four decisions, so there `kept-current` stays an outcome only the other cases allow.

 @param plan - the plan recomputed for the place

 @param carry - the carry recorded under the place

 @param marks - whether the section recorded the marks beside the words

 @returns Whether `kept-current` is an outcome the carry allows beside the others

 @example
 ```ts
 const may = mayBeLeftAsItWas({ plan: 'replace', carry, marks: 'compared', },);
 ```
 */
function mayBeLeftAsItWas(
  {
    plan,
    carry,
    marks,
  }: {
    readonly plan: ConsistencyPlan['kind'];
    readonly carry: ArtifactConsistencyCarry;
    readonly marks: 'compared' | 'unrecorded';
  },
): boolean {
  if ((plan !== 'replace') || (marks === 'unrecorded'))
    return false;
  return choosesSplice({ carry, },);
}

/**
 Checks each place the chosen words are carried to against the carry recorded under it: a splice candidate under a place
 code cannot move, and the outcome the carry allows the place.

 A splice candidate under a place whose plan is `unavailable` is built for another place of the record it shares, which only
 a section that records a unit carried in part may hold; an older section was written when one such place withheld the
 splice from its whole block. In a section that records a unit carried in part, a place code cannot move keeps its words
 when the splice was chosen. A place whose plan is `replace` may also be recorded `kept-current` after the splice was
 chosen (`mayBeLeftAsItWas`): the pass decides the gain case on the page with every replacement made, and decides a splice
 that writes a second apostrophe beside a kept mark or changes how a mark outside the words is read on the English of the
 place, which the record does not hold, so the reader can check that the outcome is one the carry allows and cannot check why
 the place was left alone. It requires the place to record the reason (`keptBecause`) whenever `kept-current` is allowed
 only through that rule, since a place the carry itself leaves as it was needs none.

 @param carriedTo - the places the chosen words are carried to, in the order of the group

 @param path - where the group sits, for the parse error

 @param marks - whether the section recorded the marks beside the words, which the refusal names

 @param partialUnits - whether the section records a unit carried in part

 @throws {@link ArtifactParseError} when a splice candidate sits under a place no place of its record can use, or an outcome
 is not one the carry allows

 @example
 ```ts
 assertCarriedOutcomes({ carriedTo, path, marks: 'compared', partialUnits: true, },);
 ```
 */
export function assertCarriedOutcomes(
  {
    carriedTo,
    path,
    marks,
    partialUnits,
  }: {
    readonly carriedTo: readonly CarriedPlace[];
    readonly path: string;
    readonly marks: 'compared' | 'unrecorded';
    readonly partialUnits: boolean;
  },
): void {
  carriedTo.forEach(function checkCarried({
    occurrence,
    here,
    plan,
    carry,
  },): void {
    if ((plan === 'unavailable')
      && carriesSplice({ carry, },)
      && ((!partialUnits) || (!spliceBuiltForSomePlace({
        carry,
        carriedTo,
      },)))) {
      throw new ArtifactParseError({
        path: `${path}.carries`,
        reason: noSpliceReasonFor({
          label: occurrence.label,
          marks,
        },),
      },);
    }
    /**
     The outcomes the carry allows this place on its own.
     */
    const byCarry = outcomesAfter({
      carry,
      spliceMovesIt: (!partialUnits) || (plan !== 'unavailable'),
    },);
    /**
     The outcomes the carry allows this place. A splice the pass decides on English the record does not hold (the gain case,
     a second mark past markup, a mark outside the words that would read differently) may leave the place as it was.
     */
    const allowed = [
      ...byCarry,
      ...(mayBeLeftAsItWas({
        plan,
        carry,
        marks,
      },)
        ? ['kept-current' as const,]
        : []),
    ];
    if (!allowed.includes(occurrence.outcome,)) {
      throw new ArtifactParseError({
        path: `${here}.outcome`,
        reason: `${allowed.join(', ',)}, for a carry that decided as recorded`,
      },);
    }
    // A place the carry itself leaves as it was needs no reason; one the rule for splices decided on English the record
    // does not hold leaves as it was must state which of those decisions it was.
    if (occurrence.outcome !== 'kept-current')
      return;
    if (byCarry.includes('kept-current',))
      return;
    if (occurrence.keptBecause !== undefined)
      return;
    throw new ArtifactParseError({
      path: `${here}.keptBecause`,
      reason: keptReasonExpected({ label: occurrence.label, },),
    },);
  },);
}

//endregion Artifact consistency shared record
