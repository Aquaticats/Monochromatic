import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type CheckerStageResult,
  runCheckerStage,
} from '@monochromatic-dev/translation-repair-repair/ts';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { IssueResolutionTally, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  ABSENT_ISSUE,
  ALL_FIXED_PATCHED_TEXT,
  ALL_FIXED_SOURCE_TEXT,
  MEANING_ISSUE,
  MIXED_SHEET_PATCHED_TEXT,
  SINGLE_ISSUE_CASES,
  SOURCE_TEXT,
  TENSE_ISSUE,
} from './checker-sensitivity-input.ts';
import {
  CHECKER_NOTE,
  sheetCheckLine,
  singleCheckLine,
} from './checker-sensitivity-print.ts';
import {
  RUN_MODELS,
  RUN_PER_CALL_TIMEOUT_MS,
} from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Checker sensitivity run
// Asks the resolution checkers about the single-issue cases and the two
// sheets and prints what they said, which is the whole of the checker
// sensitivity runner once the process has handed it a way to build a client.

/**
 Asks the checkers about a candidate over some accepted issues.

 @param client - client this one call asks through

 @param sourceText - original the checkers judge against

 @param patchedText - candidate the checkers judge

 @param issues - accepted issues the candidate is judged against

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns The stage's result

 @example
 ```ts
 const checker = await askCheckers({ client, sourceText, patchedText, issues: [TENSE_ISSUE,], stageSettings, },);
 ```
 */
async function askCheckers(
  {
    client,
    sourceText,
    patchedText,
    issues,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly sourceText: string;
    readonly patchedText: string;
    readonly issues: readonly AdjudicatedIssue[];
    readonly stageSettings: StageSettings;
  },
): Promise<CheckerStageResult> {
  return await runCheckerStage({
    client,
    checkerModelIds: RUN_MODELS.checkerModelIds,
    sourceText,
    patchedText,
    issues,
    authorship: UNATTRIBUTED_TEXT,
    signal: new AbortController().signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l: tagged({ tag: 'checker-sensitivity', },),
    stageSettings,
  },);
}

/**
 Tally the stage kept for one issue it was asked about.

 @param checker - the stage's result

 @param issueId - issue the stage was asked about

 @returns The issue's tally

 @throws {@link Error} when the result holds no tally for an issue the stage
 was given, which the stage cannot produce: it throws itself when a tally is
 missing for an issue it numbered

 @example
 ```ts
 const tally = tallyOf({ checker, issueId: TENSE_ISSUE.issueId, },);
 ```
 */
function tallyOf(
  {
    checker,
    issueId,
  }: {
    readonly checker: CheckerStageResult;
    readonly issueId: string;
  },
): IssueResolutionTally {
  /**
   Tally for the issue.
   */
  const tally = checker.tallies[issueId];
  if (tally === undefined) {
    throw new Error(
      `unreachable: the checker stage returned no tally for ${issueId}, which it was asked about, `
        + 'and it throws when it holds none for an issue it numbered',
    );
  }

  return tally;
}

/**
 Asks the checkers about one candidate and reports the tally.

 @param label - case name for the verdict line

 @param patchedText - candidate the checkers judge

 @param expectation - what a discriminating checker should answer

 @param client - client this one call asks through

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await checkOne({ label: 'unfixed', patchedText: DEFECTIVE_TEXT, expectation: 'not-fixed', client, stageSettings, },);
 ```
 */
