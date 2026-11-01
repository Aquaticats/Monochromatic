import type {
  Candidate,
  CandidateProducer,
  CandidateWeight,
  SelectionBallot,
  SelectionDisposition,
  SelectionOutcome,
  SelectionTally,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Repair judged round
// The repair lane's ballots, kept so this lane's reasoning can be READ.
//
// The translate lane has recorded every ballot with its reason since it was
// built, in `TranslateStageResult.ballots` beside the slate that makes a
// position joinable. The repair lane recorded none: `selectPerEnvelope`,
// `selectChunkPatch` and the refinement stage each threw `SelectionOutcome`'s
// `ballots`, `perCandidate` and `selectedIndex` away and kept only counts. So
// the one stage that can silently delete a person's name from a translation
// was also the one stage whose reasoning left no trace, and the defect behind
// `declared-name-survival.ts` could only be found by running a live probe
// against the real sheet.
//
// DECLINES ARE RECORDED TOO, on the same footing as wins. A round where the
// judges refused every proposal is evidence about the judges, and that probe's
// third round was exactly a decline; keeping only `kind: 'selected'` would drop
// the half of the record that says the panel could not agree.
//
// THE INCUMBENT IS NOT ON THESE BALLOTS. In the translate lane the archive text
// is a candidate and its slate entry carries an `origin`, because judges decide
// whether to keep it. Here that comparison is deterministic, made against
// checker verdicts in `select-candidate.ts`, so no judged round in this lane
// sees the unchanged text. `RepairSlateEntry.producer` still carries the kind,
// so a reader never has to assume it.

/**
 Every judged stage of the repair lane, in the order a pass runs them.

 THE TYPE IS DERIVED FROM THIS LIST, so a stage cannot be added to the type
 without being listed here, and the artifact reader and the standing readers
 that import this list cannot fall behind it.
 */
export const REPAIR_ROUND_STAGES = [
  'envelope',
  'chunk-patch',
  'refine',
] as const;

/**
 Which of the repair lane's judged stages a round belongs to.

 `envelope` and `chunk-patch` are the editor ensemble's two rounds: one per
 editable envelope, then one over whole-chunk patches. `refine` is the
 naturalness pass, which re-decides text the accuracy verdict already
 accepted and is therefore the stage most able to undo a repair.

 @example
 ```ts
 const stage: RepairRoundStage = 'envelope';
 ```
 */
export type RepairRoundStage = typeof REPAIR_ROUND_STAGES[number];

/**
 Envelope identifier standing for "this round decided the whole chunk".

 A string rather than a nullish value so every recorded round names its scope
 the same way, and so the value cannot be confused with an envelope that was
 never stamped.

 @example
 ```ts
 const scope = CHUNK_SCOPE_ENVELOPE;
 ```
 */
export const CHUNK_SCOPE_ENVELOPE = 'chunk';

/**
 One position on a repair ballot, with everything needed to read a vote for
 it.

 @example
 ```ts
 const entry: RepairSlateEntry = { index: 1, rendered, hash, producer, };
 ```
 */
export type RepairSlateEntry = {
  /**
   One-based position, exactly as judges were shown it.
   */
  readonly index: number;

  /**
   Text judges compared at that position, which is what the sheet displayed
   rather than any internal value: an envelope candidate is an operation, and
   what went on the sheet was its replacement text.
   */
  readonly rendered: string;

  /**
   Digest of that text, so a stored round can be checked against a rebuilt
   chunk without carrying every candidate twice.
   */
  readonly hash: string;

  /**
   Who produced it, including every model that reproduced it exactly.
   */
  readonly producer: CandidateProducer;
};

/**
 One judged round of the repair lane, win or refusal.

 @example
 ```ts
 const round: RepairJudgedRound = describeJudgedRound({ stage: 'refine', envelopeId, candidates, outcome, },);
 ```
 */
export type RepairJudgedRound =
  | {
    readonly kind: 'selected';

    /**
     Stage that ran this round.
     */
    readonly stage: RepairRoundStage;

    /**
     Envelope this round decided, or {@link CHUNK_SCOPE_ENVELOPE} when it
     decided the whole chunk. Names the same identifier `RepairRegion` does,
     so a round joins to the region it produced without parsing prose.
     */
    readonly envelopeId: string;

    /**
     Candidates in the order judges saw them.
     */
    readonly slate: readonly RepairSlateEntry[];

    /**
     Every ballot cast, reason verbatim.
     */
    readonly ballots: readonly SelectionBallot[];

    /**
     What this round counted.
     */
    readonly tally: SelectionTally;

    /**
     What every position drew, whether or not it won.
     */
    readonly perCandidate: readonly CandidateWeight[];

    /**
     Position that won.
     */
    readonly selectedIndex: number;

    /**
     Summed weight the winner drew.
     */
    readonly voteWeight: number;
  }
    | {
      readonly kind: 'declined';

    /**
     {@inheritDoc RepairJudgedRound.stage}
     */
      readonly stage: RepairRoundStage;

    /**
     {@inheritDoc RepairJudgedRound.envelopeId}
     */
      readonly envelopeId: string;

    /**
     {@inheritDoc RepairJudgedRound.slate}
     */
      readonly slate: readonly RepairSlateEntry[];

    /**
     {@inheritDoc RepairJudgedRound.ballots}
     */
      readonly ballots: readonly SelectionBallot[];

    /**
     {@inheritDoc RepairJudgedRound.tally}
     */
      readonly tally: SelectionTally;

    /**
     {@inheritDoc RepairJudgedRound.perCandidate}
     */
      readonly perCandidate: readonly CandidateWeight[];

    /**
     Why nothing was selected, in scorecard-stable wording.
     */
      readonly reason: string;

    /**
     Whether judges could not agree or agreed to reject.
     */
      readonly disposition: SelectionDisposition;
    }
    | {
    /**
     Sole distinct proposal, adopted without a vote: nothing to compare it
     against, so no judge was asked and no ballot exists.
     */
      readonly kind: 'adopted';

    /**
     Stage that adopted it.
     */
      readonly stage: RepairRoundStage;

    /**
     Envelope it was the one proposal for.
     */
      readonly envelopeId: string;

    /**
     The one entry, so the winner's authors are read the way a judged
     round's are.
     */
      readonly slate: readonly RepairSlateEntry[];

    /**
     Empty: no judge was asked. Carried so every reader of a round finds
     the vote fields in every kind.
     */
      readonly ballots: readonly SelectionBallot[];

    /**
     All zero, for the same reason.
     */
      readonly tally: SelectionTally;

    /**
     Empty: nothing was drawn.
     */
      readonly perCandidate: readonly CandidateWeight[];

    /**
     Index of that entry, always the first.
     */
      readonly selectedIndex: number;

    /**
     Why no vote was held.
     */
      readonly reason: string;
    };

/**
 Records the slate a round was judged on.

 @param candidates - candidates in judged order

 @returns One entry per position

 @example
 ```ts
 const slate = describeRepairSlate({ candidates: proposals, },);
 ```
 */
export function describeRepairSlate<ValueT,>(
  {
    candidates,
  }: {
    readonly candidates: readonly Candidate<ValueT>[];
  },
): readonly RepairSlateEntry[] {
  return candidates.map(function toEntry(
    candidate,
    position,
  ): RepairSlateEntry {
    return {
      index: position + 1,
      rendered: candidate.rendered,
      hash: hashContent({ content: candidate.rendered, },),
      producer: candidate.producer,
    };
  },);
}

/**
 Turns one selection round into the record an artifact carries.

 @param stage - which judged stage ran it

 @param envelopeId - envelope decided, or {@link CHUNK_SCOPE_ENVELOPE}

 @param candidates - candidates in judged order, for joining ballot positions

 @param outcome - what selection returned, win or refusal

 @returns Round record carrying every ballot either way

 @example
 ```ts
 rounds.push(describeJudgedRound({ stage: 'envelope', envelopeId, candidates, outcome, },),);
 ```
 */
export function describeJudgedRound<ValueT,>(
  {
    stage,
    envelopeId,
    candidates,
    outcome,
  }: {
    readonly stage: RepairRoundStage;
    readonly envelopeId: string;
    readonly candidates: readonly Candidate<ValueT>[];
    readonly outcome: SelectionOutcome<ValueT>;
  },
): RepairJudgedRound {
  /**
   Slate shared by both branches, since judges saw one list either way.
   */
  const slate = describeRepairSlate({ candidates, },);
  if (outcome.kind === 'declined') {
    return {
      kind: 'declined',
      stage,
      envelopeId,
      slate,
      ballots: outcome.ballots,
      tally: outcome.tally,
      perCandidate: outcome.perCandidate,
      reason: outcome.reason,
      disposition: outcome.disposition,
    };
  }
  return {
    kind: 'selected',
    stage,
    envelopeId,
    slate,
    ballots: outcome.ballots,
    tally: outcome.tally,
    perCandidate: outcome.perCandidate,
    selectedIndex: outcome.selectedIndex,
    voteWeight: outcome.voteWeight,
  };
}

/**
 Records an envelope's sole proposal being adopted without a vote.

 RECORDED AT ALL because the sole path was found pushing no round, so
 `issue-authors` read no author for any issue such an envelope served: a
 checker who wrote that text voted on it at full weight and the artifact said
 nobody wrote it. The record is the same slate shape a judged round carries,
 minus everything a vote would have produced.

 @param stage - stage adopting it

 @param envelopeId - envelope it was the one proposal for

 @param candidate - the proposal

 @returns Round record naming the adopted candidate and its authors

 @example
 ```ts
 rounds.push(describeAdoptedRound({ stage: 'envelope', envelopeId, candidate: sole, },),);
 ```
 */
export function describeAdoptedRound<ValueT,>(
  {
    stage,
    envelopeId,
    candidate,
  }: {
    readonly stage: RepairRoundStage;
    readonly envelopeId: string;
    readonly candidate: Candidate<ValueT>;
  },
): RepairJudgedRound {
  return {
    kind: 'adopted',
    stage,
    envelopeId,
    slate: describeRepairSlate({ candidates: [candidate,], },),
    ballots: [],
    tally: {
      judgesAvailable: 0,
      ballots: 0,
      abstentions: 0,
      selfVotes: 0,
    },
    perCandidate: [],
    selectedIndex: 1,
    reason: 'sole proposal, adopted without a vote',
  };
}

//endregion Repair judged round
