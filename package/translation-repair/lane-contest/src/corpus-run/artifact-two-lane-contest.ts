import {
  LANE_CONTEST_QUORUM,
  type LaneContestOutcome,
  settleArchiveBallots,
} from '../lane-contest-stage.ts';
import {
  type LaneContestEligibility,
  settleEligibleLaneContestBallots,
} from '../lane-contest-eligibility.ts';
import type { LaneContestBallot, } from '../lane-contest-wire.ts';
import type { TenseCandidateGroup, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { ArtifactComparisonRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';

//region Lane contest record
// What the roster settled at each slice where the two lanes offer different
// wording, as the settled artifact records it.
//
// WHY THE VERDICT IS A KIND RATHER THAN THE STAGE`S RAW `choice`. The contest
// answers `neither` in two unrelated situations: a roster that heard enough
// voices and still backed no candidate, and a roster too few of whose voices
// arrived to settle anything at all. Recording the raw answer would merge them,
// so a reader asking how often the contest refused both lanes would be counting
// silence as refusal. They are kept apart here, and an unsettled slice lands on
// the same footing as a win, ballots included, for the same reason the repair
// lane records its declined rounds.

/**
 What the roster settled at one contested slice.

 @example
 ```ts
 const verdict: ArtifactContestVerdict = { kind: 'lane-won', lane: 'repair', };
 ```
 */
export type ArtifactContestVerdict =
  | {
    /**
     One lane outpolled the other with enough voices behind it.
     */
    readonly kind: 'lane-won';

    /**
     Lane those voices backed.
     */
    readonly lane: 'repair' | 'translate';
  }
    | {
    /**
     Enough voices were heard and none of the lanes carried them, which
     includes a tie: the stage ships nothing on one by design.
     */
      readonly kind: 'settled-neither';

    /**
     What the roster made of the archive rendering, when it made anything.

     OMITTED RATHER THAN RECORDED AS `unjudged`, so that a slice nobody
     judged the archive at is byte-identical to one settled before the
     question was ever asked. Every artifact written before today parses
     unchanged because of this, and the exact-keys guard needs no exception.
     */
      readonly archive?: 'endorsed' | 'declined';
    }
    | {
    /**
     Too few voices arrived to settle anything, so nothing was decided here
     and the slice is not evidence about either lane.
     */
      readonly kind: 'quorum-not-met';
    }
    | {
    /**
     Only one lane had a passage to work on here, so the lane named ships by
     the rule `soleLaneOf` states and no roster was asked.
     */
      readonly kind: 'sole-lane';

    /**
     Lane that applies, which is the one the other lane marked
     `not-applicable`.
     */
      readonly lane: 'repair' | 'translate';
    };

/**
 One contested slice as the roster left it.

 @example
 ```ts
 const slice: ArtifactContestSlice = { sliceIndex: 0, verdict: { kind: 'settled-neither', }, ballots: [], usable: 0, };
 ```
 */
export type ArtifactContestSlice = {
  /**
   Slice both lanes name it by, matching the comparison row it answers.
   */
  readonly sliceIndex: number;

  /**
   What the roster settled here.
   */
  readonly verdict: ArtifactContestVerdict;

  /**
   Every usable ballot, for the audit trail rather than for the verdict.

   KEPT ON EVERY VERDICT, including the two that ship nothing. A reader asking
   why a slice shipped neither lane needs the reasons the judges gave, and a
   record that carried ballots only for wins would answer that question with
   silence exactly where it is being asked.
   */
  readonly ballots: readonly LaneContestBallot[];

  /**
   Voices whose answer arrived and could be read as a ballot, which is what
   the quorum is measured against.
   */
  readonly usable: number;

  /**
   Deterministic source-backed candidate admission, present on syntax slices.

   Raw ballots remain unchanged. Reader uses this record to exclude votes for
   candidates that cannot cross publication boundary, then re-derives verdict.
   */
  readonly eligibility?: LaneContestEligibility;

  /**
   Tense candidates the judges were shown for this slice's texts, present only
   where the entry records a death, the slice went to the roster and a text has
   a candidate. A finding for the audit trail: it never counts toward a verdict,
   a flawed archive or a run-off. Part of artifact generation fifteen alone, so
   a reader of an older record refuses the key.
   */
  readonly tenseCandidates?: readonly TenseCandidateGroup[];

  /**
   Present when no round was opened because fewer seats were reachable than the voices a verdict needs, absent when a
   roster was asked and heard nobody. Both read `quorum-not-met` with no ballot; this is what tells them apart. Part of
   artifact generation twenty-one, so a reader of an older record refuses the key.
   */
  readonly benchBelowFloor?: true;
};

/**
 Reads the artifact`s verdict out of what the contest stage returned.

/**
 Builds the verdict for a slice whose roster backed no candidate.

 DERIVED FROM THE BALLOTS RATHER THAN CARRIED ON THE OUTCOME, so that the
 writer and the artifact reader reach the same answer by running the same
 rule over the same stored ballots. Nothing has to be kept in step, because
 there is only one value.

 @param ballots - usable ballots for this slice

 @returns Verdict naming the archive outcome, or omitting it when unjudged

 @example
 ```ts
 const verdict = settledNeitherVerdict({ ballots, },);
 ```
 */
function settledNeitherVerdict(
  {
    ballots,
    eligibility,
  }: {
    readonly ballots: readonly LaneContestBallot[];
    readonly eligibility?: LaneContestEligibility;
  },
): ArtifactContestVerdict {
  if (eligibility?.archive === 'ineligible')
    return { kind: 'settled-neither', };
  /**
   What the roster made of the archive at this slice.
   */
  const archive = settleArchiveBallots({ ballots, },);
  return (archive === 'unjudged')
    ? { kind: 'settled-neither', }
    : {
      kind: 'settled-neither',
      archive,
    };
}

/**
 Records one contested slice, verdict and ballots together.

 @param sliceIndex - slice this answers

 @param outcome - what the roster settled

 @param eligibility - deterministic candidate admission for syntax slice

 @returns Record for one contested slice

 @example
 ```ts
 const slice = describeContestSlice({ sliceIndex: 0, outcome, },);
 ```
 */
export function describeContestSlice(
  {
    sliceIndex,
    outcome,
    eligibility,
  }: {
    readonly sliceIndex: number;
    readonly outcome: LaneContestOutcome;
    readonly eligibility?: LaneContestEligibility;
  },
): ArtifactContestSlice {
  /**
   Roster choice after votes for inadmissible candidates are excluded.
   */
  const choice = (eligibility === undefined)
    ? outcome.choice
    : settleEligibleLaneContestBallots({
      ballots: outcome.ballots,
      eligibility,
    },);
  /**
   What the roster settled, read against the same quorum the stage applied
   rather than against a copy of the number, so the two cannot drift.
   */
  const verdict: ArtifactContestVerdict = (outcome.usable < LANE_CONTEST_QUORUM)
    ? { kind: 'quorum-not-met', }
    : ((choice === 'neither')
      ? settledNeitherVerdict({
        ballots: outcome.ballots,
        ...((eligibility === undefined) ? {} : { eligibility, }),
      },)
      : {
        kind: 'lane-won',
        lane: choice,
      });
  return {
    sliceIndex,
    verdict,
    ballots: outcome.ballots,
    usable: outcome.usable,
    ...((eligibility === undefined) ? {} : { eligibility, }),
    ...((outcome.benchBelowFloor === true) ? { benchBelowFloor: true as const, } : {}),
  };
}

/**
 Which lane a slice applies to alone, or that the rule names neither.

 @example
 ```ts
 const reading: SoleLaneReading = { kind: 'one-lane', lane: 'translate', };
 ```
 */
export type SoleLaneReading =
  | {
    /**
     Exactly one lane had a passage here; the other recorded `not-applicable`.
     */
    readonly kind: 'one-lane';

    /**
     Lane that applies.
     */
    readonly lane: 'repair' | 'translate';
  }
    | {
    /**
     Both lanes or neither had a passage, so the roster is the question.
     */
      readonly kind: 'both-lanes';
    };

/**
 Names the one lane a slice applies to, when the other lane recorded
 `not-applicable` there.

 THE RECORDED OUTCOME DECIDES, NOT THE TEXT. A lane that tried and filled
 nothing also ships nothing, and a roster is still the right question for a
 slice where both lanes had a passage to work on. Only `not-applicable` says
 the lane had no passage at all (`repair-slice-settle.ts`, an insertion chunk).

 NEITHER OR BOTH NOT APPLICABLE ANSWERS `both-lanes` rather than throwing,
 because a reader hands this the comparison rows of a file on disk. Such a row
 never reaches the contest (`contestEligibleIndexes` needs differing texts), so
 the reader's coverage check refuses a record that names it.

 @param row - comparison row of one slice

 @returns Lane that applies alone, or `both-lanes` when neither or both lanes
 are `not-applicable`

 @example
 ```ts
 const sole = soleLaneOf({ row, },);
 ```
 */
export function soleLaneOf(
  { row, }: { readonly row: ArtifactComparisonRow; },
): SoleLaneReading {
  /**
   What the repair lane recorded at this slice.
   */
  const { kind: repairKind, } = row.repairOutcome;

  /**
   What the translate lane recorded at this slice.
   */
  const { kind: translateKind, } = row.translateOutcome;
  if ((repairKind === 'not-applicable') === (translateKind === 'not-applicable'))
    return { kind: 'both-lanes', };
  if (repairKind === 'not-applicable')
    return {
      kind: 'one-lane',
      lane: 'translate',
    };
  return {
    kind: 'one-lane',
    lane: 'repair',
  };
}

/**
 Records one slice a single lane applies to, which no roster was asked about.

 BALLOTS AND USABLE ARE EMPTY AND ZERO rather than absent, so the record keeps
 the exact key set every contested slice carries and a reader needs no second
 shape. Zero usable voices is also what keeps such a slice out of every
 consolidation rule that counts ballots.

 @param sliceIndex - slice this answers

 @param lane - lane that applies, from {@link soleLaneOf}

 @returns Record naming the applicable lane with no ballots behind it

 @example
 ```ts
 const slice = describeSoleLaneSlice({ sliceIndex: 4, lane: 'translate', },);
 ```
 */
export function describeSoleLaneSlice(
  {
    sliceIndex,
    lane,
  }: {
    readonly sliceIndex: number;
    readonly lane: 'repair' | 'translate';
  },
): ArtifactContestSlice {
  return {
    sliceIndex,
    verdict: {
      kind: 'sole-lane',
      lane,
    },
    ballots: [],
    usable: 0,
  };
}

/**
 Names the slices a contest is meaningful at, which are those where the two
 lanes left different wording.

 THE TEXTS RATHER THAN THE VERDICT NAME. `both-differ`, `repair-only` and
 `translate-only` are exactly the verdicts whose two lane texts disagree, and
 the other three are exactly those where they match, so asking the texts asks
 the real question and stays right if a verdict is ever added.

 @param comparison - rows the reader recomputed from both ledgers

 @returns Slice indexes a contest may answer, in row order

 @example
 ```ts
 const eligible = contestEligibleIndexes({ comparison, },);
 ```
 */
export function contestEligibleIndexes(
  { comparison, }: { readonly comparison: readonly ArtifactComparisonRow[]; },
): readonly number[] {
  return comparison
    .filter(function lanesDiffer(row: ArtifactComparisonRow,): boolean {
      return row.repairText !== row.translateText;
    },)
    .map(function nameIt(row: ArtifactComparisonRow,): number {
      return row.sliceIndex;
    },);
}

/**
 Which lane ships, as the settled artifact records it.

 BOTH KINDS STAY LEGAL. An artifact written before the contest ran carries the
 pending kind, and dropping it would make every settled artifact unreadable to
 say something none of them claims. A reader that cannot tell "nobody has
 asked yet" from "the roster answered" is the defect class this generation
 exists to end.

 @example
 ```ts
 const selection: ArtifactLaneSelection = { kind: 'pending-human-decision', };
 ```
 */
export type ArtifactLaneSelection =
  | {
    /**
     The contest has not run over this entry.
     */
    readonly kind: 'pending-human-decision';
  }
    | {
    /**
     Every slice where the lanes offer different wording was settled: by the
     roster, or by rule where one lane alone applies (`sole-lane`).
     */
      readonly kind: 'contested';

    /**
     One record per eligible slice, in comparison-row order.
     */
      readonly slices: readonly ArtifactContestSlice[];
    };

//endregion Lane contest record
