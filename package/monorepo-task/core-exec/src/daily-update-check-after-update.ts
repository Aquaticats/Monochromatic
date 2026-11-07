/**
 The second pass of the daily update's package checks: after the update commits, every package is checked again, each
 package that passed before and fails now gets an issue, and what could not be told is listed for the owner.

 @module
 */

import { CHECK_PASS_DEADLINE_MS, } from './daily-update-check-bounds.ts';
import { compareChecks, } from './daily-update-check-comparison.ts';
import { readExternal, } from './daily-update-check-before-update.ts';
import { oneIssuePerMarker, } from './daily-update-check-pending-issues.ts';
import { runChecks, } from './daily-update-check-run.ts';
import { publishIssues, } from './daily-update-issue-publish.ts';
import { renderIssue, } from './daily-update-issue-text.ts';
import {
  cargoLockMoves,
  pnpmImporterMoves,
  toolchainLockMoves,
} from './daily-update-lock-moves.ts';
import type {
  BeforeUpdate,
  CheckComparison,
  CheckEffects,
  CheckReport,
  CheckResult,
  CheckVerdict,
  FirstError,
  MoveSource,
  PackageCheck,
  PublishOutcome,
} from './daily-update-package-check-types.ts';
import type {
  DailyUpdateEffects,
  GroupReport,
} from './daily-update-run-types.ts';

/**
 Unknown checks named in the notification; the journal lists all of them.
 The desktop shows the first thousand characters of a notification, which fits this many lines of the form
 `package task: reason`.
 */
export const LISTED_UNKNOWN_CHECKS = 8;

/**
 The report of a run that did not compare anything.
 */
export const NO_CHECK_REPORT: CheckReport = {
  ran: false,
  comparisons: [],
  newlyRed: [],
  alreadyRed: [],
  unknown: [],
  recovered: [],
  outcomes: [],
};

/**
 Reads what moved in the locks that bear on one check.

 @param checkEffects - the outside world of the checks

 @param check - the newly red check

 @param commits - the commit before the update and the last commit of the update

 @param shared - moves that bear on every check (the repository and machine-global toolchains)

 @returns the moves per lock, leaving out locks where nothing moved

 @example
 ```ts
 const moves = await movesFor({ checkEffects, check, commits, shared: [], },);
 ```
 */
export async function movesFor({
  checkEffects,
  check,
  commits,
  shared,
}: {
  readonly checkEffects: CheckEffects;
  readonly check: PackageCheck;
  readonly commits: {
    readonly before: string;
    readonly after: string
  };
  readonly shared: readonly MoveSource[];
},): Promise<readonly MoveSource[]> {
  /**
   The lock that holds this kind of package's versions.
   */
  const path = check.kind === 'rust' ? `${check.package}/Cargo.lock` : 'pnpm-lock.yaml';
  // The two texts do not depend on each other, so they are read side by side.
  /**
   The lock before and after the update.
   */
  const [before, after,] = await Promise.all([
    checkEffects.fileAt({
      commit: commits.before,
      path,
    },),
    checkEffects.fileAt({
      commit: commits.after,
      path,
    },),
  ],);
  return [
    {
      source: path,
      moves: check.kind === 'rust'
        ? cargoLockMoves({
          before,
          after,
        },)
        : pnpmImporterMoves({
          before,
          after,
          importer: check.package,
        },),
    },
    ...shared,
  ].filter(function hasMoves(source,) {
    return source.moves
      .length
      > 0;
  },);
}

/**
 First error to show for a failed check, with a stand-in when it printed no error line.

 @param result - the failed result

 @returns the error

 @example
 ```ts
 const error = errorOf(result,);
 ```
 */
export function errorOf(result: CheckResult,): FirstError {
  if (result.firstError !== undefined)
    return result.firstError;
  return {
    code: '',
    message: 'the check failed without printing a line that starts with error',
    excerpt: 'the check failed without printing a line that starts with error; run the command below for its output',
  };
}

/**
 Finds how the toolchain locks moved: the repository's lock between two commits and the outside locks between two
 readings.

 @param checkEffects - the outside world of the checks

 @param commits - the commit before the update and the last commit of the update

 @param externalBefore - outside files before the update

 @param externalAfter - outside files after the update

 @returns the moves per lock, including locks where nothing moved

 @example
 ```ts
 const shared = await toolchainMoves({ checkEffects, commits, externalBefore, externalAfter, },);
 ```
 */
export async function toolchainMoves({
  checkEffects,
  commits,
  externalBefore,
  externalAfter,
}: {
  readonly checkEffects: CheckEffects;
  readonly commits: {
    readonly before: string;
    readonly after: string
  };
  readonly externalBefore: ReadonlyMap<string, string>;
  readonly externalAfter: ReadonlyMap<string, string>;
},): Promise<readonly MoveSource[]> {
  /**
   Moves of the outside locks.
   */
  const outside: MoveSource[] = [];
  for (const [path, text,] of externalBefore) {
    outside.push({
      source: 'machine-global toolchain lock',
      moves: toolchainLockMoves({
        before: text,
        after: externalAfter.get(path,) ?? '',
      },),
    },);
  }
  return [
    {
      source: 'toolchain.lock.json',
      moves: toolchainLockMoves({
        before: await checkEffects.fileAt({
          commit: commits.before,
          path: 'toolchain.lock.json',
        },),
        after: await checkEffects.fileAt({
          commit: commits.after,
          path: 'toolchain.lock.json',
        },),
      },),
    },
    ...outside,
  ];
}

/**
 Writes what the owner is told about the issues and the checks that could not be compared.

 @param comparisons - every check compared

 @param outcomes - what came of publishing the issues

 @returns one text per thing to tell

 @example
 ```ts
 const told = problemsOf({ comparisons, outcomes, },);
 ```
 */
