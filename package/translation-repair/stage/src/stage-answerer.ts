import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { HeardVoice, } from './stage-gather.ts';
import type { RoundOutcome, } from './stage-round.ts';

//region Stage answerer
// WHO WROTE THE ANSWER OF A SEAT (ledger B1142). A seat whose reasoning runaway was re-asked of another model has an
// answer that model wrote, and the outcome of the round still stands under the seat. A stage that records, weighs or
// counts the answer by its author reads it through this one reader, so none of them reads the seat in its place.

/**
 The model that wrote the answer of a seat's round.

 @param outcome - one seat's outcome from a round

 @returns Model that answered where another than the seat did; the seat itself otherwise, and for a lost voice
 (which has no author) the seat

 @example
 ```ts
 const author = answererOf({ outcome, },);
 ```
 */
export function answererOf<ValueT,>({ outcome, }: { readonly outcome: RoundOutcome<ValueT>; },): RosterModelId {
  return outcome.voice
    .heard
    ? (outcome.voice
      .answeredBy
      ?? outcome.modelId)
    : outcome.modelId;
}

/**
 The heard voice a seat's outcome comes to: the model that wrote the answer, and the seat when that is another model.

 @param outcome - one seat's outcome from a round, which must be heard

 @param stage - stage the gather runs for, which the line that names both the seat and the author carries

 @param l - logger of the gather

 @returns Voice whose `modelId` is the author, with the `seat` set only where another model answered for the seat

 @throws {@link Error} when the outcome is no heard voice, which the caller has already ruled out

 @example
 ```ts
 const voice = heardVoiceOf({ outcome, stage: 'editor', l, },);
 ```
 */
export function heardVoiceOf<ValueT,>(
  {
    outcome,
    stage,
    l,
  }: {
    readonly outcome: RoundOutcome<ValueT>;
    readonly stage: string;
    readonly l: Logger;
  },
): HeardVoice<ValueT> {
  /**
   What came back from the seat.
   */
  const { voice, } = outcome;
  if (!voice.heard)
    throw new Error(`unreachable: ${outcome.modelId} is no heard voice`,);
  if (voice.answeredBy !== undefined) {
    l.info(
      `${stage}: ${outcome.modelId}: the answer was written by ${voice.answeredBy}, which the re-ask of a runaway asked in its place`,
    );
  }
  return {
    modelId: voice.answeredBy ?? outcome.modelId,
    // Conditional spread keeps the seat absent where the seat's own model answered.
    ...((voice.answeredBy === undefined) ? {} : { seat: outcome.modelId, }),
    value: voice.value,
  };
}

//endregion Stage answerer
