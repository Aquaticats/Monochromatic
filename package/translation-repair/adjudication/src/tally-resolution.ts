import {
  type IssueAuthorship,
  SELF_VOTE_WEIGHT,
  wroteTextForIssue,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  isResolutionVerdict,
  type ResolutionReportWire,
  type ResolutionVerdict,
} from './resolution-wire.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Resolution tally
// Checker replies resolve through the prompt plan exactly like panel
// ballots, and a strict majority of the WEIGHT behind `fixed` verdicts marks
// an issue resolved. `worse` majorities from two or more checkers flag the
// repair as damaging, which candidate measurement counts as a regression.
// Weight rather than a count, because a checker that helped write the text it
// is judging is heard at `SELF_VOTE_WEIGHT`, matching the discount selection
// already applies.

/**
 Cast ballots an issue needs before it can read as resolved: two, so one
 checker never decides that a defect is gone. Counted in ballots rather
 than weight, as `MIN_SELECTION_BALLOTS` is for a slate winner.
 */
export const MIN_RESOLUTION_BALLOTS = 2;

/**
 Ballots naming `worse` an issue needs before it can read as regressed: two,
 so one checker never sets the flag that reorders shippable candidates, strips
 an edit from a patch or rolls a rewrite back (the three readers of the flag
 since 2026-10-25, "Two ballots for undo too"). Counted
 in ballots naming `worse`, not in cast ballots and not in weight, so a
 `worse` against a text author's own half-weight `fixed` is still one voice.
 */
const MIN_REGRESSION_BALLOTS = 2;

/**
 One checker's resolved verdicts over one sheet.

 @example
 ```ts
 const report: ResolutionBallot = {
   verdicts: { 'adjudicated/abc': 'fixed', },
   findings: [],
 };
 ```
 */