export function problemsOf({
  comparisons,
  outcomes,
}: {
  readonly comparisons: readonly CheckComparison[];
  readonly outcomes: readonly PublishOutcome[];
},): readonly string[] {
  /**
   One line per issue.
   */
  const issueLines = outcomes.map(function lineOf(outcome,) {
    if (outcome.action === 'failed') {
      return `${outcome.issue
        .title}: the issue could not be published and is kept for the next run (${outcome.reason})`;
    }
    return `${outcome.issue
      .title}: ${outcome.action === 'created' ? 'issue opened' : 'comment added'}${
      outcome.issueNumber === undefined ? '' : ` (#${outcome.issueNumber})`
    }`;
  },);
  /**
   Checks whose result was not a clear pass or fail.
   */
  const unknown = comparisons.filter(function isUnknown(comparison,) {
    return comparison.verdict === 'unknown';
  },);
  if (unknown.length === 0)
    return issueLines;
  return [
    ...issueLines,
    `${unknown.length} package checks could not be compared across the update, so no issue was opened for them; first ${
      Math.min(
        unknown.length,
        LISTED_UNKNOWN_CHECKS,
      )
    }:\n${
      unknown.slice(
        0,
        LISTED_UNKNOWN_CHECKS,
      )
        .map(function lineOf(comparison,) {
          return `${comparison.check
            .package} ${comparison.check
              .task}: ${comparison.reason}`;
        },)
        .join('\n',)
    }`,
  ];
}

/**
 Checks every package after the update, compares with the first pass and publishes an issue for each newly red check.

 @param effects - the outside world of the update

 @param checkEffects - the outside world of the checks

 @param before - what was taken before the update

 @param reports - what the groups did

 @param problems - where problems for the owner are collected

 @returns what the checks found

 @example
 ```ts
 const report = await checkAfterUpdate({ effects, checkEffects, before, reports, problems: [], },);
 ```
 */
export async function checkAfterUpdate({
  effects,
  checkEffects,
  before,
  reports,
  problems,
}: {
  readonly effects: DailyUpdateEffects;
  readonly checkEffects: CheckEffects;
  readonly before: BeforeUpdate;
  readonly reports: readonly GroupReport[];
  readonly problems: string[];
},): Promise<CheckReport> {
  /**
   Commits the update made.
   */
  const updateCommits = reports.map(function commitOf(report,) {
    return report.commit;
  },)
    .filter(function exists(commit,) {
      return commit !== '';
    },);
  /**
   Last commit of the update, where the repository's own locks are read.
   */
  const commits = {
    before: before.headBefore,
    after: updateCommits.at(-1,) ?? before.headBefore,
  };
  /**
   Files outside the repository, as they are after.
   */
  const externalAfter = await readExternal({
    effects,
    paths: [...before.externalBefore
      .keys(),],
  },);
  /**
   How the toolchain locks moved.
   */
  const shared = await toolchainMoves({
    checkEffects,
    commits,
    externalBefore: before.externalBefore,
    externalAfter,
  },);
  if (
    (updateCommits.length === 0) && shared.every(function none(source,) {
      return source.moves
        .length
        === 0;
    },)
  ) {
    effects.log('the update moved nothing, so the second pass over the package checks is skipped',);
    return NO_CHECK_REPORT;
  }
  /**
   Every check compared across the update.
   */
  const comparisons = compareChecks({
    before: before.results,
    after: await runChecks({
      effects: checkEffects,
      checks: before.checks,
      deadlineAt: checkEffects.now() + CHECK_PASS_DEADLINE_MS,
      log: effects.log,
    },),
  },);
  /**
   Comparisons with a verdict.

   @param verdict - the verdict

   @returns the comparisons that have it
   */
  function withVerdict(verdict: CheckVerdict,): readonly CheckComparison[] {
    return comparisons.filter(function has(comparison,) {
      return comparison.verdict === verdict;
    },);
  }
  /**
   The issue for each newly red check.
   */
  const issues = await Promise.all(withVerdict('newly-red',)
    .map(async function issueFor(comparison,) {
      return renderIssue({
        check: comparison.check,
        firstError: errorOf(comparison.after,),
        updateCommits,
        baselineCommit: before.headBefore,
        moves: await movesFor({
          checkEffects,
          check: comparison.check,
          commits,
          shared,
        },),
      },);
    },),);
  /**
   What came of publishing them.
   */
  const outcomes = await publishIssues({
    gh: checkEffects.gh,
    issues,
  },);
  /**
   Issues that could not be published, which wait for the next run.
   */
  const failed = outcomes.filter(function didFail(outcome,) {
    return outcome.action === 'failed';
  },);
  if (failed.length > 0) {
    await checkEffects.writePendingIssues(oneIssuePerMarker([
      ...await checkEffects.readPendingIssues(),
      ...failed.map(function issueOf(outcome,) {
        return outcome.issue;
      },),
    ],),);
  }
  for (const outcome of outcomes) {
    effects.log(
      `issue ${outcome.issue
        .title}: ${outcome.action}${
        outcome.issueNumber === undefined ? '' : ` #${outcome.issueNumber}`
      }${outcome.reason === '' ? '' : ` (${outcome.reason})`}`,
    );
  }
  problems.push(...problemsOf({
    comparisons,
    outcomes,
  },),);
  return {
    ran: true,
    comparisons,
    newlyRed: withVerdict('newly-red',),
    alreadyRed: withVerdict('already-red',),
    unknown: withVerdict('unknown',),
    recovered: withVerdict('recovered',),
    outcomes,
  };
}