export async function checkOne(
  {
    label,
    patchedText,
    expectation,
    client,
    stageSettings,
  }: {
    readonly label: string;
    readonly patchedText: string;
    readonly expectation: string;
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Checker result for this single issue.
   */
  const checker = await askCheckers({
    client,
    sourceText: SOURCE_TEXT,
    patchedText,
    issues: [TENSE_ISSUE,],
    stageSettings,
  },);

  console.log(
    singleCheckLine({
      label,
      expectation,
      heard: checker.heardCheckers,
      tally: tallyOf({
        checker,
        issueId: TENSE_ISSUE.issueId,
      },),
    },),
  );
}

/**
 Asks the checkers about a SHEET of issues at once, as production does.

 The single-issue cases establish that the stage can discriminate at all. This
 one asks whether it still discriminates when the sheet is mixed, which is the
 only shape the 98.1 percent rate was ever measured on: production passes
 every accepted issue of a chunk in one call, so a checker that keeps up on
 one issue and agrees with everything on seven would produce that rate while
 proving nothing.

 @param client - client this one call asks through

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await checkMixedSheet({ client, stageSettings, },);
 ```
 */
export async function checkMixedSheet(
  {
    client,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Checker result over the mixed sheet.
   */
  const checker = await askCheckers({
    client,
    sourceText: SOURCE_TEXT,
    patchedText: MIXED_SHEET_PATCHED_TEXT,
    issues: [
      TENSE_ISSUE,
      MEANING_ISSUE,
      ABSENT_ISSUE,
    ],
    stageSettings,
  },);

  for (const [
    issueId,
    expectation,
  ] of [
    [
      TENSE_ISSUE.issueId,
      'fixed',
    ],
    [
      MEANING_ISSUE.issueId,
      'not-fixed',
    ],
    [
      ABSENT_ISSUE.issueId,
      'not-fixed-defect-was-never-there',
    ],
  ] as const) {
    console.log(
      sheetCheckLine({
        sheet: 'mixed-sheet',
        issueId,
        expectation,
        tally: tallyOf({
          checker,
          issueId,
        },),
      },),
    );
  }
}

/**
 Asks the checkers about a sheet of three issues that were ALL fixed.

 Isolates the variable the mixed sheet left confounded. That sheet changed two
 things at once against the single-issue case: it grew to three issues AND its
 candidate carried a loud unfixed defect, so under-crediting there could have
 come from either. Here the sheet is the same size and every issue really is
 repaired. Continued under-crediting indicts SHEET SIZE; correct crediting
 points at contamination from the unfixed defect instead.

 @param client - client this one call asks through

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await checkAllFixedSheet({ client, stageSettings, },);
 ```
 */
export async function checkAllFixedSheet(
  {
    client,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Checker result over the all-fixed sheet.
   */
  const checker = await askCheckers({
    client,
    sourceText: ALL_FIXED_SOURCE_TEXT,
    patchedText: ALL_FIXED_PATCHED_TEXT,
    issues: [
      TENSE_ISSUE,
      MEANING_ISSUE,
      ABSENT_ISSUE,
    ],
    stageSettings,
  },);

  for (const issue of [
    TENSE_ISSUE,
    MEANING_ISSUE,
    ABSENT_ISSUE,
  ]) {
    console.log(
      sheetCheckLine({
        sheet: 'all-fixed',
        issueId: issue.issueId,
        expectation: 'fixed',
        tally: tallyOf({
          checker,
          issueId: issue.issueId,
        },),
      },),
    );
  }
}

/**
 Runs the cases that separate a discriminating checker from a
 rubber-stamping one.

 @param newClient - builds the client each call asks through: ONE FRESH CLIENT
 PER CALL, since a client reuses the reply to a prompt it has already been
 asked

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @example
 ```ts
 await runCheckerSensitivity({ newClient: createProbeRunClient, stageSettings, },);
 ```
 */
export async function runCheckerSensitivity(
  {
    newClient,
    stageSettings,
  }: {
    readonly newClient: () => ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  // Sequential so this never competes with a running corpus pass for the
  // per-model stream slots.
  /* oxlint-disable no-await-in-loop -- sequential by design, see comment */
  for (const check of SINGLE_ISSUE_CASES)
    await checkOne({
      ...check,
      client: newClient(),
      stageSettings,
    },);
  /* oxlint-enable no-await-in-loop */

  await checkMixedSheet({
    client: newClient(),
    stageSettings,
  },);
  await checkAllFixedSheet({
    client: newClient(),
    stageSettings,
  },);

  console.log(CHECKER_NOTE,);
}

//endregion Checker sensitivity run
