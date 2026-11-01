import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type LaneContestOutcome,
  settleLaneContestBallots,
} from './lane-contest-stage.ts';
import type {
  LaneChoice,
  LaneContestBallot,
} from './lane-contest-wire.ts';
import { validateTranslatedSlice, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Lane contest publication eligibility

/**
 Deterministic publication eligibility of syntax-bearing contest candidates.

 Source text is retained so artifact reader can recompute these claims rather
 than trusting stored booleans.

 @example
 ```ts
 const eligibility: LaneContestEligibility = { syntax: 'front-matter', sourceText, archive: 'ineligible', repair: 'ineligible', translate: 'eligible', };
 ```
 */
export type LaneContestEligibility = {
  /**
   Syntax policy producing these readings.
   */
  readonly syntax: 'front-matter';

  /**
   Original syntax-bearing slice governing identity relationships.
   */
  readonly sourceText: string;

  /**
   Whether archive can cross final publication boundary.
   */
  readonly archive: 'eligible' | 'ineligible';

  /**
   Whether repair candidate can cross final publication boundary.
   */
  readonly repair: 'eligible' | 'ineligible';

  /**
   Whether translate candidate can cross final publication boundary.
   */
  readonly translate: 'eligible' | 'ineligible';
};

/**
 Candidate names deterministic syntax admission classifies, in the order the
 contest names them.
 */
const CONTEST_CANDIDATES = [
  'archive',
  'repair',
  'translate',
] as const;

/**
 Names the candidates the prompt marks unavailable before the panel votes.

 @param eligibility - deterministic readings of the three candidates

 @returns The ineligible candidates in contest order; empty when none is

 @example
 ```ts
 const marked = ineligibleCandidatesOf({ eligibility, },);
 ```
 */
export function ineligibleCandidatesOf(
  { eligibility, }: { readonly eligibility: LaneContestEligibility; },
): readonly (typeof CONTEST_CANDIDATES)[number][] {
  return CONTEST_CANDIDATES
    .filter(function isIneligible(candidate,): boolean {
      return eligibility[candidate] === 'ineligible';
    },);
}

/**
 Stable finding marking raw choices excluded by deterministic admission.
 */
export const LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING = 'lane-contest-eligibility-floor (inadmissible choices excluded)';

/**
 Reads one candidate against syntax-bearing publication invariants.

 @param sourceText - original syntax-bearing slice

 @param incumbentText - archive metadata defining compatible shape

 @param candidateText - candidate under deterministic admission

 @returns Eligibility status

 @example
 ```ts
 const status = candidateEligibility({ sourceText, incumbentText, candidateText, });
 ```
 */
function candidateEligibility(
  {
    sourceText,
    incumbentText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly candidateText: string;
  },
): LaneContestEligibility['archive'] {
  /**
   Structural result under final syntax policy.
   */
  const validation = validateTranslatedSlice({
    sourceText,
    candidateText,
    pageText: incumbentText,
    syntax: 'front-matter',
  },);
  return validation.kind === 'valid' ? 'eligible' : 'ineligible';
}

/**
 Computes deterministic eligibility for every front matter candidate.

 @param sourceText - original metadata

 @param incumbentText - archive metadata

 @param repairText - repair lane candidate

 @param translateText - translate lane candidate

 @returns Source-backed candidate eligibility record

 @example
 ```ts
 const eligibility = frontMatterContestEligibility({ sourceText, incumbentText, repairText, translateText, });
 ```
 */
export function frontMatterContestEligibility(
  {
    sourceText,
    incumbentText,
    repairText,
    translateText,
  }: {
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly repairText: string;
    readonly translateText: string;
  },
): LaneContestEligibility {
  return {
    syntax: 'front-matter',
    sourceText,
    archive: candidateEligibility({
      sourceText,
      incumbentText,
      candidateText: incumbentText,
    },),
    repair: candidateEligibility({
      sourceText,
      incumbentText,
      candidateText: repairText,
    },),
    translate: candidateEligibility({
      sourceText,
      incumbentText,
      candidateText: translateText,
    },),
  };
}

/**
 Reads contest winner after deterministically inadmissible votes are excluded.

 Raw ballots remain unchanged for audit. A vote for invalid lane contributes
 to neither lane; it is never redirected into vote for valid alternative.

 @param ballots - raw usable ballots

 @param eligibility - source-backed candidate admission

 @returns Eligible lane with normal quorum and strict lead, or neither

 @example
 ```ts
 const choice = settleEligibleLaneContestBallots({ ballots, eligibility, });
 ```
 */
export function settleEligibleLaneContestBallots(
  {
    ballots,
    eligibility,
  }: {
    readonly ballots: readonly LaneContestBallot[];
    readonly eligibility?: LaneContestEligibility;
  },
): LaneChoice {
  if (eligibility === undefined)
    return settleLaneContestBallots({ ballots, },);
  /**
   Raw ballots whose chosen lane may cross publication boundary.
   */
  const effectiveBallots = ballots.filter(function choseEligible(ballot,): boolean {
    if (ballot.choice === 'neither')
      return true;
    return eligibility[ballot.choice] === 'eligible';
  },);
  return settleLaneContestBallots({ ballots: effectiveBallots, },);
}

/**
 Applies deterministic eligibility floor to raw contest outcome.

 @param outcome - raw roster outcome

 @param eligibility - source-backed candidate admission

 @returns Same raw ballots with effective eligible choice

 @example
 ```ts
 const admitted = applyLaneContestEligibility({ outcome, eligibility, });
 ```
 */
export function applyLaneContestEligibility(
  {
    outcome,
    eligibility,
  }: {
    readonly outcome: LaneContestOutcome;
    readonly eligibility?: LaneContestEligibility;
  },
): LaneContestOutcome {
  /**
   Effective choice after candidate admission.
   */
  const choice = settleEligibleLaneContestBallots({
    ballots: outcome.ballots,
    ...((eligibility === undefined) ? {} : { eligibility, }),
  },);
  /**
   Raw ballots retained without candidate redirection.
   */
  const { ballots, } = outcome;
  /**
   Whether any raw choice named candidate publication guard rejects.
   */
  const excluded = (eligibility === undefined)
    ? false
    : ballots
      .some(function choseInvalid(ballot,): boolean {
        return (ballot.choice !== 'neither')
          && (eligibility[ballot.choice] === 'ineligible');
      },);
  if (!excluded)
    return outcome;
  return {
    ...outcome,
    choice,
    findings: [
      ...outcome.findings,
      LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING,
    ],
  };
}

/**
 Whether a contest choice may ship, and the deterministic findings when not.
 */
export type LaneContestChoiceVerdict = {
  /**
   Whether the selected lane passes the publication rules.
   */
  readonly mayShip: boolean;

  /**
   Findings behind a refusal, empty on a pass.
   */
  readonly findings: readonly string[];
};

/**
 Verdict on whether the selected lane may cross the publication boundary,
 with the deterministic findings behind a refusal.

 THE FINDINGS ARE FOR THE RUN LOG: a refusal the log names is a defect
 class the next reading finds in one grep, where "fails publication
 invariants" alone sent the 2026-09-04 luxuanwen3 reading into the slice
 records to learn that a link destination the archive had rewritten was
 the cause.

 @param outcome - contest outcome after eligibility filtering

 @param sourceText - original slice

 @param incumbentText - page text being replaced

 @param repairText - repair lane candidate

 @param translateText - translate lane candidate

 @param syntax - explicit syntax role, absent for ordinary prose

 @param lineStructured - whether the line-structure rule governs this slice,
 so a winner merging its lines is refused like any other broken floor.
 REQUIRED, NOT DEFAULTED (ledger H2): the optional form read false wherever a
 caller left it off, and this verdict was that caller, persisting a merged
 rendering of a governed slice as warm-run evidence

 @param declared - name pairs the front matter declares, which the rule
 reads for a linked title naming a declared person (class one hundred
 fourteen); none leaves that floor silent

 @returns Whether the choice may ship, and why not when it may not

 @example
 ```ts
 const verdict = laneContestChoiceVerdict({ outcome, sourceText, incumbentText, repairText, translateText, lineStructured: false, },);
 ```
 */
export function laneContestChoiceVerdict(
  {
    outcome,
    sourceText,
    incumbentText,
    repairText,
    translateText,
    syntax,
    lineStructured,
    declared = [],
  }: {
    readonly outcome: LaneContestOutcome;
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly repairText: string;
    readonly translateText: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly declared?: readonly DeclaredNamePair[];
  },
): LaneContestChoiceVerdict {
  if (outcome.choice === 'neither') {
    if (syntax === undefined)
      return {
        mayShip: true,
        findings: [],
      };
    /**
     Findings distinguishing genuine decline from no safe eligible winner.
     */
    const { findings, } = outcome;
    /**
     Whether the decline stands for an eligible slate that was empty.
     */
    const floored = findings.includes(LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING,);
    return {
      mayShip: !floored,
      findings: floored ? [LANE_CONTEST_ELIGIBILITY_FLOOR_FINDING,] : [],
    };
  }
  /**
   Exact selected lane candidate.
   */
  const candidateText = (outcome.choice === 'repair')
    ? repairText
    : translateText;
  /**
   Final deterministic eligibility for syntax and ordinary contributor authority.
   */
  const validation = validateTranslatedSlice({
    sourceText,
    candidateText,
    pageText: incumbentText,
    ...((syntax === undefined) ? {} : { syntax, }),
    lineStructured,
    declared,
  },);
  if (validation.kind === 'valid')
    return {
      mayShip: true,
      findings: [],
    };
  if (validation.kind === 'invalid')
    return {
      mayShip: false,
      findings: validation.findings,
    };
  return {
    mayShip: false,
    findings: [`no comparison was possible: ${validation.detail}`,],
  };
}

/**
 One lane the front-matter floor can exclude, with the text it offered.
 */
type FloorLane = {
  readonly lane: 'archive' | 'repair' | 'translate';
  readonly text: string;
};

/**
 Names each lane the front-matter floor excludes and the finding that
 excludes it, for the log line that reports the floor.

 WRITTEN FOR THE READING. The Uekawakuyuurei run of 2026-09-04 (16:09 UTC)
 logged only `lane-contest-eligibility-floor (inadmissible choices excluded)`
 for its metadata slice, and learning that the translate lane had restored a
 `location` field the archive dropped, which the shape rule refuses, took
 opening the slice cache. The eligibility record itself carries labels only,
 and the artifact reader holds it to exact keys, so the findings are derived
 again here from the same deterministic rule rather than carried on it.

 @param sourceText - metadata slice as the original writes it

 @param incumbentText - metadata slice as the page carries it, which is also
 the archive lane's offer

 @param repairText - repair lane's offer

 @param translateText - translate lane's offer

 @returns One line per excluded lane, naming the lane and its findings; empty
 when every lane is admissible

 @example
 ```ts
 const why = describeInadmissibleLanes({ sourceText, incumbentText, repairText, translateText, },);
 ```
 */
export function describeInadmissibleLanes(
  {
    sourceText,
    incumbentText,
    repairText,
    translateText,
  }: {
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly repairText: string;
    readonly translateText: string;
  },
): readonly string[] {
  /**
   Every lane with what it offered, in the order the contest names them.
   */
  const lanes: readonly FloorLane[] = [
    {
      lane: 'archive',
      text: incumbentText,
    },
    {
      lane: 'repair',
      text: repairText,
    },
    {
      lane: 'translate',
      text: translateText,
    },
  ];
  return lanes.flatMap(function excluded(
    {
      lane,
      text,
    },
  ): readonly string[] {
    /**
     Deterministic verdict on this lane's offer.
     */
    const validation = validateTranslatedSlice({
      sourceText,
      candidateText: text,
      pageText: incumbentText,
      syntax: 'front-matter',
    },);
    if (validation.kind === 'valid')
      return [];
    if (validation.kind === 'invalid') {
      /**
       Findings the rule raised.
       */
      const { findings, } = validation;

      /**
       Findings as one clause.
       */
      const why = findings.join(' ',);
      return [`${lane} inadmissible: ${why}`,];
    }
    return [`${lane} inadmissible: no comparison was possible: ${validation.detail}`,];
  },);
}

//endregion Lane contest publication eligibility