export type ResolutionBallot = {
  /**
   Verdicts keyed by issue id.
   */
  readonly verdicts: Readonly<Record<string, ResolutionVerdict>>;

  /**
   Wire irregularities in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Resolves one wire report into id-keyed verdicts through the prompt plan.
 Fails closed per item: out-of-range or duplicate references and unknown
 verdicts become findings, and issues left unanswered are recorded.

 An unanswered issue is named by its id and the checker that left it, with
 the cause:`unanswered` where the report held no check on it, and
 `unknown-verdict` where every check it held on it carried a verdict the
 tally does not know (the `unknown-resolution-verdict` finding beside it
 says which verdict). One finding per checker per issue is written, so two
 checkers skipping one issue leave two lines that tell them apart; numbering
 the issue by its place on the sheet left them identical.

 @param wire - report as the checker reported it

 @param issueIds - issue ids in prompt numbering order

 @param checkerModelId - checker whose report this is, so an unanswered issue
 names who left it

 @returns Resolved ballot with findings as data

 @example
 ```ts
 const ballot = resolveResolutionChecks({ wire, issueIds, checkerModelId, },);
 ```
 */
export function resolveResolutionChecks(
  {
    wire,
    issueIds,
    checkerModelId,
  }: {
    readonly wire: ResolutionReportWire;
    readonly issueIds: readonly string[];
    readonly checkerModelId: string;
  },
): ResolutionBallot {
  /**
   Findings accumulated across every wire item.
   */
  const findings: string[] = [];

  /**
   Resolved verdicts keyed by issue id; first occurrence wins. A map until
   handed back, as every record filled by a key is (ledger B77).
   */
  const verdicts = new Map<string, ResolutionVerdict>();

  /**
   Issues some check of this report answered with a verdict the tally does not
   know, which tells an unanswered issue's two causes apart.
   */
  const unknownVerdictFor = new Set<string>();
  for (const check of wire.checks) {
    /**
     Issue id referenced by this check's one-based number.
     */
    const issueId = issueIds[check.issue - 1];
    if ((check.issue < 1) || (issueId === undefined)) {
      findings.push(`check-index-out-of-range (${check.issue})`,);
      continue;
    }
    if (verdicts.has(issueId,)) {
      findings.push(`duplicate-check (${check.issue})`,);
      continue;
    }
    if (!isResolutionVerdict(check.verdict,)) {
      findings.push(`unknown-resolution-verdict (${check.verdict})`,);
      unknownVerdictFor.add(issueId,);
      continue;
    }
    verdicts.set(
      issueId,
      check.verdict,
    );
  }
  for (const issueId of issueIds) {
    if (!verdicts.has(issueId,)) {
      findings.push(`missing-check (${issueId}, ${checkerModelId}, ${
        unknownVerdictFor.has(issueId,)
          ? 'unknown-verdict'
          : 'unanswered'
      })`,);
    }
  }

  return {
    verdicts: Object.fromEntries(verdicts,),
    findings,
  };
}

/**
 Fate of one issue after the checker majority spoke.

 @example
 ```ts
 const fate: IssueResolutionTally = {
   fixed: 1.5, notFixed: 1, worse: 0, resolved: true, regressed: false,
 };
 ```
 */
export type IssueResolutionTally = {
  /**
   Weight behind verdicts judging the defect gone. FRACTIONAL WHENEVER AN
   AUTHOR VOTED, so this is not a head count.
   */
  readonly fixed: number;

  /**
   {@inheritDoc IssueResolutionTally.fixed} Judging it still present.
   */
  readonly notFixed: number;

  /**
   {@inheritDoc IssueResolutionTally.fixed} Judging the revision damaged the
   region further.
   */
  readonly worse: number;

  /**
   Whether fixed verdicts strictly outweigh the rest of the cast votes, with
   at least {@link MIN_RESOLUTION_BALLOTS} ballots cast.
   */
  readonly resolved: boolean;

  /**
   Whether worse verdicts strictly outweigh the rest of the cast votes, with
   at least `MIN_REGRESSION_BALLOTS` ballots naming `worse`.
   */
  readonly regressed: boolean;
};

/**
 One cast verdict with the weight its checker earned on this issue.

 @example
 ```ts
 const weighed: WeighedVerdict = { verdict: 'fixed', weight: 1, };
 ```
 */
type WeighedVerdict = {
  /**
   What this checker answered.
   */
  readonly verdict: ResolutionVerdict;

  /**
   Weight this answer carries: {@link SELF_VOTE_WEIGHT} when the checker helped
   write the text it is judging, otherwise a whole vote.
   */
  readonly weight: number;
};

/**
 Weight standing behind one answer.

 SUMMED PER ANSWER RATHER THAN SUBTRACTED FROM A TOTAL. Halves are exact in
 binary so no rounding is at stake, but deriving one figure from the others
 would silently credit an unrecognized answer to whatever was left over.

 @param weighed - cast verdicts with their weights

 @param answer - verdict being weighed

 @returns Summed weight behind that answer

 @example
 ```ts
 const backing = weightBehind({ weighed, answer: 'fixed', },);
 ```
 */
function weightBehind(
  {
    weighed,
    answer,
  }: {
    readonly weighed: readonly WeighedVerdict[];
    readonly answer: ResolutionVerdict;
  },
): number {
  return weighed
    .filter(function isAnswer(one,) {
      return one.verdict === answer;
    },)
    .reduce(
    function add(
      total,
      one,
    ) {
      return total + one.weight;
    },
    0,
    );
}

/**
 Tallies checker ballots per issue, discounting a checker that helped write the
 text it is judging.
 An issue with no cast verdicts stays unresolved and unregressed:
 silence proves nothing in either direction.

 A DISCOUNT RATHER THAN A BAR. A model that wrote a repair is often best placed
 to see whether it worked, so its verdict is heard at {@link SELF_VOTE_WEIGHT}
 rather than discarded.

 THE DISCOUNT IS DIRECTION-BLIND. An author calling its own work `not-fixed` is
 halved exactly as one calling it `fixed` is, because what is being weighed is
 the stake in the text, not which way the answer points.

 ONE CAST BALLOT RESOLVES NOTHING, whatever its weight, since 2026-09-27
 ({@link MIN_RESOLUTION_BALLOTS}). This was previously accepted, since half a vote
 outweighs none, and it reached far past the lone author: 759 of 8,788
 recorded readings over 403 run directories resolved on one ballot, 756 of
 them inside a selected patch, where a checker omitted the issue or the
 bench had one voice left. The owner's rule of 2026-08-12 is that no single
 model decides.

 `regressed` NEEDS TWO VOICES, since 2026-10-25 (lead ruling on re-check 3,
 by the owner rule that one judge alone never decides). Candidate selection
 ranks it third (`compareCandidates`), above total resolution and
 preservation, so it can choose which of two shippable candidates ships; one
 `worse` ballot, with the other checker silent or against a text author's own
 half-weight `fixed`, therefore leaves it unset. It holds when two or more
 ballots name `worse` and the weight behind `worse` exceeds the weight behind
 `fixed` and `not-fixed` together (`MIN_REGRESSION_BALLOTS`). Level weights do
 not regress, as before.

 THE FLAG IS THE ONE DEFINITION OF A WORSE VERDICT THAT ACTS, read at three
 sites since the owner's answer of 2026-10-25, "Two ballots for undo too": the
 candidate ranking (`regressedKnownIssues`, `repair/src/chunk-measure.ts`), the
 strip of a patch's unconfirmed edits (ledger L3,
 `repair/src/repair-worse-strip.ts`) and the rollback of a rewrite on an open
 issue (ledger L11, `repair/src/refine-recheck.ts`). The last two restore
 earlier text and ship nothing new, but they change what ships, so they read
 this flag and not a count of their own. They acted on one `worse` ballot
 before that answer.

 @param issueIds - issue ids under check

 @param ballots - resolved ballots keyed by checker id

 @param authorship - who wrote the text under check

 @returns Per-issue tallies keyed by issue id

 @example
 ```ts
 const tallies = tallyResolutionChecks({ issueIds, ballots, authorship, },);
 ```
 */
export function tallyResolutionChecks(
  {
    issueIds,
    ballots,
    authorship,
  }: {
    readonly issueIds: readonly string[];
    readonly ballots: Readonly<Record<string, ResolutionBallot>>;
    readonly authorship: IssueAuthorship;
  },
): Readonly<Record<string, IssueResolutionTally>> {
  return Object.fromEntries(issueIds.map(function toEntry(issueId,) {
    /**
     Cast verdicts for this issue across every ballot, each carrying the weight
     its checker earned on this issue.
     */
    const cast = Object
      .entries(ballots,)
      .flatMap(function toWeighed([modelId, ballot,]: ReadonlyEntry<string, ResolutionBallot>,): readonly WeighedVerdict[] {
        /**
         This checker's verdict, when cast.
         */
        const verdict = ballot.verdicts[issueId];
        if (verdict === undefined)
          return [];
        return [
          {
            verdict,
            weight: wroteTextForIssue({
              authorship,
              issueId,
              modelId,
            },)
              ? SELF_VOTE_WEIGHT
              : 1,
          },
        ];
      },);

    /**
     Weight calling the defect gone.
     */
    const fixed = weightBehind({
      weighed: cast,
      answer: 'fixed',
    },);

    /**
     Weight calling the defect still present.
     */
    const notFixed = weightBehind({
      weighed: cast,
      answer: 'not-fixed',
    },);

    /**
     Weight calling the revision damaging.
     */
    const worse = weightBehind({
      weighed: cast,
      answer: 'worse',
    },);

    /**
     Ballots that named the revision damaging, however much weight each
     carries.
     */
    const worseBallots = cast
      .filter(function namesWorse(one,) {
        return one.verdict === 'worse';
      },)
      .length;

    return [
      issueId,
      {
        fixed,
        notFixed,
        worse,
        resolved: (fixed > (notFixed + worse)) && (cast.length >= MIN_RESOLUTION_BALLOTS),
        regressed: (worse > (fixed + notFixed)) && (worseBallots >= MIN_REGRESSION_BALLOTS),
      },
    ];
  },),);
}

//endregion Resolution tally
