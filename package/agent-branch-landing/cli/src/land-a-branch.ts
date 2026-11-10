/**
 Landing one branch on `main` in the main checkout: merge it without committing, build the merged tree when it touches the Git
 wrapper, commit through the wrapper, and report what is still not on `main`.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  A_MERGE_IS_OPEN_AFTER_A_FAILED_BUILD,
  A_MERGE_IS_OPEN,
  A_RELEASE_GATE_IS_INVALIDATED,
  COMMIT_PROBLEM_CHARACTERS,
  MERGE_OUTPUT_CHARACTERS,
  MERGE_SUBJECT_CHARACTERS,
  NO_MERGE_WAS_LEFT_OPEN,
  REFLOW_COMMIT_MESSAGE,
  TASK_ERROR_LINES,
  WRAPPER_COMPILE_LIMITS,
  WRAPPER_COMPILE_TASKS,
  WRAPPER_PATH_PREFIX,
} from './constants.ts';
import {
  changedPathsSinceMain,
  commitsOfBranchNotOnMain,
  lastCommitOnMain,
  mergeIsOpen,
  type Repository,
  stagedPaths,
} from './git-commands.ts';
import {
  commitStaged,
  mergeWithoutCommitting,
} from './git-changes.ts';
import {
  found,
  type Looked,
  missing,
} from './lookups.ts';
import {
  landingReportOf,
  type LandingReport,
} from './landing-report.ts';
import type { Report, } from './report.ts';
import {
  errorLinesOf,
  runTaskInACgroup,
  tailOf,
} from './task-in-a-cgroup.ts';
import type { ProgramOutput, } from './run-a-program.ts';
import {
  blockedFilesIn,
  type KindOfUncommittedEdit,
  kindOfUncommittedEdit,
} from './uncommitted-edits.ts';

/**
 What landing one branch came to.
 */
export type LandingOutcome =
  /**
   The merge is committed on `main`.
   */
  | {
    readonly landing: 'landed';
    readonly report: LandingReport
  }
  /**
   A merge was already open in the main checkout, so nothing was attempted.
   */
    | { readonly landing: 'refused-merge-already-open' }
  /**
   Uncommitted edits the merge would overwrite change words, so they stay and nothing landed.
   */
    | {
      readonly landing: 'refused-uncommitted-edits';
      readonly paths: readonly string[]
    }
  /**
   Git would not merge the branch.
   */
    | {
      readonly landing: 'merge-failed';
      readonly mergeIsOpen: boolean;
      readonly output: string;
    }
  /**
   The merged tree does not build, and the merge is left open for a person to fix or abort.
   */
    | {
      readonly landing: 'tree-does-not-compile';
      readonly errors: string
    }
  /**
   Every try at committing the merge was refused, so the merge is still open.
   */
    | {
      readonly landing: 'commit-refused';
      readonly tries: number
    };

/**
 Tells whether a landing put the branch on `main` completely.

 @param outcome - what the landing came to

 @returns true when the merge is committed and no commit of the branch is left behind

 @example
 ```ts
 if (!landedCompletely(outcome,)) report.line('not fully landed',);
 ```
 */
export function landedCompletely(outcome: LandingOutcome,): boolean {
  return (outcome.landing === 'landed') && (outcome.report
    .commitsNotOnMain
    === 0);
}

/**
 Reports a merge Git refused, and whether it left one open, which is the state a caller must resolve before anything else.

 @param repository - the checkout to ask

 @param output - what it printed

 @param report - where to report

 @returns the failed landing

 @example
 ```ts
 return mergeFailed({ repository, output: 'CONFLICT', report, },);
 ```
 */
async function mergeFailed({
  repository,
  output,
  report,
}: {
  readonly repository: Repository;
  readonly output: string;
  readonly report: Report;
},): Promise<LandingOutcome> {
  /**
   Whether the refused merge left a merge open.
   */
  const stillOpen = await mergeIsOpen(repository,);
  report.line(`merge failed:\n${output.slice(
    0,
    MERGE_OUTPUT_CHARACTERS,
  )}`,);
  report.line(stillOpen ? A_MERGE_IS_OPEN : NO_MERGE_WAS_LEFT_OPEN,);
  return {
    landing: 'merge-failed',
    mergeIsOpen: stillOpen,
    output,
  };
}

