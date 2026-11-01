import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { heardVoiceOf, } from './stage-answerer.ts';
import type { HeardVoice, } from './stage-gather.ts';
import type { RoundOutcome, } from './stage-round.ts';

//region Stage recovery settlement
// WHAT THE QUORUM LOOP MAKES OF THE OUTCOMES OF THE RECOVERY ROUND (ledger B1371). The round re-asks each seat that
// answered and could not be read, and its outcomes are of three kinds: a voice that was heard, a seat lost as
// unreachable (the router refused its re-ask, or the hold of its card withheld the ask of its own model), and a seat
// that is still unreadable. The first joins the voices, the second leaves the seats the quorum counts, and the third
// stays where it was. Reading the outcomes for the heard voices alone left the second kind in the denominator of the
// quorum, which the rule that an unreachable seat never counts there forbids.
//
// ITS OWN MODULE because `stage-quorum.ts` stands at the line budget the lint allows, and because the settlement is
// the one step of the gather that reads what the recovery round returned.

/**
 Settles the outcomes of the recovery round into the sets the gather reads its quorum from.

 @param recovered - outcomes of re-asking each unreadable seat once

 @param stage - stage label the heard voices are recorded under

 @param l - logger the heard voices are written to

 @param collected - heard voices of the gather, which a recovered voice joins

 @param unreadableSeats - seats whose latest answer nothing could read, each with why; a seat heard or lost as
 unreachable leaves it

 @param unreachableSeats - seats the router refused, which a seat the recovery round lost as unreachable joins

 @mutates collected - gains the voices the recovery round heard

 @mutates unreadableSeats - loses the seats the round heard or lost as unreachable

 @mutates unreachableSeats - gains the seats the round lost as unreachable

 @example
 ```ts
 settleRecovered({ recovered, stage, l, collected, unreadableSeats, unreachableSeats, },);
 ```
 */
export function settleRecovered<ValueT,>(
  {
    recovered,
    stage,
    l,
    collected,
    unreadableSeats,
    unreachableSeats,
  }: {
    readonly recovered: readonly RoundOutcome<ValueT>[];
    readonly stage: string;
    readonly l: Logger;
    readonly collected: HeardVoice<ValueT>[];
    readonly unreadableSeats: Map<RosterModelId, unknown>;
    readonly unreachableSeats: Set<RosterModelId>;
  },
): void {
  for (const outcome of recovered) {
    if (outcome.voice
      .heard) {
      collected.push(heardVoiceOf({
        outcome,
        stage,
        l,
      },),);
      unreadableSeats.delete(outcome.modelId,);
      continue;
    }
    // The seat is unreachable, and it leaves the unreadable seats so no reader counts it as one that answered beside a
    // bench sized without it (`archive-block-review-stage.ts` counts the unreadable seats as answering).
    if (outcome.voice
      .unreachable) {
      unreachableSeats.add(outcome.modelId,);
      unreadableSeats.delete(outcome.modelId,);
    }
  }
}

//endregion Stage recovery settlement
