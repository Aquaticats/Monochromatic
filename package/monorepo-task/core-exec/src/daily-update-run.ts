/**
 The daily update's runner: runs each group's steps, restores a failed group's files, commits what changed and tells the
 owner what failed.

 Every effect (commands, files, git, notifications) is passed in, so the sequencing rules are tested without a network.
 Rationale: `doc/planning/toolchain-always-latest.md` ("The daily update").

 @module
 */

import { oneAfterAnother, } from './daily-update-one-after-another.ts';
import {
  selects,
  type UpdateGroup,
  type UpdateStep,
} from './daily-update-plan.ts';
import {
  absoluteOf,
  changedOwnedPaths,
  relativeOf,
  restoreFiles,
  snapshotGroup,
} from './daily-update-restore-files.ts';
import type {
  CommandOutcome,
  DailyUpdateEffects,
  GroupReport,
} from './daily-update-run-types.ts';

export type {
  CommandOutcome,
  DailyUpdateEffects,
  GroupReport,
} from './daily-update-run-types.ts';
export {
  absoluteOf,
  relativeOf,
  restoreFiles,
} from './daily-update-restore-files.ts';

/**
 Runs one step's commands in order.

 @param effects - the outside world

 @param step - the step

 @returns a problem text when a command failed, empty when every command succeeded

 @example
 ```ts
 const problem = await runStep({ effects, step, },);
 ```
 */
export async function runStep({
  effects,
  step,
}: {
  readonly effects: DailyUpdateEffects;
  readonly step: UpdateStep;
},): Promise<string> {
  // Commands of one step depend on each other, so they run one after another and the first failure stops the rest.
  for await (const {
    command,
    outcome,
  } of oneAfterAnother({
    items: step.commands,
    run: async function runLogged(toRun,) {
      effects.log(`running: ${toRun.label}`,);
      return {
        command: toRun,
        outcome: await effects.runCommand(toRun,),
      };
    },
  },)) {
    if (outcome.exitCode !== 0) {
      /**
       Why the command ended, in words the owner can act on.
       */
      const reason = outcome.timedOut
        ? 'ran past its time bound and was stopped'
        : `exited with status ${outcome.exitCode}`;
      return `\`${command.label}\` ${reason}. Last output:\n${outcome.outputTail}`;
    }
  }
  return '';
}

/**
 Runs the steps of a group in order, putting files back as each failure requires.

 @param effects - the outside world

 @param group - the group

 @param snapshot - texts taken before the group began

 @returns the problems found and whether a hard step ended the group

 @example
 ```ts
 const { problems, ended, } = await runSteps({ effects, group, snapshot, },);
 ```
 */
export async function runSteps({
  effects,
  group,
  snapshot,
}: {
  readonly effects: DailyUpdateEffects;
  readonly group: UpdateGroup;
  readonly snapshot: ReadonlyMap<string, string>;
},): Promise<{
  readonly problems: readonly string[];
  readonly ended: boolean
}> {
  /**
   Problems found so far.
   */
  const problems: string[] = [];
  // Steps depend on each other and a hard failure ends the group, so they run one after another, each with its own restore.
  for await (const result of oneAfterAnother({
    items: group.steps,
    run: async function runAndRestore(step,): Promise<{
      readonly problem: string;
      readonly ended: boolean
    }> {
      /**
       Problem text of the step, empty when it succeeded.
       */
      const problem = await runStep({
        effects,
        step,
      },);
      if (problem === '') {
        return {
          problem: '',
          ended: false,
        };
      }
      if (step.soft) {
        await restoreFiles({
          effects,
          snapshot,
          selector: step.touches,
        },);
        return {
          problem: `${group.name}: "${step.name}" failed and its files were put back; the rest of the group went on. ${problem}`,
          ended: false,
        };
      }
      await restoreFiles({
        effects,
        snapshot,
      },);
      return {
        problem: `${group.name}: "${step.name}" failed, so nothing from this group was landed and its files were put back. ${problem}`,
        ended: true,
      };
    },
  },)) {
    if (result.problem !== '')
      problems.push(result.problem,);
    if (result.ended) {
      return {
        problems,
        ended: true,
      };
    }
  }
  return {
    problems,
    ended: false,
  };
}

/**
 Runs one group: checks that its files are clean, runs the steps, restores on a hard failure, commits what changed.

 @param effects - the outside world

 @param group - the group

 @returns what happened, including every problem the owner should read

 @example
 ```ts
 const report = await runGroup({ effects, group, },);
 ```
 */