/**
 Builds the merged tree when the merge touches the wrapper, because a branch tested before another landed has never been built
 with it.

 Two branches that each passed broke `main` this way: `367c52c390` on top of `e590c0554d`.

 @param repository - the checkout to ask

 @param report - where to report

 @returns the errors when it does not build, or nothing when it does or nothing needed building

 @example
 ```ts
 const failure = await wrapperBuildFailure({ repository, report, },);
 ```
 */
async function wrapperBuildFailure({
  repository,
  report,
}: {
  readonly repository: Repository;
  readonly report: Report;
},): Promise<Looked<string>> {
  /**
   The paths the merge staged.
   */
  const staged = await stagedPaths(repository,);
  if (!staged.some(function isInTheWrapper(path: string,) {
    return path.startsWith(WRAPPER_PATH_PREFIX,);
  },)) return missing('the merge touches nothing the wrapper build covers',);
  for (const task of WRAPPER_COMPILE_TASKS) {
    /**
     What the build of that target printed.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- both targets one after another, so one build owns the host
    const ran = await runTaskInACgroup({
      repository,
      task,
      argumentsAfterTask: [
        '--',
        '--no-run',
      ],
      limits: WRAPPER_COMPILE_LIMITS,
      collect: false,
    },);
    if (ran.status === 0) continue;
    /**
     The lines naming what broke.
     */
    const errors = errorLinesOf({
      output: `${ran.stdout}${ran.stderr}`,
      limit: TASK_ERROR_LINES,
    },);
    report.line(`status of the ${task} compile: ${String(ran.status,)}`,);
    return found(errors.length === 0 ? `${task} ended with status ${String(ran.status,)}` : errors.join('\n',),);
  }
  return missing('both of the wrapper targets compiled',);
}

/**
 Commits the merge, trying again while the wrapper's scratch commit ceiling refuses it.

 The commit is done only when the merge marker is gone.

 @param repository - the checkout to ask

 @param message - the message its commit carries

 @param tries - how many times its commit is tried

 @param pauseMilliseconds - milliseconds between two tries

 @param commitTimeoutMilliseconds - milliseconds one commit may take

 @param report - where to report

 @returns whether the merge is committed

 @example
 ```ts
 const committed = await commitOnMain({ repository, message, tries, pauseMilliseconds, commitTimeoutMilliseconds, report, },);
 ```
 */
