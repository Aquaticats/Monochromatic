import {
  isDecisionSeat,
  MODEL_CARDS,
  ROSTER_MODEL_IDS,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Re-ask alternate models
// WHICH MODELS A REASONING RUNAWAY IS RE-ASKED OF (ledger B1072, the lead's
// decision of 2026-10-29, open to the owner's veto). A reply that ended in a
// length stop with no content is a model that spent its whole completion
// cap thinking; asking the same weights again on another route repeats it
// (6 of the 7 direct DeepSeek calls that ran to the cap, in the runs of
// 2026-10-22 to 2026-10-24, were the re-ask of a NeuralWatt runaway of the
// same model, and each ran away again). The re-ask therefore goes to
// another roster model.
//
// ROSTER ORDER, NOTHING SMARTER. The list is the roster's own order minus
// the model that ran away, and the caller asks each in turn until one has a
// provider with budget. No card records which model suits a question, so
// any further ordering would be a guess; the lead may replace the order.
//
// TWO KINDS OF SEAT ARE LEFT OUT. A decision-only seat answers a typed
// question and never a chat completion, so it cannot take a re-ask. The
// Claude route is metered on the owner's key ("This key would have $200
// every month so be careful what you use"), and a re-ask is a call nobody
// planned, so it is not spent there.
//
// A HAZARD THE LEAD OWNS: the answer is recorded as the voice of the seat
// that ran away, and the model that gave it may sit on the same bench. The
// provider package does not know the bench, so it does not leave such a model
// out (`roster-id.ts` explains why one model in two seats overstates
// agreement).

/**
 Roster models a reasoning runaway may be re-asked of, in the order to try them.

 @param modelId - model that ran away, left out

 @returns Roster models other than it that take a chat completion and are not on the metered Claude route, in roster
 order; empty only where the roster holds no other such model

 @example
 ```ts
 const [first,] = alternateModelsFor({ modelId: 'deepseek-v4.1-flash', },);
 ```
 */
export function alternateModelsFor(
  { modelId, }: { readonly modelId: RosterModelId; },
): readonly RosterModelId[] {
  return ROSTER_MODEL_IDS.filter(function isAlternate(candidate,): boolean {
    /**
     The candidate's card.
     */
    const card = MODEL_CARDS[candidate];
    return (candidate !== modelId)
      && (!isDecisionSeat({ modelId: candidate, },))
      && (card.anthropic === undefined);
  },);
}

//endregion Re-ask alternate models
