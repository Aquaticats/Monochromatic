import {
  recordContest,
  type SelectionOutcome,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { decideBestCandidate, } from './candidate-select.ts';
import type { ProducerRosterError, } from '@monochromatic-dev/translation-repair-assembly/ts';

//region Candidate selection recording
// THE PUBLIC SELECTION ENTRY POINT, which is the deciding call plus the record
// of what it decided over.
//
// A WRAPPER RATHER THAN A HOOK INSIDE THE CASCADE. `decideBestCandidate` leaves
// by six different returns, five of them declines, and threading a write
// through each one would mean five chances to forget the sixth. Wrapping the
// whole call records every path by construction, and leaves the decision logic
// untouched.
//
// A SEPARATE MODULE because `candidate-select.ts` sits at 269 of its 300
// permitted code lines, and the request type restated here would breach it.
// `Parameters<typeof ...>` borrows the signature instead of copying it, so the
// two cannot drift.

/**
 Chooses among candidates and records the contest that decided it.

 Identical in behaviour to {@link decideBestCandidate}; every caller in this
 package should reach for this one, so that no judged contest goes unrecorded.

 THE RECORD IS AWAITED, not fired and forgotten. A contest whose write is
 still in flight when the process exits is a contest missing from the ledger,
 and the whole point is that a later reader finds the evidence. The cost is one
 small file write against a round that just spent several model calls.

 @param request - exactly what {@link decideBestCandidate} takes, whose
 `stageSettings` also name where the contest is recorded

 @returns Whatever it decided, unchanged

 @throws {@link ProducerRosterError} when a judge
 appears twice on the roster

 @example
 ```ts
 const outcome = await selectBestCandidate({ client, candidates, judgeModelIds, ..., stageSettings, },);
 ```
 */
export async function selectBestCandidate<ValueT,>(
  request: Parameters<typeof decideBestCandidate<ValueT>>[0],
): Promise<SelectionOutcome<ValueT>> {
  /**
   What the judges decided.
   */
  const outcome = await decideBestCandidate<ValueT>(request,);

  await recordContest({
    task: request.task,
    candidates: request.candidates,
    ballots: outcome.ballots,
    selectedIndex: (outcome.kind === 'selected') ? outcome.selectedIndex : 'declined',
    l: request.l,
    contestLedger: request
      .stageSettings
      .contestLedger,
  },);

  return outcome;
}

//endregion Candidate selection recording
