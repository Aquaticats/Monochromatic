import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import { selectionFanOut, } from './candidate-select-fanout.ts';
import {
  assertDistinctJudges,
  stakesByIndexOf,
} from './candidate-select-roster.ts';
import {
  selectionBench,
  shortBenchFinding,
} from './candidate-select-minimum.ts';
import {
  buildCandidateSelectMessages,
  CANDIDATE_SELECT_RESPONSE_FORMAT,
  countBallots,
  isCandidateBallotAsSent,
  selectDecision,
  type SelectEvidence,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  type Candidate,
  describeProducer,
  MIN_SELECTION_BALLOTS,
  type SelectionOutcome,
  type SelectionTally,
  SELF_VOTE_WEIGHT,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type FanOutMode,
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Candidate selection stage
// Shared by every stage that generates text rather than verdicts: the ensemble
// editor, the naturalness refinement lane and the translate lane all propose N
// candidates and need one chosen.
//
// Two invariants make the ensemble worth having, and both are enforced here
// rather than trusted to callers:
//
// - A model may judge its own work, and its ballot for its own work counts for
//   LESS. Producers used to be removed from the roster outright; the user
//   replaced that with a discount on 2026-08-15, because these models have
//   different blind spots and dropping three of six judges to keep the rest
//   disinterested threw away readings nothing else supplies. A producer reading
//   its own text is a weaker instrument than a disinterested one, not a
//   worthless one.
// - A tie or an empty judge roster DECLINES. Declining returns the caller's
//   fallback, which is text the pipeline already trusts, so the conservative
//   outcome is the default whenever the ensemble fails to agree.
//
// The discount decides which candidate leads and nothing else: a winner needs
// two answering judges, a self-voting judge is one of them, and no summed
// weight is required (the owner's rulings of 2026-10-25, see `SELF_VOTE_WEIGHT`).
// Self-preference is not assumed away, so every self-vote is recorded by name,
// weighed on the ballot, and counted in the tally.
//
// Callers pass their WHOLE roster as `judgeModelIds`. Hand-partitioning a judge
// list outside is how a caller ends up with an empty roster it cannot see.

/**
 Runs one selection round: judges compare the anonymized set and name a
 winner, and anything short of a clear plurality declines. Every judge on the
 roster is seated, with a ballot for its own work discounted.

 Every candidate handed in is judged, including a lone one. A caller that has
 already deduplicated its set and knows one candidate survived may short
 circuit before calling; this function does not assume that on its behalf,
 because a single candidate arriving here is generally the caller's only
 proposal rather than a proven consensus.

 @param client - injected model client

 @param candidates - proposals in caller-fixed order

 @param judgeModelIds - whole roster, producers included

 @param task - one sentence naming what candidates attempt

 @param criteria - ordered decision rules, most important first

 @param evidence - source and baseline material judges compare against

 @param declineConsequence - what this caller does when every judge declines,
 omitted where the default holds

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param sourceText - original the candidates render, when the caller has
 it, so the sheet names a candidate lacking a community rendering

 @param l - logger of the calling stage

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Winner with the ballot weight it drew, or a decline carrying its
 reason; either way the round's tally and every ballot cast

 @throws {@link ProducerRosterError} when a judge
 appears twice on the roster, which would let one model count as two
 answering judges

 @example
 ```ts
 const outcome = await decideBestCandidate({ client, candidates, judgeModelIds, ... },);
 ```
 */
export async function decideBestCandidate<ValueT,>(
  {
    client,
    candidates,
    judgeModelIds,
    task,
    criteria,
    evidence,
    declineConsequence,
    signal,
    perCallTimeoutMs,
    l,
    fanOut,
    sourceText,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly candidates: readonly Candidate<ValueT>[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly task: string;
    readonly criteria: readonly string[];
    readonly evidence: readonly SelectEvidence[];
    readonly declineConsequence?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly sourceText?: string;
    readonly stageSettings: StageSettings;
  }>,
): Promise<SelectionOutcome<ValueT>> {
  /**
   Logger tagged with this stage.
   */
  const sl = tagged({
    tag: decideBestCandidate.name,
    l,
  },);

  /**
   Tally of a round that never reached the judges.
   */
  const emptyTally: SelectionTally = {
    judgesAvailable: 0,
    ballots: 0,
    abstentions: 0,
    selfVotes: 0,
  };
  if (candidates.length === 0)
    return {
      kind: 'declined',
      reason: 'no candidates proposed',
      disposition: 'rejection',
      tally: emptyTally,
      findings: [],
      ballots: [],
      perCandidate: [],
    };

  /**
   Judges this round seats: the whole roster, producers included.
   */
  const judges = judgeModelIds;
  if (judges.length === 0) {
    sl.warn('no judge roster was passed; declining rather than selecting unexamined text',);
    return {
      kind: 'declined',
      reason: 'no judge available',
      disposition: 'rejection',
      tally: emptyTally,
      findings: [],
      ballots: [],
      perCandidate: [],
    };
  }

  assertDistinctJudges({
    judges,
    candidates,
  },);

  /**
   Models with a stake in each one-based candidate index, for telling a
   self-vote from an ordinary one.
   */
  const stakesByIndex = stakesByIndexOf({ candidates, },);

  /**
   Ballots from the judges that answered.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: judges,
    messages: buildCandidateSelectMessages({
      task,
      criteria,
      evidence,
      rendered: candidates.map(function toRendered(candidate,) {
        return candidate.rendered;
      },),
      ...((declineConsequence === undefined) ? {} : { declineConsequence, }),
      // Conditional spread keeps the original absent instead of undefined.
      ...((sourceText === undefined) ? {} : { sourceText, }),
    },),
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: CANDIDATE_SELECT_RESPONSE_FORMAT,
    validate: isCandidateBallotAsSent,
    // THE SAME BALLOT AS A TYPED QUESTION, for a decision seat on the bench
    // (2026-09-18); a chat seat never sees it.
    decision: selectDecision({
      task,
      criteria,
      evidence,
      rendered: candidates.map(function toRendered(candidate,) {
        return candidate.rendered;
      },),
      ...((declineConsequence === undefined) ? {} : { declineConsequence, }),
      ...((sourceText === undefined) ? {} : { sourceText, }),
    },),
    stage: 'select',
    l,
    fanOut: selectionFanOut({
      judgeCount: judges.length,
      // Conditional spread keeps the request absent instead of undefined.
      ...((fanOut === undefined) ? {} : { requested: fanOut, }),
    },),
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  /**
   Every ballot as cast and weighed, what each candidate drew, the ranking
   and the abstention and self-vote counts (`candidate-select-count.ts`).
   */
  const {
    ballots,
    perCandidate,
    ranked,
    abstained,
    self,
  } = countBallots({
    voices: gather.voices,
    stakesByIndex,
    candidateCount: candidates.length,
    l: sl,
  },);

  /**
   Ballots a judge cast for its own work, named so the rate is readable from
   findings as well as from the tally.
   */
  const selfVotes = ballots.filter(function isSelfVote(ballot,): boolean {
    return ballot.weight === SELF_VOTE_WEIGHT;
  },);

  /**
   Seats a wet provider could serve (the owner's decision of 2026-09-09,
   `candidate-select-minimum.ts`), read for the findings and the log only: the
   winner needs two ballots whatever the bench.
   */
  const bench = selectionBench({
    benchSize: judges.length,
    unreachable: gather.unreachable
      .size,
  },);
  if (bench.short) {
    sl.warn(
      `bench short of quorum: seats reachable ${String(bench.reachable,)} of ${String(judges.length,)} against a `
        + `quorum of ${String(bench.quorum,)}; the winner needs at least ${String(MIN_SELECTION_BALLOTS,)} ballots`,
    );
  }

  /**
   Findings every exit past the fan-out carries.
   */
  const roundFindings: readonly string[] = [
    ...gather.findings,
    ...(bench.short
      ? [shortBenchFinding({
        bench,
        benchSize: judges.length,
      },),]
      : []),
    ...selfVotes.map(function toSelfVoteFinding(ballot,): string {
      return `select-self-vote (${ballot.modelId})`;
    },),
  ];

  /**
   What this round counted, reported whichever way it ends.
   */
  const counted: SelectionTally = {
    judgesAvailable: judges.length,
    ballots: gather.voices
      .length,
    abstentions: abstained,
    selfVotes: self,
  };

  /**
   Leading entry, absent when every judge abstained.
   */
  const [leader,] = ranked;
  if (leader === undefined) {
    sl.info(
      `every judge declined (abstentions: ${String(abstained,)}); keeping the fallback`,
    );
    return {
      kind: 'declined',
      reason: 'every judge declined',
      disposition: 'rejection',
      tally: counted,
      findings: roundFindings,
      ballots,
      perCandidate,
    };
  }

  /**
   Runner-up's drawn weight, zero when only one candidate drew any.
   */
  const runnerUpWeight = ranked[1]?.[1] ?? 0;
  if (leader[1] === runnerUpWeight) {
    sl.info(`judges tied at weight ${String(leader[1],)}; keeping the fallback`,);
    return {
      kind: 'declined',
      reason: 'judges tied',
      disposition: 'indecision',
      tally: counted,
      findings: roundFindings,
      ballots,
      perCandidate,
    };
  }
  /**
   What the leader drew. ALWAYS FOUND: `countBallots` ranks only ballots of
   nonzero weight, which its `usable` test gives only a `best` from 1 to the
   candidate count, and `countCandidateWeights` gives each such index a row.
   */
  const leaderDrawn = nonNullishOrThrow(
    perCandidate.find(function isLeader(drawn,): boolean {
      return drawn.index === leader[0];
    },),
  );

  /**
   Ballots naming the leader, self-votes included.
   */
  const leaderBallots = leaderDrawn.ballots;

  /**
   Winning candidate, indexed back from the one-based ballot. ALWAYS FOUND:
   `countBallots` ranks only a `best` from 1 to the candidate count, the
   length of `candidates`.
   */
  const winner = nonNullishOrThrow(candidates[leader[0] - 1],);

  if (leaderBallots < MIN_SELECTION_BALLOTS) {
    // A PLURALITY OF ONE IS NOT AGREEMENT, whatever it weighs. Lost voices and
    // abstentions can leave a single judge as the only one who named anything,
    // and letting that judge decide would put one model back in control of the
    // stage (owner, 2026-09-09). This count is the only floor: the minimum
    // vote weight went on 2026-10-25 ("Every weight floor"), so two ballots
    // decide even when they weigh 1.5.
    sl.info(
      `winner was named by ${String(leaderBallots,)} ballot against a floor of ${String(MIN_SELECTION_BALLOTS,)}; `
        + 'keeping the fallback',
    );
    return {
      kind: 'declined',
      reason: 'winner named by one judge alone',
      disposition: 'indecision',
      tally: counted,
      findings: roundFindings,
      ballots,
      perCandidate,
    };
  }

  sl.info(
    `candidate ${String(leader[0],)} from ${describeProducer(winner.producer,)} won `
      + `weight ${String(leader[1],)} (ballots: ${String(counted.ballots,)})`,
  );
  return {
    kind: 'selected',
    value: winner.value,
    producer: winner.producer,
    voteWeight: leader[1],
    selectedIndex: leader[0],
    tally: counted,
    findings: roundFindings,
    ballots,
    perCandidate,
  };
}

//endregion Candidate selection stage
