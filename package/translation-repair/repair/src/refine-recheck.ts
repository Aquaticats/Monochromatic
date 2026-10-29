import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  AdjudicatedIssue,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { IssueCheckerReading, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { collectRefinedAuthors, } from './issue-authors.ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import { runCheckerStage, } from './repair-edit-stages.ts';
import {
  silentStagesOf,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Refine recheck
// The checker round a refined slice passes before its rewrite may ship: the
// naturalness lane's rollback gate (ledger L11).
//
// SPLIT OUT OF `refine-slice-settle.ts` for the file-length cap when the gate
// gained its exit for a round short of quorum. The settler calls this once,
// after a rewrite won and before the damage probe, and rolls the whole slice
// back on anything but a pass.

/**
 Whether a refinement kept every issue the checkers had already confirmed,
 and made no accepted issue `T1` leaves open regress.

 Rolls back the WHOLE slice when it did not. Checkers report per ISSUE while
 refinement happens per paragraph, and an issue can span paragraphs, so which
 paragraph broke a given issue is not derivable from what the checker returns.
 The regressed issue is named in the findings so a later session can judge
 whether finer attribution is worth building.

 @param client - injected model client

 @param checkerModelIds - checkers as seated at the stage

 @param outcome - settled accuracy outcome for this slice, carrying who wrote
 the repaired text this rewrote

 @param refineContributors - models whose rewrite won, empty when none did

 @param sourceText - original chunk text

 @param refinedText - candidate text the refinement produced

 @param identityContext - declared names and handles, when any (ledger L14)

 @param referenceContext - what the pages the original cites say, when it
 cites any (ledger L14)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Whether refinement may ship, plus the checker stage's own findings
 and the recheck's verdict as findings

 AN OPEN ISSUE IS ROLLED BACK WHERE IT REGRESSES: two or more checkers name it
 worse and their weight exceeds the rest (the tally's `regressed`, the
 threshold of a patch's unconfirmed edits, ledger L3). The owner raised both
 from one worse ballot on 2026-10-25 ("Two ballots for undo too"). The
 checkers were never going to call an issue the patch did not fix `fixed`, so
 `not-fixed` is the text as it stood and `worse` is the only verdict that
 says the rewrite damaged it. A `fixed` ballot credits nothing; the round is
 a rollback gate, and a resolution it recorded would rest on a round no panel
 or selection ever weighed.

 A ROUND SHORT OF ITS QUORUM ROLLS BACK TOO, under `refine-recheck-unheard`
 and whatever its ballots say: the gate passes a rewrite only on a round the
 checker stage closed at quorum, which is the rule the package's other gates
 between a new text and a standing one keep (the consolidation gate and the
 polish gate each ship the standing text unless two ballots name the new
 one).

 SO DOES AN ISSUE SHORT OF THAT QUORUM in a round that met it: an issue the
 round cast fewer ballots on than the checker stage closed on is named under
 `refine-recheck-unheard` with its count, whatever those ballots say, and is
 never read as regressed or worsened. An issue that was heard and broke rolls
 back beside it under `refine-rolled-back`.

 @example
 ```ts
 const retained = await retainsResolvedIssues({ client, checkerModelIds, outcome, refineContributors, sourceText, refinedText, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function retainsResolvedIssues(
  {
    client,
    checkerModelIds,
    outcome,
    refineContributors,
    sourceText,
    refinedText,
    identityContext,
    referenceContext,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly checkerModelIds: readonly RosterModelId[];
    readonly outcome: ChunkRepairOutcome;
    readonly refineContributors: readonly RosterModelId[];
    readonly sourceText: string;
    readonly refinedText: string;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<{
  readonly retained: boolean;
  readonly findings: readonly string[];

  /**
   What each checker said this time, empty where no round was bought.
   */
  readonly readings: Readonly<Record<string, IssueCheckerReading>>;
}> {
  /**
   Issues the checkers had confirmed fixed in `T1`.
   */
  const confirmed = outcome.issues
    .filter(function wasResolved(issue,) {
      return outcome.resolvedIssueIds
        .includes(issue.issueId,);
    },);

  /**
   Accepted issues `T1` does not resolve, which the rewrite was never shown:
   every accepted issue of a slice whose accuracy patch lost, where the
   rewrite is of the archive, and any a winning patch left open. LEDGER L11,
   the owner's ruling of 2026-09-28 ("Recheck the rewrite"): 1,218 of 2,144
   refined slices over every run rewrote the archive after the patch lost,
   457 of them over accepted issues, and none had a checker round.
   */
  const open = outcome.issues
    .filter(function isOpen(issue,) {
      return (issue.status === 'accepted')
        && (!outcome.resolvedIssueIds
          .includes(issue.issueId,));
    },);

  /**
   Every issue the round rules on, confirmed first.
   */
  const checked = [
    ...confirmed,
    ...open,
  ];

  // Nothing was proved about this slice and no defect is known in it, so a
  // refinement can neither un-prove nor worsen one. This is the common case:
  // the lane's whole target is text with no accepted issue, and spending a
  // checker round there would buy nothing.
  if (checked.length === 0)
    return {
      retained: true,
      findings: [],
      // NO ROUND RAN, so there is nothing to have said. Distinct from a round
      // every checker answered with the same verdict, which leaves ballots.
      readings: {},
    };

  /**
   Checker verdicts over the refined text.
   */
  const checker = await runCheckerStage({
    client,
    checkerModelIds,
    sourceText,
    patchedText: refinedText,
    issues: checked,
    authorship: collectRefinedAuthors({
      editorAuthorship: outcome.authorship,
      refineContributors,
    },),
    ...(identityContext === undefined ? {} : { identityContext, }),
    ...(referenceContext === undefined ? {} : { referenceContext, }),
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);

  /**
   The checker stage's own findings that it closed short of its quorum, empty
   when enough checkers were heard.

   READ THROUGH `silentStagesOf`, the one reader `stage-silence.ts` keeps for
   the finding the gather writes, because the checker stage hands back no
   quorum verdict of its own and a second spelling of "short of quorum" here
   could drift from the one that keeps a settlement out of the cache.
   */
  const shortOfQuorum = silentStagesOf({ findings: checker.findings, },);
  if (shortOfQuorum.length > 0) {
    // A ROUND SHORT OF ITS QUORUM IS NO CHECKER ROUND (ledger L11, the
    // owner's ruling "Recheck the rewrite"). With no ballot cast, no tally
    // says worse, so an open issue read as not worsened and the rewrite
    // shipped under `refine-recheck-passed`, the very findings a round every
    // checker answered leaves; and one heard checker of three decided alone
    // what the owner's rule of 2026-08-12 lets no single model decide. So
    // the slice rolls back to the text a checker round or the archive already
    // stood behind, as it does for a confirmed issue the round does not
    // confirm again, and the finding names the round rather than an issue:
    // nothing was heard to say the rewrite broke or worsened anything.
    //
    // DECIDED BEFORE THE ISSUES ARE READ, since a confirmed issue cannot read
    // as resolved on fewer than two ballots and would be named as one the
    // rewrite broke.
    /**
     The unheard round, in scorecard-stable wording.
     */
    const finding = `refine-recheck-unheard (${String(checker.heardCheckers,)} of ${
      String(checkerModelIds.length,)
    } ${
      wordForCount({
        count: checkerModelIds.length,
        one: 'checker',
        many: 'checkers',
      },)
    } heard)`;
    l.warn(`slice ${String(outcome.sliceIndex,)}: ${finding}; keeping the text before the rewrite`,);
    return {
      retained: false,
      findings: [
        ...checker.findings,
        finding,
      ],
      readings: checker.readings,
    };
  }

  /**
   Ballots an issue needs before this round counts as heard on it: the
   quorum the checker stage closed the round on.
   */
  const { quorum, } = checker;

  /**
   This round's reading of one issue it ruled on.

   @param issue - issue among those the round was asked about

   @returns Issue's ballots and tally, present by the checker stage's own
   contract: it builds one reading per issue asked and `nonNullishOrThrow`-reads
   each tally for it, and every issue read here is in the checked list

   @example
   ```ts
   const reading = readingOf(issue,);
   ```
   */
  function readingOf(issue: AdjudicatedIssue,): IssueCheckerReading {
    return nonNullishOrThrow(checker.readings[issue.issueId],);
  }

  /**
   Whether the round cast as many ballots on an issue as it closed on.

   THE COUNT IS THE READING'S BALLOTS, one per heard checker that ruled on
   the issue. The `missing-check` findings say the same thing per checker,
   each naming the issue by id, the checker and the cause, but are not read
   here: a finding's wording is for a reader, and the reading holds the
   ballots themselves.

   @param issue - issue among those the round was asked about

   @returns Whether the issue's ballots reach the quorum

   @example
   ```ts
   const readable = heardEnough(issue,);
   ```
   */
  function heardEnough(issue: AdjudicatedIssue,): boolean {
    return readingOf(issue,)
      .ballots
      .length
      >= quorum;
  }

  // AN ISSUE HEARD SHORT OF THE QUORUM IS UNHEARD, though the round met its
  // own. A checker heard on the round may skip an issue (`missing-check`) or
  // cast a verdict the tally cannot read, and the round's reading of that
  // issue then rests on fewer ballots than the rule that no single model
  // decides allows. With no worse ballot among them an open issue read as
  // not worsened and the rewrite shipped under `refine-recheck-passed`.
  //
  // SET APART BEFORE THE ISSUES ARE READ, as the round is: a confirmed issue
  // cannot read as resolved on fewer than two ballots, and would be named as
  // one the rewrite broke.
  /**
   Issues the round heard fewer ballots on than it closed on, each with its
   finding in scorecard-stable wording.
   */
  const unheard = checked
    .filter(function heardTooFew(issue,): boolean {
      return !heardEnough(issue,);
    },)
    .map(function toFinding(issue,): string {
      /**
       Ballots this round cast on the issue.
       */
      const cast = readingOf(issue,)
        .ballots
        .length;
      return `refine-recheck-unheard (${issue.issueId}: ${String(cast,)} ${
        wordForCount({
          count: cast,
          one: 'ballot',
          many: 'ballots',
        },)
      }, quorum ${String(quorum,)})`;
    },);
  for (const finding of unheard)
    l.warn(`slice ${String(outcome.sliceIndex,)}: ${finding}; keeping the text before the rewrite`,);

  /**
   Issues the refinement broke, named so the rollback is explainable.
   */
  const regressed = confirmed
    .filter(function readable(issue,) {
      return heardEnough(issue,);
    },)
    .filter(function brokeIt(issue,) {
      return !readingOf(issue,)
        .tally
        .resolved;
    },)
    .map(function toId(issue,) {
      return issue.issueId;
    },);

  /**
   Open issues the refinement made worse by the tally's own definition: two or
   more checkers named it worse and the worse weight exceeds the rest
   (`regressed`, owner 2026-10-25, "Two ballots for undo too").
   */
  const worsened = open
    .filter(function readable(issue,) {
      return heardEnough(issue,);
    },)
    .filter(function madeItWorse(issue,) {
      return readingOf(issue,)
        .tally
        .regressed;
    },)
    .map(function toId(issue,) {
      return issue.issueId;
    },);

  /**
   Every issue the rollback answers for.
   */
  const lost = [
    ...regressed,
    ...worsened,
  ];
  // THE CHECKER STAGE'S OWN FINDINGS RIDE ON EVERY VERDICT, as they do on the
  // accuracy lane's outcome (`repair-chunk.ts`): a round that met its quorum
  // with a voice lost, or with a ballot the tally could not read whole, is
  // otherwise the same findings as a round every checker answered.
  if ((unheard.length === 0) && (lost.length === 0))
    return {
      retained: true,
      findings: [
        ...checker.findings,
        `refine-recheck-passed (${String(checked.length,)} ${
          wordForCount({
            count: checked.length,
            one: 'issue',
            many: 'issues',
          },)
        })`,
      ],
      readings: checker.readings,
    };
  // THE ROLLBACK OF A WINNING REWRITE IS LOGGED HERE, where the issues and the
  // ballots that decided it are in hand. The select stage has already logged
  // that the rewrite won, so a run log without this line reads as a rewrite
  // that shipped (XingZ6020, slices 4, 26 and 32).
  if (lost.length > 0) {
    /**
     The ballots cast on each issue the rollback answers for, in issue order.
     */
    const ballotsByIssue = lost
      .map(function ballotsOn(issueId,): string {
        return `ballots on ${issueId}: ${
          nonNullishOrThrow(checker.readings[issueId],)
            .ballots
            .map(function spell({
              modelId,
              verdict,
            },): string {
              return `${modelId} ${verdict}`;
            },)
            .join(', ',)
        }`;
      },)
      .join('; ',);
    l.warn(
      `slice ${String(outcome.sliceIndex,)}: refine-rolled-back (${lost.join(', ',)}); rewrite by ${
        (refineContributors.length === 0) ? 'no recorded model' : refineContributors.join(', ',)
      }; ${ballotsByIssue}; keeping the text before the rewrite`,
    );
  }
  return {
    retained: false,
    findings: [
      ...checker.findings,
      ...unheard,
      // Named only where a heard issue answers for the rollback, so an issue
      // short of ballots is never also named as one the rewrite broke.
      ...((lost.length === 0) ? [] : [`refine-rolled-back (${lost.join(', ',)})`,]),
    ],
    readings: checker.readings,
  };
}

//endregion Refine recheck
