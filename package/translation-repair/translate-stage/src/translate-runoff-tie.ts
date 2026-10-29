import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type Candidate,
  type CandidateWeight,
  describeProducer,
  MIN_SELECTION_BALLOTS,
  type SelectionOutcome,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { isDecisionSeat, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  rendersAsNothing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type RunoffBasis,
  TranslateAbsenceError,
  type TranslateAbsenceReason,
  type TranslateCandidateValue,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type {
  TranslateDecision,
  TranslateStageResult,
} from './translate-stage-result.ts';

//region Translate run-off tie
// CLASS ONE HUNDRED SEVENTY-FOUR (TianqiChen6669 slice 4, 2026-09-26): the
// archive's rendering failed the class one hundred seventy-two floor, so the
// consolidation slate ran with nothing to keep; two valid renderings, "bore
// the greatest pressure on her frailest body" and "with the frailest body",
// tied 1.5 to 1.5, the run-off over the two tied 1.5 to 1.5 again, and the
// entry stopped INCOMPLETE after 24 minutes. A run-off of two cannot narrow,
// so the class eighty-two loop had nothing left to ask.
//
// THE OWNER'S RULE IS "PREFER THE BEST VALID PROPOSAL, ELSE FAIL THE SLICE AT
// ONCE" (2026-09-04, `doc/decision/translation-repair-ineligible-standing.md`):
// the failure is for a slice with no valid proposal, and here
// every finalist is valid and the judges could not rank them, which is a
// failure to rank rather than a rejection (`candidate-select-model.ts`). So a
// challenge round tied across every candidate it asked about ships one by
// preference: the repair lane's text first, the owner's fallback where the
// archive cannot stand ("Not eligible; fall back to the repair text",
// 2026-09-24), since it is the minimal edit of the human translation; then
// the translate lane's text; then slate order. A tie among some of the
// candidates still narrows through the class eighty-two loop, a first round
// is still challenged, and a round with an incumbent keeps it as before.
//
// THE SAME ORDER SHIPS PAST A DECLINE where the passage has wording that
// cannot ship (owner answer 2026-09-27, "Preference + polish"): a standing
// the deterministic rule refused, or an archive rendering the floor refuses
// or the adjudicators disputed. hulicaijia14 stopped its entry on such a
// slate, declined twice with valid proposals on it. A challenge round
// declined there with nothing left to narrow ships its preferred candidate,
// and the reasons of the written ballots that did not back it travel with the
// record for the consolidation's polish to check against the ORIGINAL; a
// decision seat's distribution is no such reason (ledger B125). A passage
// with no wording at all still raises.
//
// ONLY A CANDIDATE TWO JUDGES NAMED SHIPS THAT WAY SINCE 2026-10-25 (owner
// answer "Two judges must name it"; one judge alone never decides). The order
// had no judge behind it: in XingZ6021 five slices (13, 14, 33, 44 and 84)
// were judged by one reachable seat of seven, which named one candidate or
// none, and the order shipped another. The order now ranks only the
// candidates that drew two ballots, and a round that drew none keeps the
// wording the archive has, which the consolidation's settlement answers by
// keeping the archive (owner, 2026-09-27, "Keep archive, ship"), and the
// translate lane carries to the same end. A tie across every finalist
// (class one hundred seventy-four) is broken by the same fixed order of repair
// lane, translate lane and slate order only among the tied candidates two
// judges named (the lead's ruling of 2026-10-25, settled by the same answer);
// where none of them drew two ballots nothing ships by the order, and the
// caller keeps the wording in place by the path of a declined round, or raises
// the absence where the passage has none.

/**
 How a run-off tie was broken, as the finding names it.
 */
export type TieBasis = 'repair lane' | 'translate lane' | 'slate order';

/**
 What a challenge round's tie comes to: one finalist by preference, or no
 tie across every candidate to break.

 @example
 ```ts
 const tie: RunoffTie<TranslateCandidateValue> = { kind: 'unbroken', };
 ```
 */
export type RunoffTie<ValueT,> =
  | {
    readonly kind: 'broken';

    /**
     One-based position of the chosen finalist, as the judges saw it.
     */
    readonly index: number;

    /**
     The chosen finalist.
     */
    readonly candidate: Candidate<ValueT>;

    /**
     Weight every tied finalist drew.
     */
    readonly weight: number;

    /**
     Which preference chose it.
     */
    readonly basis: TieBasis;
  }
    | {
    /**
     Fewer than two drew weight, or some candidate drew less than the
     leaders, which a further run-off narrows instead, or no tied candidate
     drew two ballots, which nothing ships by preference.
     */
      readonly kind: 'unbroken';
    };

/**
 One tied finalist with its preference.
 */
type RankedFinalist<ValueT,> = {
  readonly index: number;
  readonly candidate: Candidate<ValueT>;
  readonly rank: number;
  readonly basis: TieBasis;
};

/**
 Finding a broken run-off tie records, followed by its basis in parentheses.
 */
export const RUNOFF_TIE_BROKEN_FINDING = 'translate-runoff-tie-broken';

/**
 Finding a slate declined in its challenge round records where it ships a
 candidate two or more judges named, by preference among those, over wording
 that cannot ship, followed by its basis in parentheses (owner, 2026-09-27,
 "Preference + polish", narrowed on 2026-10-25 by "Two judges must name it").
 */
export const SHIPPED_PAST_DECLINE_FINDING = 'translate-slate-declined-shipped-by-preference';

/**
 Finding a slate declined in its challenge round records where no candidate
 drew two judges, so nothing ships past the decline and the wording the
 archive has stays (owner, 2026-10-25, "Two judges must name it").
 */
export const DECLINE_KEEPS_ARCHIVE_FINDING = 'translate-slate-declined-archive-kept';

/**
 Preference of a finalist: the repair lane's text first, the translate
 lane's next, a writer's proposal last.

 @param candidate - finalist the judges tied over

 @returns Rank, lower first, and the basis it names

 @example
 ```ts
 preferenceOf({ candidate, },); // { rank: 0, basis: 'repair lane' }
 ```
 */
function preferenceOf<ValueT,>(
  { candidate, }: { readonly candidate: Candidate<ValueT>; },
): {
  readonly rank: number;
  readonly basis: TieBasis;
} {
  /**
   Who wrote the finalist.
   */
  const { producer, } = candidate;
  if (producer.kind !== 'lane') {
    return {
      rank: 2,
      basis: 'slate order',
    };
  }
  return (producer.lane === 'repair')
    ? {
      rank: 0,
      basis: 'repair lane',
    }
    : {
      rank: 1,
      basis: 'translate lane',
    };
}

/**
 A candidate at its one-based position on the round's sheet.
 */
type PlacedCandidate<ValueT,> = {
  readonly index: number;
  readonly candidate: Candidate<ValueT>;
};

/**
 Candidates ranked by preference, best first, slate order breaking equal
 preference.

 @param finalists - candidates at their positions

 @returns Every candidate with its rank and basis, best first

 @example
 ```ts
 const [chosen,] = rankedByPreference({ finalists, },);
 ```
 */
function rankedByPreference<ValueT,>(
  { finalists, }: { readonly finalists: readonly PlacedCandidate<ValueT>[]; },
): readonly RankedFinalist<ValueT>[] {
  return finalists
    .map(function withPreference(finalist,): RankedFinalist<ValueT> {
      return {
        ...finalist,
        ...preferenceOf({ candidate: finalist.candidate, },),
      };
    },)
    .toSorted(function byPreference(
      left: RankedFinalist<ValueT>,
      right: RankedFinalist<ValueT>,
    ): number {
      return (left.rank === right.rank) ? left.index - right.index : left.rank - right.rank;
    },);
}

/**
 The finalist a challenge round tied across every candidate ships, by
 preference.

 @param candidates - candidates in the order the judges saw them

 @param perCandidate - what each position drew

 @returns Preferred of the tied finalists two or more judges named, or
 `unbroken` where fewer than two drew weight, some candidate drew less than the
 leaders, or no tied finalist drew two ballots

 @example
 ```ts
 const tie = breakRunoffTie({ candidates: rotated, perCandidate: outcome.perCandidate, },);
 ```
 */
export function breakRunoffTie<ValueT,>(
  {
    candidates,
    perCandidate,
  }: {
    readonly candidates: readonly Candidate<ValueT>[];
    readonly perCandidate: readonly CandidateWeight[];
  },
): RunoffTie<ValueT> {
  /**
   Highest weight any candidate drew.
   */
  const top = Math.max(
    0,
    ...perCandidate.map(function weightOf(drawn,): number {
      return drawn.weight;
    },),
  );

  /**
   Candidates at that weight.
   */
  const tied = perCandidate.filter(function atTop(drawn,): boolean {
    return (top > 0) && (drawn.weight === top);
  },);
  if ((tied.length < 2) || (tied.length !== candidates.length))
    return { kind: 'unbroken', };

  /**
   Tied finalists two or more judges named, with their preference, best first,
   slate order breaking equal preference: the preference ranks only what a
   judge pair backed (owner, 2026-10-25, "Two judges must name it").
   */
  const [chosen,] = rankedByPreference({
    finalists: tied
      .filter(function namedByTwoJudges(drawn,): boolean {
        return drawn.ballots >= MIN_SELECTION_BALLOTS;
      },)
      .map(function withCandidate(drawn,): PlacedCandidate<ValueT> {
        return {
          index: drawn.index,
          // The round counts one row per candidate on the slate, so every
          // tied row names a position the slate has.
          candidate: nonNullishOrThrow(candidates[drawn.index - 1],),
        };
      },),
  },);
  // No tied candidate drew two ballots: the order would ship a candidate no
  // judge pair backed, so nothing ships by it and the caller keeps the wording
  // in place, or raises the absence where there is none.
  if (chosen === undefined)
    return { kind: 'unbroken', };
  return {
    kind: 'broken',
    index: chosen.index,
    candidate: chosen.candidate,
    weight: top,
    basis: chosen.basis,
  };
}

/**
 Ships the preferred of the candidates two or more judges named, in a
 challenge round the judges declined over wording that cannot ship (owner,
 2026-09-27, "Preference + polish", narrowed on 2026-10-25 by "Two judges must
 name it"), with the reasons of every written ballot that did not back it as
 objections, which the final polish reads where the round was the
 consolidation slate's (`TranslateStageResult`'s `shippedPastDecline` says
 which reads them). A decision seat's ballot objects to nothing: its reason is
 a distribution. Where no candidate drew two ballots nothing ships past the
 decline: the wording the archive has stays, with
 {@link DECLINE_KEEPS_ARCHIVE_FINDING}.

 @param outcome - declined challenge round

 @param rotated - candidates in the order the judges saw them

 @param keepIncumbent - stage's record shape, whose slate and counts the
 shipped candidate keeps, and whose text stays where none ships

 @param declineFindings - findings the decline reports

 @param declined - decline in the stage's vocabulary, decision of a
 round that keeps the archive wording

 @param l - logger of the judging stage

 @returns Shipped candidate's record, or the record of the archive
 wording staying

 @example
 ```ts
 return shipPreferredPastDecline({ outcome, rotated, keepIncumbent, declineFindings, declined, l, },);
 ```
 */
function shipPreferredPastDecline(
  {
    outcome,
    rotated,
    keepIncumbent,
    declineFindings,
    declined,
    l,
  }: {
    readonly outcome: Extract<SelectionOutcome<TranslateCandidateValue>, { readonly kind: 'declined'; }>;
    readonly rotated: readonly Candidate<TranslateCandidateValue>[];
    readonly keepIncumbent: Omit<TranslateStageResult, 'decision' | 'findings'>;
    readonly declineFindings: readonly string[];
    readonly declined: TranslateDecision;
    readonly l: Logger;
  },
): TranslateStageResult {
  /**
   The preferred of the round's candidates two or more judges named, absent
   where none drew two ballots.
   */
  const [chosen,] = rankedByPreference({
    finalists: rotated
      .map(function placed(
        candidate,
        position,
      ): PlacedCandidate<TranslateCandidateValue> {
        return {
          index: position + 1,
          candidate,
        };
      },)
      .filter(function namedByTwoJudges(finalist,): boolean {
        return outcome.perCandidate
          .some(function drewTwoBallots(drawn,): boolean {
            return (drawn.index === finalist.index) && (drawn.ballots >= MIN_SELECTION_BALLOTS);
          },);
      },),
  },);
  if (chosen === undefined) {
    l.warn(
      `translate stage: challenge round declined (${outcome.disposition}) over wording that cannot ship, and no `
        + `candidate drew ${String(MIN_SELECTION_BALLOTS,)} ${
          wordForCount({
            count: MIN_SELECTION_BALLOTS,
            one: 'ballot',
            many: 'ballots',
          },)
        }; nothing ships past the decline and the wording the archive has stays`,
    );
    return {
      ...keepIncumbent,
      decision: declined,
      tally: outcome.tally,
      ballots: outcome.ballots,
      findings: [
        ...declineFindings,
        DECLINE_KEEPS_ARCHIVE_FINDING,
      ],
      perCandidate: outcome.perCandidate,
    };
  }
  /**
   What the written ballots that did not back the shipped candidate said,
   once each.
   */
  const objections = [
    ...new Set(
      outcome.ballots
        .filter(function backedOther(ballot,): boolean {
          /**
           Who judged, what it named and why.
           */
          const {
            modelId,
            best,
            reason,
          } = ballot;
          // A reason showing a reader nothing objects to nothing, invisible
          // characters `trim()` keeps among it (ledger B40); nor does a decision
          // seat's, which is a distribution over the candidate numbers and no
          // words a refiner could check against the ORIGINAL (ledger B125).
          return (best !== chosen.index)
            && (!rendersAsNothing({ text: reason, },))
            && (!isDecisionSeat({ modelId, },));
        },)
        .map(function reasonOf(ballot,): string {
          return ballot.reason;
        },),
    ),
  ];
  /**
   Weight the shipped candidate drew: its row's weight, which is zero where
   no ballot named it, and zero where the round counted no rows at all
   because it declined before any judge sat.
   */
  const drawn = outcome.perCandidate
    .find(function atChosen(weighed,): boolean {
      return weighed.index === chosen.index;
    },);
  /**
   The shipped candidate's value and author.
   */
  const {
    value,
    producer,
  } = chosen.candidate;
  l.warn(
    `translate stage: challenge round declined (${outcome.disposition}) over wording that cannot ship; shipping `
      + `candidate ${String(chosen.index,)} from ${describeProducer(producer,)} by ${chosen.basis}, with ${
        String(objections.length,)
      } objection(s) recorded`,
  );
  return {
    ...keepIncumbent,
    text: value.text,
    origin: value.origin,
    producer,
    decision: 'judged',
    voteWeight: drawn?.weight ?? 0,
    tally: outcome.tally,
    ballots: outcome.ballots,
    findings: [
      ...declineFindings,
      `${SHIPPED_PAST_DECLINE_FINDING} (${chosen.basis})`,
    ],
    selectedIndex: chosen.index,
    shippedIndex: chosen.index,
    perCandidate: outcome.perCandidate,
    shippedPastDecline: {
      basis: chosen.basis,
      objections,
    },
  };
}

/**
 What a declined round ships where the slice has no incumbent: the preferred
 finalist two or more judges named where a challenge round tied across every
 candidate; the preferred
 of the candidates two or more judges named where the passage has wording that
 cannot ship and the challenge round leaves nothing to narrow (owner,
 2026-09-27, narrowed on 2026-10-25), or the wording the archive has where no
 candidate drew two ballots; else the absence error the retry reads.

 @param challenged - whether this round challenges an earlier decline

 @param outcome - declined round

 @param rotated - candidates in the order the judges saw them

 @param keepIncumbent - stage's record shape, whose slate and counts the
 shipped finalist keeps

 @param declineFindings - findings the decline reports

 @param declined - decline in the stage's vocabulary

 @param finalists - candidates a partial tie backed, for the next run-off

 @param basis - why those are the finalists, given with them

 @param shipPastDecline - whether a challenge round declined with nothing
 left to narrow ships its preferred candidate among those two judges named,
 or keeps the archive wording where none drew two ballots: the passage has
 wording that cannot ship, as opposed to none at all, and the caller has no
 further round to ask

 @param l - logger of the judging stage

 @returns Shipped finalist's record

 @throws {@link TranslateAbsenceError} when no tie across every candidate
 was broken and nothing ships past the decline

 @example
 ```ts
 return settleAbsentDecline({ challenged: true, outcome, rotated, keepIncumbent, declineFindings, declined, shipPastDecline: false, l, },);
 ```
 */
export function settleAbsentDecline(
  {
    challenged,
    outcome,
    rotated,
    keepIncumbent,
    declineFindings,
    declined,
    finalists,
    basis,
    shipPastDecline,
    l,
  }: {
    readonly challenged: boolean;
    readonly outcome: Extract<SelectionOutcome<TranslateCandidateValue>, { readonly kind: 'declined'; }>;
    readonly rotated: readonly Candidate<TranslateCandidateValue>[];
    readonly keepIncumbent: Omit<TranslateStageResult, 'decision' | 'findings'>;
    readonly declineFindings: readonly string[];
    readonly declined: TranslateAbsenceReason;
    readonly finalists?: readonly Candidate<TranslateCandidateValue>[];
    readonly basis?: RunoffBasis;
    readonly shipPastDecline: boolean;
    readonly l: Logger;
  },
): TranslateStageResult {
  /**
   The tie this round comes to, read only on a challenge round's indecision.
   */
  const tie: RunoffTie<TranslateCandidateValue> = (challenged && (outcome.disposition === 'indecision'))
    ? breakRunoffTie({
      candidates: rotated,
      perCandidate: outcome.perCandidate,
    },)
    : { kind: 'unbroken', };
  /**
   Whether a further run-off could narrow this round's question, which the
   retry asks before anything ships by preference.
   */
  const narrows = (finalists !== undefined) && (finalists.length < rotated.length);
  /**
   Whether this decline ships by preference: no tie was broken, the caller
   ships past a decline, this is the challenge round, and no run-off is left.
   */
  const shipsByPreference = (tie.kind === 'unbroken')
    && shipPastDecline
    && challenged
    && (!narrows);
  if (shipsByPreference) {
    return shipPreferredPastDecline({
      outcome,
      rotated,
      keepIncumbent,
      declineFindings,
      declined,
      l,
    },);
  }
  if (tie.kind === 'unbroken') {
    throw new TranslateAbsenceError({
      reason: declined,
      findings: declineFindings,
      // Conditional spread keeps the field absent where the whole slate stands.
      ...((finalists === undefined) ? {} : { finalists, }),
      ...((basis === undefined) ? {} : { basis, }),
    },);
  }

  /**
   The shipped finalist's value and author.
   */
  const {
    value,
    producer,
  } = tie.candidate;
  l.info(
    `translate stage: challenge round tied at weight ${String(tie.weight,)} across all ${String(rotated.length,)} `
      + `valid candidates with nothing to keep; shipping candidate ${String(tie.index,)} from `
      + `${describeProducer(producer,)} by ${tie.basis}`,
  );
  return {
    ...keepIncumbent,
    text: value.text,
    origin: value.origin,
    producer,
    decision: 'judged',
    voteWeight: tie.weight,
    tally: outcome.tally,
    ballots: outcome.ballots,
    findings: [
      ...declineFindings,
      `${RUNOFF_TIE_BROKEN_FINDING} (${tie.basis})`,
    ],
    selectedIndex: tie.index,
    shippedIndex: tie.index,
    perCandidate: outcome.perCandidate,
  };
}

//endregion Translate run-off tie
