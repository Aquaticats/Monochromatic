import {
  type Candidate,
  producerModelIds,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { ProducerRosterError, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Selection roster checks
// The two readings of a selection round's roster that run before any judge is
// asked, split out of `candidate-select.ts` at its line budget.

/**
 Refuses a judge listed twice, before any model call.

 A repeated id is one model given two exchanges and two ballots, which is
 enough to count as two answering judges alone: exactly the single-model
 control the ensemble exists to prevent, arriving as a roster typo rather
 than as a policy change.

 Refused HERE as well as in `assertJudgeableProducerRoster`, because that
 guard runs at STAGE entry while `selectPerEnvelope` and `selectChunkPatch`
 are exported and reachable without one. Thrown rather than deduplicated,
 since a caller that passed a repeat believes it has more judges than it
 has, and silently collapsing the roster answers a question it did not ask.

 Before the fan-out rather than at the count, so a roster fault costs no
 model calls.

 @param judges - whole roster the round seats

 @param candidates - proposals, whose producers the fault names

 @throws {@link ProducerRosterError} when a judge appears twice on the roster

 @example
 ```ts
 assertDistinctJudges({ judges, candidates, },);
 ```
 */
export function assertDistinctJudges<ValueT,>(
  {
    judges,
    candidates,
  }: {
    readonly judges: readonly RosterModelId[];
    readonly candidates: readonly Candidate<ValueT>[];
  },
): void {
  /**
   Judges keyed for repeat detection.
   */
  const distinctJudges = new Set(judges,);
  if (distinctJudges.size === judges.length)
    return;
  throw new ProducerRosterError({
    producerModelIds: candidates.flatMap(function toStakeholders(
      candidate,
    ): readonly RosterModelId[] {
      return producerModelIds(candidate.producer,);
    },),
    judgeModelIds: judges,
    role: 'producer',
    fault: 'a judge is listed more than once, which would let one model count as two answering judges',
  },);
}

/**
 Models with a stake in each one-based candidate index, for telling a
 self-vote from an ordinary one.

 @param candidates - proposals in caller-fixed order

 @returns Stakeholders keyed by one-based position

 @example
 ```ts
 const stakesByIndex = stakesByIndexOf({ candidates, },);
 ```
 */
export function stakesByIndexOf<ValueT,>(
  { candidates, }: { readonly candidates: readonly Candidate<ValueT>[]; },
): ReadonlyMap<number, ReadonlySet<RosterModelId>> {
  return new Map(
    candidates.map(function toStake(
      candidate,
      index,
    ): readonly [
      number,
      ReadonlySet<RosterModelId>,
    ] {
      return [
        index + 1,
        new Set(producerModelIds(candidate.producer,),),
      ];
    },),
  );
}

//endregion Selection roster checks
