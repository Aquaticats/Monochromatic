import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  AdjudicatedIssue,
  ReadonlyEntry,
  RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type IssueAuthorship,
  wroteTextForIssue,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  buildResolutionMessages,
  type IssueCheckerBallot,
  type IssueCheckerReading,
  type IssueResolutionTally,
  logBallotIrregularities,
  RESOLUTION_RESPONSE_FORMAT,
  type ResolutionBallot,
  resolveResolutionChecks,
  tallyResolutionChecks,
  usableResolutionReportFor,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import { inRosterOrder, } from './refine-stage-replies.ts';
import {
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Checker stage
// The proving stage of one chunk's repair: checker models judge whether each
// accepted issue is actually gone from the candidate the editor ensemble chose.
// Lost checker voices weaken the proof and with it the candidate's
// measurements. The editors themselves live in `repair-editor-stage.ts`.

/**
 Everything the checker stage produced for one chunk.

 @example
 ```ts
 const { tallies, } = await runCheckerStage({ ... },);
 ```
 */
export type CheckerStageResult = {
  /**
   Per-issue resolution tallies keyed by issue id.
   */
  readonly tallies: Readonly<Record<string, IssueResolutionTally>>;

  /**
   Per-issue ballots and seated roster beside those tallies, keyed the same.

   Carried so what lands in the artifact is the round rather than its sum. The
   tally inside each reading is the very object {@link CheckerStageResult.tallies}
   holds, so the two cannot disagree.
   */
  readonly readings: Readonly<Record<string, IssueCheckerReading>>;

  /**
   Checkers whose reply arrived and validated.
   */
  readonly heardCheckers: number;

  /**
   Voices the stage's gather closed its round on, which is also the number of
   ballots an issue needs before the round counts as heard on it.

   The bench quorum where every seat the stage seated was one a provider
   could serve, and on a bench the router left short of it the smaller
   number the gather wrote into its `stage-short-bench` finding
   (`reachableQuorum`, never below two). It is the gather's own `quorum`,
   carried out so a reader of the round never rebuilds it from the findings'
   wording.
   */
  readonly quorum: number;

  /**
   Wire irregularities across checkers in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Runs the resolution checkers over one chunk's patched candidate.

 @param client - injected model client

 @param checkerModelIds - checker voices

 @param sourceText - original chunk text

 @param patchedText - candidate text after the apply gate

 @param issues - accepted issues the editors addressed

 @param authorship - who wrote `patchedText`, so a checker judging its own
 work is heard at a discount rather than at full weight

 @param identityContext - declared names and handles the panel judged the
 issues by, absent or empty on a page declaring none (ledger L14)

 @param referenceContext - what the pages the original links say, absent or
 empty when it links nowhere (ledger L14)

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Per-issue tallies, the quorum the round closed on, plus findings

 @example
 ```ts
 const checker = await runCheckerStage({ ... },);
 ```
 */
export async function runCheckerStage(
  {
    client,
    checkerModelIds,
    sourceText,
    patchedText,
    issues,
    authorship,
    identityContext,
    referenceContext,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly checkerModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly patchedText: string;
    readonly issues: readonly AdjudicatedIssue[];
    readonly authorship: IssueAuthorship;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<CheckerStageResult> {
  /**
   Checker sheet for the patched candidate.
   */
  const plan = buildResolutionMessages({
    sourceText,
    patchedText,
    issues,
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  },);

  /**
   Heard checkers after retry-to-quorum.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: checkerModelIds,
    messages: plan.messages,
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: RESOLUTION_RESPONSE_FORMAT,
    validate: usableResolutionReportFor({
      issueCount: plan.issueIds
        .length,
    },),
    stage: 'checker',
    l,
    graceMs: stageSettings
      .grace
      .roundMs,
  },);

  // ROSTER ORDER, NOT THE GATHER'S. `gatherStageVoices` hands voices back by
  // the retry round that heard them, then by the prompt's rotation of the
  // bench, so one input read twice ordered its findings and its readings
  // differently whenever a seat lost its first answer and was heard on a retry
  // (probed: three seats, all heard, findings in two orders). Every step of this
  // function reads this list, so the ballots, the findings, the readings and the log
  // lines all follow the roster.
  /**
   Heard checkers in the order the roster seats them.
   */
  const heard = inRosterOrder({
    replies: gather.voices,
    roster: checkerModelIds,
  },);

  /**
   Resolved ballots keyed by checker id.
   */
  const ballots: Record<string, ResolutionBallot> = Object.fromEntries(
    heard
      .map(function toEntry(voice,): readonly [
        string,
        ResolutionBallot,
      ] {
        return [
          voice.modelId,
          resolveResolutionChecks({
            wire: voice.value,
            issueIds: plan.issueIds,
            checkerModelId: voice.modelId,
          },),
        ];
      },),
  );

  logBallotIrregularities({
    ballots,
    stage: 'checker',
    l,
  },);

  /**
   Quorum degradation plus ballot irregularities across heard checkers.
   */
  const findings = [
    ...gather.findings,
    ...Object
      .values(ballots,)
      .flatMap(function toFindings(ballot,) {
        return ballot.findings;
      },),
  ];

  /**
   Majority tallies per issue.
   */
  const tallies = tallyResolutionChecks({
    issueIds: plan.issueIds,
    ballots,
    authorship,
  },);

  /**
   Everything this round decided about each issue, kept rather than summed
   away, so a settled run can answer who dissented and be re-read at another
   roster width without buying the stage again.
   */
  const readings: Record<string, IssueCheckerReading> = Object.fromEntries(
    plan.issueIds
      .map(function toReading(issueId,): readonly [
        string,
        IssueCheckerReading,
      ] {
        return [
          issueId,
          {
            ballots: heard
              .flatMap(function toBallot(voice,): readonly IssueCheckerBallot[] {
                /**
                 This checker's answer, absent where its ballot skipped this
                 issue.
                 */
                const verdict = nonNullishOrThrow(ballots[voice.modelId],)
                  .verdicts[issueId];
                if (verdict === undefined)
                  return [];
                return [
                  {
                    modelId: voice.modelId,
                    verdict,
                    wroteTheText: wroteTextForIssue({
                      authorship,
                      issueId,
                      modelId: voice.modelId,
                    },),
                  },
                ];
              },),
            configuredCheckers: checkerModelIds.length,
            tally: nonNullishOrThrow(tallies[issueId],),
          },
        ];
      },),
  );

  // ALSO PUBLISHED AS LINES, because an operator watching a live pass has the
  // log and not the artifact, and because a run that aborts before settling
  // still leaves them. Derived from the readings so the two cannot drift.
  //
  // Ids and verdicts only: no line here carries a word of either document.
  for (
    const line of Object
      .entries(readings,)
      .flatMap(function toLines([issueId, reading,]: ReadonlyEntry<string, IssueCheckerReading>,): readonly string[] {
        return reading.ballots
          .map(function toLine(ballot,): string {
            return `checker-ballot ${ballot.modelId} ${issueId} ${ballot.verdict} ${
              ballot.wroteTheText
                ? 'author'
                : 'outsider'
            }`;
          },);
      },)
  )
    l.info(line,);

  l.info(
    `checker stage: ${String(Object.keys(ballots,)
      .length,)}/${
      String(checkerModelIds.length,)
    } heard`,
  );

  return {
    tallies,
    readings,
    heardCheckers: Object.keys(ballots,)
      .length,
    // The gather's own number, handed on as it closed on it and never sized
    // here a second time.
    quorum: gather.quorum,
    findings,
  };
}

//endregion Checker stage
