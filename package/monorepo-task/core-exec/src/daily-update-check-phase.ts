/**
 The daily update with its package checks: every package is checked before the update, the update runs and commits,
 every package is checked again, and each package that passed before and fails now gets a fix-forward issue.

 The update never waits on a check's verdict and is never undone by it: the commits are made by the groups before the
 second pass starts, and nothing here restores a file. A failure anywhere in the checks or in `gh` is logged, told to
 the owner and left for the next run; it never changes the update's exit status.

 Rationale: `doc/planning/toolchain-always-latest.md` ("Checking every package after the update").

 @module
 */

import {
  checkAfterUpdate,
  NO_CHECK_REPORT,
} from './daily-update-check-after-update.ts';
import { checkBeforeUpdate, } from './daily-update-check-before-update.ts';
import { retryPendingIssues, } from './daily-update-check-pending-issues.ts';
import type {
  Attempt,
  CheckEffects,
  CheckReport,
} from './daily-update-package-check-types.ts';
import type { UpdateGroup, } from './daily-update-plan.ts';
import {
  groupProblems,
  runGroups,
} from './daily-update-run.ts';
import type {
  DailyUpdateEffects,
  GroupReport,
} from './daily-update-run-types.ts';

export type { CheckReport, } from './daily-update-package-check-types.ts';

/**
 What the daily update with its checks did.
 */
export type UpdateWithChecksResult = {
  /**
   One report per update group.
   */
  readonly reports: readonly GroupReport[];
  /**
   Exit status: 1 when a group failed, otherwise 0; the checks never change it.
   */
  readonly exitCode: number;
  /**
   What the package checks found.
   */
  readonly checks: CheckReport;
};

/**
 Runs one step of the check phase so that its failure is logged and told to the owner and goes no further.

 @param effects - the outside world of the update

 @param problems - where problems for the owner are collected

 @param name - what the step is, for the message

 @param run - the step

 @returns the step's value, or a failed attempt when it threw

 @example
 ```ts
 const attempt = await attemptStep({ effects, problems: [], name: 'checking', run: async function one() { return 1; }, },);
 ```
 */
export async function attemptStep<Value,>({
  effects,
  problems,
  name,
  run,
}: {
  readonly effects: DailyUpdateEffects;
  readonly problems: string[];
  readonly name: string;
  readonly run: () => Promise<Value>;
},): Promise<Attempt<Value>> {
  try {
    return {
      failed: false,
      value: await run(),
    };
  }
  catch (error) {
    effects.log(`${name} failed: ${String(error,)}`,);
    problems.push(`${name} failed, and the update went on without it: ${String(error,)}`,);
    return { failed: true, };
  }
}

/**
 Runs the daily update with its package checks.

 @param effects - the outside world of the update

 @param checkEffects - the outside world of the checks

 @param groups - the update's groups in the order they run

 @returns the groups' reports, the exit status the update alone decides, and what the checks found

 @example
 ```ts
 const result = await runUpdateWithChecks({ effects, checkEffects, groups: dailyUpdateGroups({ globalLockPath, },), },);
 ```
 */
export async function runUpdateWithChecks({
  effects,
  checkEffects,
  groups,
}: {
  readonly effects: DailyUpdateEffects;
  readonly checkEffects: CheckEffects;
  readonly groups: readonly UpdateGroup[];
},): Promise<UpdateWithChecksResult> {
  /**
   Problems of the check phase, told to the owner at the end.
   */
  const problems: string[] = [];
  await attemptStep({
    effects,
    problems,
    name: 'Publishing the issues an earlier run kept',
    run: function retry() {
      return retryPendingIssues({
        effects,
        checkEffects,
        problems,
      },);
    },
  },);
  /**
   What the first pass found, or that it could not run.
   */
  const before = await attemptStep({
    effects,
    problems,
    name: 'Checking the packages before the update',
    run: function first() {
      return checkBeforeUpdate({
        effects,
        checkEffects,
        groups,
      },);
    },
  },);
  // The update runs whatever the checks did: it never waits on their verdict.
  /**
   What the update's groups did.
   */
  const reports = await runGroups({
    effects,
    groups,
  },);
  /**
   What the second pass found, or that it could not run.
   */
  const second = before.failed ? before : await attemptStep({
    effects,
    problems,
    name: 'Checking the packages after the update',
    run: function afterwards() {
      return checkAfterUpdate({
        effects,
        checkEffects,
        before: before.value,
        reports,
        problems,
      },);
    },
  },);
  // One notification for the whole run: the groups' problems and the checks' problems together.
  /**
   Everything the owner is told, the update's own problems first.
   */
  const told = [
    ...groupProblems(reports,),
    ...problems,
  ];
  if (told.length > 0) {
    await effects.notify({
      title: 'Daily update needs a look',
      body: `${told.join('\n\n',)}\n\nThe journal has the full output: journalctl --user --unit toolchain-daily-update`,
    },);
  }
  return {
    reports,
    exitCode: reports.some(function failed(report,) {
      return report.outcome === 'failed';
    },) ? 1 : 0,
    checks: second.failed ? NO_CHECK_REPORT : second.value,
  };
}
