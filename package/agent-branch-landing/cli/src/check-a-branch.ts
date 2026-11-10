/**
 The check a branch gets before it lands: `main` is merged into the agent's worktree, and only what the agent could not check on
 its own tree is run there, which is the compile or the cheapest type check of the packages the branch touches.

 @module
 */

import {
  A_RELEASE_GATE_IS_INVALIDATED,
  COMPILE_ONLY_ARGUMENT,
  MERGE_OUTPUT_CHARACTERS,
  PACKAGE_TASK_LIMITS,
  TASK_ERROR_LINES,
  TASK_TAIL_CHARACTERS,
  WORKTREE_COMPILE_LIMITS,
  WRAPPER_COMPILE_TASKS,
} from './constants.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  changedPathsSinceMain,
  currentBranchOf,
  headOf,
  type Repository,
} from './git-commands.ts';
import {
  mergeMainInto,
} from './git-changes.ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';
import { checkTasksFor, } from './package-tasks.ts';
import {
  packagesAmong,
  releasePathsAmong,
} from './paths-of-a-branch.ts';
import type { Report, } from './report.ts';
import {
  errorLinesOf,
  runTaskInACgroup,
  tailOf,
  tallyLineOf,
} from './task-in-a-cgroup.ts';
import type { ProgramOutput, } from './run-a-program.ts';

/**
 What a check runs, which is the whole difference between one kind of branch and another.
 */
export type CheckKind =
  /**
   The wrapper's two test targets, compiled and not run, because `package/git-policy/cli` is what every commit goes through.
   */
  | 'wrapper'
  /**
   The cheapest declared task of each package the branch changed.
   */
    | 'package'
  /**
   Exactly the tasks the caller names.
   */
    | 'tasks'
  /**
   Nothing, which is what a documentation branch gets after its agent's report was read.
   */
    | 'none';

/**
 One task a check ran, and what it said.
 */
export type TaskOutcome = {
  /**
   The task, repository-qualified.
   */
  readonly task: string;

  /**
   The status it exited with.
   */
  readonly status: number;

  /**
   The one line a report keeps for it.
   */
  readonly line: string;

  /**
   What to show when it failed, empty when it did not.
   */
  readonly detail: string;
};

/**
 What a check found.
 */
export type CheckOutcome =
  /**
   `main` was already in the branch, so nothing is new since the agent's own checks.
   */
  | {
    readonly check: 'nothing-new';
    readonly branch: string
  }
  /**
   Every task the check ran passed.
   */
    | {
      readonly check: 'passed';
      readonly branch: string;
      readonly tasks: readonly TaskOutcome[]
    }
  /**
   A task failed, so the branch must not land.
   */
    | {
      readonly check: 'failed';
      readonly branch: string;
      readonly tasks: readonly TaskOutcome[]
    }
  /**
   `main` would not merge into the branch, so its conflicts stop the landing.
   */
    | {
      readonly check: 'merge-failed';
      readonly branch: string;
      readonly output: string
    }
  /**
   Nothing would check this branch, so it was not checked and must not land by this route.
   */
    | {
      readonly check: 'nothing-checked-it';
      readonly branch: string
    };

/**
 Tells whether a check lets its branch land.

 @param outcome - what the check found

 @returns true only when the check passed or found nothing new

 @example
 ```ts
 if (!checkPassed(outcome,)) return;
 ```
 */
export function checkPassed(outcome: CheckOutcome,): boolean {
  return (outcome.check === 'passed') || (outcome.check === 'nothing-new');
}

/**
 Reads the paths a branch changed since `main`, and says nothing about them when Git cannot answer.

 @param repository - the checkout to ask

 @param branch - the branch

 @returns the changed paths, or none when Git could not list them

 @example
 ```ts
 const changed = await changedPathsOf({ repository, branch: 'fix/a', },);
 ```
 */
async function changedPathsOf({
  repository,
  branch,
}: {
  readonly repository: Repository;
  readonly branch: string;
},): Promise<Looked<readonly string[]>> {
  try {
    return found(await changedPathsSinceMain({
      repository,
      branch,
    },),);
  }
  catch (caught) {
    // A branch Git cannot describe is one whose packages cannot be derived, which leaves the caller's own tasks to check it.
    return missing(caughtValueText(caught,),);
  }
}

/**
 Says which paths a branch changed that a release gate depends on, so a gate that must run again is named before the landing.

 @param report - where to report

 @param changedPaths - the paths the branch changed

 @example
 ```ts
 reportGuardedPaths({ report, changedPaths: ['Cargo.toml',], },);
 ```
 */
function reportGuardedPaths({
  report,
  changedPaths,
}: {
  readonly report: Report;
  readonly changedPaths: Looked<readonly string[]>;
},): void {
  /**
   The paths among them a release depends on.
   */
  const guarded = changedPaths.found ? releasePathsAmong(changedPaths.value,) : [];
  if (guarded.length === 0) return;
  report.line(`release paths changed by this branch: ${guarded.join(' ',)}`,);
  report.line(A_RELEASE_GATE_IS_INVALIDATED,);
}

/**
 Names the tasks that check the packages a branch changed, adding the ones the caller named.

 @param worktree - the worktree's folder name

 @param repository - the checkout to ask

 @param branch - the branch

 @param tasks - tasks the caller names

 @returns the tasks to run, each once

 @example
 ```ts
 const tasks = await packageCheckTasks({ worktree, repository, branch: 'fix/a', tasks: [], },);
 ```
 */
