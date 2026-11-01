import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  type AbsoluteNaturalnessReviewOutcome,
  reviewAbsoluteNaturalness,
} from './absolute-naturalness-review-stage.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Absolute naturalness confirmation
// Publication approval needs a second exact-half-quorum responsibility over
// exact same candidate. Rejection remains immediate: it already proves candidate
// cannot ship and its findings should reach correction without confirmation spend.
// Confirmation challenges prior acceptance with reverse-order reading rather than
// repeating same model and prompt as fake independent evidence.

/**
 Decisive review beside earlier acceptable readings of same exact candidate.

 @example
 ```ts
 const confirmed: ConfirmedAbsoluteNaturalness = { review, confirmations: [initial,] };
 ```
 */
export type ConfirmedAbsoluteNaturalness = {
  /**
   Final review deciding acceptance or correction.
   */
  readonly review: AbsoluteNaturalnessReviewOutcome;

  /**
   Earlier acceptable readings bought before decisive review.
   */
  readonly confirmations: readonly AbsoluteNaturalnessReviewOutcome[];
};

/**
 Requires distinct challenge acceptance after exact-candidate discovery approval.

 @param request - exact request each independent review receives

 @returns Decisive review and every earlier acceptable reading

 @example
 ```ts
 const confirmed = await confirmAbsoluteNaturalness({ client, modelIds, subject, signal, exchangeTimeoutMs, l, stageSettings, });
 ```
 */
export async function confirmAbsoluteNaturalness(
  request: ForeignBorrowed<Parameters<typeof reviewAbsoluteNaturalness>[0]>,
): Promise<ConfirmedAbsoluteNaturalness> {
  /**
   First approving quorum reading.
   */
  const initial = await reviewAbsoluteNaturalness({
    ...request,
    perspective: 'defect-discovery',
  },);
  if (initial.verdict !== 'acceptable') {
    return {
      review: initial,
      confirmations: [],
    };
  }

  /**
   Substantively distinct challenge of prior acceptance over exact same subject.
   */
  const confirmation = await reviewAbsoluteNaturalness({
    ...request,
    perspective: 'acceptance-challenge',
    // THE SEATS THE DISCOVERY ASKED, at the discovery's quorum. Since the
    // fan-out window of 2026-09-09 a review asks quorum plus one seat of its
    // bench, rotated by the prompt, and the two perspectives are two prompts;
    // the artifact reads one requested roster across both readings
    // (`artifact-two-lane-read-naturalness-confirmation.ts`), and a challenge
    // put to the reviewers who approved is what a confirmation is.
    modelIds: initial.seats
      .map(function seatOf(seat,): RosterModelId {
        return seat.modelId;
      },),
    quorumOver: initial.quorumOver,
    fanOut: 'whole-bench',
  },);
  return {
    review: confirmation,
    confirmations: [initial,],
  };
}

//endregion Absolute naturalness confirmation