async function commitOnMain({
  repository,
  message,
  tries,
  pauseMilliseconds,
  commitTimeoutMilliseconds,
  report,
}: {
  readonly repository: Repository;
  readonly message: string;
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
  readonly report: Report;
},): Promise<boolean> {
  for (let attempt = 1; attempt <= tries; attempt += 1) {
    /**
     What the commit said.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one commit at a time, since each try must see whether the last one landed
    const committed = await commitStaged({
      repository,
      message,
      paths: [],
      timeoutMilliseconds: commitTimeoutMilliseconds,
    },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the commit is done only once the merge marker is gone
    if (!(await mergeIsOpen(repository,))) return true;
    report.line(`commit try ${String(attempt,)}: status ${String(committed.status,)}: ${committed.stderr
      .slice(
        0,
        COMMIT_PROBLEM_CHARACTERS,
      )
      .replaceAll(
        '\n',
        ' ',
      )}`,);
    // oxlint-disable-next-line eslint/no-await-in-loop -- the wait between tries is the design, see doc/lint/sequential-lands.md
    await wait(pauseMilliseconds,);
  }
  report.line(`A MERGE IS OPEN IN THE MAIN CHECKOUT after ${String(tries,)} commit tries`,);
  return false;
}

/**
 Merges the branch, committing the reflow a format run left in the main checkout when that is the only thing in the way.

 Two shapes come back: the merge the landing goes on with, or the refusal that ends the landing here.

 @param repository - the main checkout

 @param branch - the branch to merge

 @param commitTimeoutMilliseconds - milliseconds one commit may take

 @param report - where to report

 @returns the merge to go on with, or the refusal that ends it

 @example
 ```ts
 const attempt = await mergeAfterBlockedEdits({ repository, branch: 'fix/a', commitTimeoutMilliseconds: 120_000, report, },);
 ```
 */
async function mergeAfterBlockedEdits({
  repository,
  branch,
  commitTimeoutMilliseconds,
  report,
}: {
  readonly repository: Repository;
  readonly branch: string;
  readonly commitTimeoutMilliseconds: number;
  readonly report: Report;
},): Promise<{
  readonly merged: ProgramOutput;
  readonly refused?: LandingOutcome;
}> {
  /**
   What the first try at the merge left.
   */
  const first = await mergeWithoutCommitting({
    repository,
    branch,
  },);
  /**
   The paths the refused merge would overwrite.
   */
  const blocked = blockedFilesIn(`${first.stdout}${first.stderr}`,);
  if ((first.status === 0) || (blocked.length === 0)) return { merged: first, };
  /**
   What each blocked edit does to its file.
   */
  const kinds: readonly {
    readonly path: string;
    readonly kind: KindOfUncommittedEdit;
  }[] = await Promise.all(blocked.map(async function kindOf(path: string,) {
    return {
      path,
      kind: await kindOfUncommittedEdit({
        repository,
        branch,
        path,
      },),
    };
  },),);
  /**
   The blocked paths whose edit changes words, which stay where they are.
   */
  const changedWords = kinds.filter(function changesWords(each: {
    readonly path: string;
    readonly kind: KindOfUncommittedEdit;
  },) {
    return each.kind === 'changed-words';
  },);
  if (changedWords.length > 0) {
    /**
     The paths that stay, as the report names them.
     */
    const paths = changedWords.map(function pathOf(each: {
      readonly path: string;
      readonly kind: KindOfUncommittedEdit;
    },) {
      return each.path;
    },);
    report.line(`stopped: these uncommitted edits change words, so they stay: ${paths.join(' ',)}`,);
    return {
      merged: first,
      refused: {
        landing: 'refused-uncommitted-edits',
        paths,
      },
    };
  }
  /**
   What committing the reflow left.
   */
  const reflowed = await commitStaged({
    repository,
    message: REFLOW_COMMIT_MESSAGE,
    paths: blocked,
    timeoutMilliseconds: commitTimeoutMilliseconds,
  },);
  if (reflowed.status === 0) {
    report.line(`committed reflow edits as they stand: ${blocked.join(' ',)}`,);
  }
  else {
    report.line(`the reflow would not commit: ${`${reflowed.stdout}${reflowed.stderr}`.trim()
      .slice(
        0,
        COMMIT_PROBLEM_CHARACTERS,
      )}`,);
  }
  return {
    merged: await mergeWithoutCommitting({
      repository,
      branch,
    },),
  };
}

/**
 Lands one branch on `main`.

 @param repository - the checkout to ask

 @param branch - the branch

 @param message - the message its commit carries

 @param tries - how many times its commit is tried

 @param pauseMilliseconds - milliseconds between two tries

 @param commitTimeoutMilliseconds - milliseconds one commit may take

 @param report - where to report

 @returns what the landing came to

 @example
 ```ts
 const outcome = await landABranch({ repository, branch: 'fix/a', message: 'merge(a): b', tries: 15, pauseMilliseconds: 20_000, commitTimeoutMilliseconds: 120_000, report, },);
 ```
 */
export async function landABranch({
  repository,
  branch,
  message,
  tries,
  pauseMilliseconds,
  commitTimeoutMilliseconds,
  report,
}: {
  readonly repository: Repository;
  readonly branch: string;
  readonly message: string;
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
  readonly report: Report;
},): Promise<LandingOutcome> {
  if (await mergeIsOpen(repository,)) {
    report.line(`a merge is already open in ${repository.folder}: resolve it, or run git merge --abort, before anything else commits there`,);
    return { landing: 'refused-merge-already-open', };
  }
  /**
   What the merge left once any blocked reflow was committed.
   */
  const attempt = await mergeAfterBlockedEdits({
    repository,
    branch,
    commitTimeoutMilliseconds,
    report,
  },);
  if (attempt.refused !== undefined) return attempt.refused;
  if (attempt.merged
    .status
    !== 0) {
    return mergeFailed({
      repository,
      output: `${attempt.merged
        .stdout}${attempt.merged
          .stderr}`,
      report,
    },);
  }
  /**
   What the merged tree does when the merge touches the wrapper.
   */
  const buildFailure = await wrapperBuildFailure({
    repository,
    report,
  },);
  if (buildFailure.found) {
    report.line(`the merged tree does not build:\n${buildFailure.value}`,);
    report.line(A_MERGE_IS_OPEN_AFTER_A_FAILED_BUILD,);
    return {
      landing: 'tree-does-not-compile',
      errors: buildFailure.value,
    };
  }
  if (!(await commitOnMain({
    repository,
    message,
    tries,
    pauseMilliseconds,
    commitTimeoutMilliseconds,
    report,
  },))) {
    return {
      landing: 'commit-refused',
      tries,
    };
  }
  return {
    landing: 'landed',
    report: await landingReportOf({
      repository,
      branch,
      report,
    },),
  };
}