async function packageCheckTasks({
  worktree,
  repository,
  branch,
  tasks,
}: {
  readonly worktree: string;
  readonly repository: Repository;
  readonly branch: string;
  readonly tasks: readonly string[];
},): Promise<readonly string[]> {
  /**
   The paths the branch changed.
   */
  const changed = await changedPathsOf({
    repository,
    branch,
  },);
  return [...new Set([
    ...await checkTasksFor({
      worktree,
      packages: changed.found ? packagesAmong(changed.value,) : [],
    },),
    ...tasks,
  ],),];
}

/**
 Runs one task of a check and reports what it said.

 @param repository - the checkout to ask

 @param task - the task

 @param kind - the kind of check

 @param report - where to report

 @returns what the task said

 @example
 ```ts
 const outcome = await runOneCheckTask({ repository, task, kind: 'tasks', report, },);
 ```
 */
async function runOneCheckTask({
  repository,
  task,
  kind,
  report,
}: {
  readonly repository: Repository;
  readonly task: string;
  readonly kind: CheckKind;
  readonly report: Report;
},): Promise<TaskOutcome> {
  /**
   What the task printed and how it ended.
   */
  const ran: ProgramOutput = await runTaskInACgroup({
    repository,
    task,
    argumentsAfterTask: kind === 'wrapper' ? [
      '--',
      COMPILE_ONLY_ARGUMENT,
    ] : [],
    limits: kind === 'wrapper' ? WORKTREE_COMPILE_LIMITS : PACKAGE_TASK_LIMITS,
    collect: kind === 'wrapper',
  },);
  /**
   Everything the task printed.
   */
  const output = `${ran.stdout}${ran.stderr}`;
  if (kind === 'wrapper') {
    report.line(`status of the ${task} compile: ${String(ran.status,)}`,);
    /**
     The lines naming what broke, which a compile reports as its answer.
     */
    const errors = errorLinesOf({
      output,
      limit: TASK_ERROR_LINES,
    },);
    if (errors.length > 0) report.line(errors.join('\n',),);
    return {
      task,
      status: ran.status,
      line: `status of the ${task} compile: ${String(ran.status,)}`,
      detail: errors.join('\n',),
    };
  }
  /**
   The line naming the task and its counts.
   */
  const line = tallyLineOf({
    task,
    status: ran.status,
    output,
  },);
  report.line(line,);
  if (ran.status === 0) {
    return {
      task,
      status: ran.status,
      line,
      detail: '',
    };
  }
  /**
   The end of what it printed, which is where a failing suite explains itself.
   */
  const detail = tailOf({
    output,
    characters: TASK_TAIL_CHARACTERS,
  },);
  report.line(detail,);
  return {
    task,
    status: ran.status,
    line,
    detail,
  };
}
/**
 Merges `main` into an agent's worktree and runs the check its branch asks for.

 @param repository - the checkout to ask

 @param kind - the kind of check

 @param tasks - tasks the caller names

 @param report - where to report

 @returns what the check found

 @throws LandingConfigurationError when the worktree has no branch checked out

 @example
 ```ts
 const outcome = await checkABranch({ repository: worktree, kind: 'wrapper', tasks: [], report, },);
 ```
 */
export async function checkABranch({
  repository,
  kind,
  tasks,
  report,
}: {
  readonly repository: Repository;
  readonly kind: CheckKind;
  readonly tasks: readonly string[];
  readonly report: Report;
},): Promise<CheckOutcome> {
  /**
   The branch the worktree has checked out.
   */
  const branch = await currentBranchOf(repository,);
  if (branch === '') {
    throw new LandingConfigurationError(
      `${repository.folder} is on a detached HEAD, so checking it would not check its branch; switch it to the branch first`,
    );
  }
  report.line(`branch ${branch}`,);
  if (kind === 'none') return {
    check: 'nothing-checked-it',
    branch,
  };
  /**
   The commit the worktree is at before `main` is merged into it.
   */
  const before = await headOf(repository,);
  /**
   What merging `main` into the worktree left.
   */
  const merged = await mergeMainInto(repository,);
  if (merged.status !== 0) {
    return {
      check: 'merge-failed',
      branch,
      output: tailOf({
        output: `${merged.stdout}${merged.stderr}`,
        characters: MERGE_OUTPUT_CHARACTERS,
      },),
    };
  }
  reportGuardedPaths({
    report,
    changedPaths: await changedPathsOf({
      repository,
      branch,
    },),
  },);
  if ((await headOf(repository,)) === before) {
    report.line('main was already in this branch, so nothing is new since the agent\'s own checks',);
    return {
      check: 'nothing-new',
      branch,
    };
  }
  /**
   The tasks this check runs.
   */
  const chosen = kind === 'package'
    ? await packageCheckTasks({
      worktree: repository.folder,
      repository,
      branch,
      tasks,
    },)
    : kind === 'wrapper'
      ? [
        ...WRAPPER_COMPILE_TASKS,
        ...tasks,
      ]
      : [...tasks,];
  if (chosen.length === 0) return {
    check: 'nothing-checked-it',
    branch,
  };
  /**
   What each task said, one after another, so two compiles never share the host at once.
   */
  const outcomes: TaskOutcome[] = [];
  for (const task of chosen) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one task at a time, so two compiles never share the host
    outcomes.push(await runOneCheckTask({
      repository,
      task,
      kind,
      report,
    },),);
  }
  /**
   How many of them failed.
   */
  const failures = outcomes.filter(function failed(outcome: TaskOutcome,) {
    return outcome.status !== 0;
  },)
    .length;
  report.line(`tasks failed ${String(failures,)} of ${String(outcomes.length,)}`,);
  return {
    check: failures === 0 ? 'passed' : 'failed',
    branch,
    tasks: outcomes,
  };
}