export async function runGroup({
  effects,
  group,
}: {
  readonly effects: DailyUpdateEffects;
  readonly group: UpdateGroup;
},): Promise<GroupReport> {
  /**
   Files with uncommitted changes that this group would change and commit.
   */
  const dirtyBefore = await effects.dirtyPaths();
  /**
   Owned files that were already dirty before the run.
   */
  const dirtyOwned = dirtyBefore.filter(function isOwned(repositoryPath,) {
    return selects({
      selector: group.owns,
      repositoryPath,
    },);
  },);
  if (dirtyOwned.length > 0) {
    return {
      name: group.name,
      outcome: 'skipped',
      committed: [],
      commit: '',
      problems: [
        `${group.name}: skipped because these files have uncommitted changes that the update would mix with its own: ${
          dirtyOwned.join(', ',)
        }. Commit or discard them and the next run picks the update up.`,
      ],
    };
  }

  /**
   Tracked files the group may change, with the text each has now.
   */
  const snapshot = await snapshotGroup({
    effects,
    group,
  },);
  /**
   Problems found by the steps, and whether a hard step ended the group.
   */
  const stepResult = await runSteps({
    effects,
    group,
    snapshot,
  },);
  /**
   Problems so far, to which later findings are added.
   */
  const problems: string[] = [...stepResult.problems,];
  if (stepResult.ended) {
    return {
      name: group.name,
      outcome: 'failed',
      committed: [],
      commit: '',
      problems,
    };
  }

  /**
   Owned files whose text changed during the run.
   */
  const changed = await changedOwnedPaths({
    effects,
    snapshot,
  },);

  /**
   Files changed outside the group's ownership, which the update does not commit.
   */
  const dirtyAfter = await effects.dirtyPaths();
  /**
   Files that changed during the run but are not part of the group.
   */
  const unexpected = dirtyAfter.filter(function isNewAndForeign(repositoryPath,) {
    return (!dirtyBefore.includes(repositoryPath,)) && (!selects({
      selector: group.owns,
      repositoryPath,
    },));
  },);
  if (unexpected.length > 0) {
    problems.push(
      `${group.name}: these files changed but are not part of the group, so they were left uncommitted: ${
        unexpected.join(', ',)
      }`,
    );
  }

  if ((changed.length === 0) || (group.commitSubject === '')) {
    return {
      name: group.name,
      outcome: 'unchanged',
      committed: [],
      commit: '',
      problems,
    };
  }

  /**
   Commit message: the subject, then what moved and what went wrong.
   */
  const message = [
    group.commitSubject,
    '',
    `Changed files: ${changed.length}.`,
    ...problems,
  ].join('\n',);
  try {
    await effects.commit({
      message,
      paths: changed,
    },);
  }
  catch (error) {
    problems.push(
      `${group.name}: the commit failed (${String(error,)}); the changed files stay in the working tree: ${
        changed.join(', ',)
      }`,
    );
    return {
      name: group.name,
      outcome: 'failed',
      committed: [],
      commit: '',
      problems,
    };
  }
  return {
    name: group.name,
    outcome: 'landed',
    committed: changed,
    commit: await effects.headCommit(),
    problems,
  };
}

/**
 Runs every group in order and tells nobody: the caller decides how the reports reach the owner.
 A failed or skipped group never stops the groups after it.

 @param effects - the outside world

 @param groups - groups in the order they run

 @returns one report per group

 @example
 ```ts
 const reports = await runGroups({ effects, groups: dailyUpdateGroups({ globalLockPath, },), },);
 ```
 */
export async function runGroups({
  effects,
  groups,
}: {
  readonly effects: DailyUpdateEffects;
  readonly groups: readonly UpdateGroup[];
},): Promise<readonly GroupReport[]> {
  // Groups share the working tree and the toolchain, so they run one after another.
  /**
   One report per group, in order.
   */
  const reports = await Array.fromAsync(oneAfterAnother({
    items: groups,
    run: async function runLogged(group,): Promise<GroupReport> {
      effects.log(`group: ${group.name}`,);
      /**
       What the group did.
       */
      const report = await runGroup({
        effects,
        group,
      },);
      effects.log(`group ${group.name}: ${report.outcome} (${report.committed
        .length} files committed)`,);
      return report;
    },
  },),);

  return reports;
}

/**
 Every problem of every group, in group order.

 @param reports - what the groups did

 @returns the problems, empty when nothing went wrong

 @example
 ```ts
 const problems = groupProblems(reports,);
 ```
 */
export function groupProblems(reports: readonly GroupReport[],): readonly string[] {
  return reports.flatMap(function problemsOf(report,) {
    return report.problems;
  },);
}

/**
 Runs every group in order and tells the owner about anything that went wrong, in one notification.
 A failed or skipped group never stops the groups after it.

 @param effects - the outside world

 @param groups - groups in the order they run

 @returns one report per group

 @example
 ```ts
 const reports = await runDailyUpdate({ effects, groups: dailyUpdateGroups({ globalLockPath, },), },);
 ```
 */
export async function runDailyUpdate({
  effects,
  groups,
}: {
  readonly effects: DailyUpdateEffects;
  readonly groups: readonly UpdateGroup[];
},): Promise<readonly GroupReport[]> {
  /**
   What the groups did.
   */
  const reports = await runGroups({
    effects,
    groups,
  },);
  /**
   Every problem of every group.
   */
  const problems = groupProblems(reports,);
  if (problems.length > 0) {
    await effects.notify({
      title: 'Daily toolchain update needs a look',
      body: `${problems.join('\n\n',)}\n\nThe journal has the full output: journalctl --user --unit toolchain-daily-update`,
    },);
  }
  return reports;
}
