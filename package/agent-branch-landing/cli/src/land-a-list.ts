/**
 Landing a list of branches, one at a time: each is checked on the merged tree when its entry asks for a check, landed when that
 passed, and reported. A landing that does not finish stops the run, because later branches must not land on a broken `main`.

 @module
 */

import {
  A_RELEASE_GATE_IS_INVALIDATED,
  LOCKFILE_PATH,
  OFFLINE_INSTALL_LIMITS,
  OFFLINE_INSTALL_WORDS,
  SYSTEMD_RUN_PROGRAM,
  worktreeFolderOf,
} from './constants.ts';
import {
  askGit,
  changedPathsSinceMain,
  type Repository,
} from './git-commands.ts';
import { LandingConfigurationError, } from './errors.ts';
import {
  checkABranch,
  checkPassed,
  type CheckOutcome,
} from './check-a-branch.ts';
import { landABranch, } from './land-a-branch.ts';
import type { LandingEntry, } from './landing-list.ts';
import type { Report, } from './report.ts';

/**
 What one entry of a list came to.
 */
export type EntryOutcome = {
  /**
   The branch the entry named or its worktree had checked out.
   */
  readonly branch: string;

  /**
   What happened to it.
   */
  readonly result:
    /**
     The merge is committed on `main`, with this many commits of the branch still not on it.
     */
    | 'landed'
    /**
     Nothing was landed and a later entry may still be, because this one left `main` alone.
     */
      | 'skipped'
    /**
     The landing did not finish, so the run stopped.
     */
      | 'not-landed';

  /**
   The one sentence a report keeps for it.
   */
  readonly reason: string;
};

/**
 What landing a list came to.
 */
export type ListOutcome = {
  /**
   What each entry came to, in the order attempted.
   */
  readonly entries: readonly EntryOutcome[];

  /**
   Whether the run stopped before its last entry.
   */
  readonly stopped: boolean;
};

/**
 Tells whether every entry of a list landed completely.

 @param outcome - what landing the list came to

 @returns true when no entry was skipped and none was left behind

 @example
 ```ts
 if (!listLandedCompletely(outcome,)) process.exitCode = 1;
 ```
 */
export function listLandedCompletely(outcome: ListOutcome,): boolean {
  return (outcome.entries
    .length
    > 0)
    && outcome.entries
      .every(function landed(entry: EntryOutcome,) {
        return entry.result === 'landed';
      },);
}

/**
 Builds the repository of a worktree an entry names.

 @param repository - the checkout to ask

 @param name - the worktree's folder name

 @returns the worktree's repository

 @example
 ```ts
 const worktree = worktreeRepositoryOf({ repository, name: 'pi-fix-a', },);
 ```
 */
export function worktreeRepositoryOf({
  repository,
  name,
}: {
  readonly repository: Repository;
  readonly name: string;
},): Repository {
  if (name.includes('/',) || (name === '.')
    || (name === '..')) {
    throw new LandingConfigurationError(`${name}: a worktree is named by its folder beside .claude/worktrees, not by a path`,);
  }
  return {
    ...repository,
    folder: `${worktreeFolderOf(repository.folder,)}/${name}`,
  };
}

/**
 Reads a worktree's branch, treating a worktree Git cannot read as one with no branch.

 @param repository - the worktree

 @returns the branch, or empty text

 @example
 ```ts
 const branch = await branchOfQuietly(worktree,);
 ```
 */
async function branchOfQuietly(repository: Repository,): Promise<string> {
  /**
   What Git answered.
   */
  const asked = await askGit({
    repository,
    words: [
      'branch',
      '--show-current',
    ],
  },);
  return asked.status === 0 ? asked.stdout
    .trim() : '';
}

/**
 Names why a check skipped its branch.

 @param branch - the branch

 @param checked - what the check found

 @returns the sentence a report keeps

 @example
 ```ts
 const reason = skippedReasonOf({ branch: 'fix/a', checked, },);
 ```
 */
function skippedReasonOf({
  branch,
  checked,
}: {
  readonly branch: string;
  readonly checked: CheckOutcome;
},): string {
  return checked.check === 'nothing-checked-it'
    ? 'skipped: no task would check this branch, so it was not landed'
    : `skipped: ${branch} did not merge with main or did not pass its check on the merged tree`;
}

/**
 Installs a landed lockfile change offline in the main checkout, which is what keeps the next command's tools resolvable.

 @param repository - the checkout to ask

 @param branch - the branch

 @param report - where to report

 @example
 ```ts
 await installALandedLockfile({ repository, branch: 'fix/a', report, },);
 ```
 */
async function installALandedLockfile({
  repository,
  branch,
  report,
}: {
  readonly repository: Repository;
  readonly branch: string;
  readonly report: Report;
},): Promise<void> {
  /**
   The paths the branch changed.
   */
  const changed = await changedPathsSinceMain({
    repository,
    branch,
  },);
  if (!changed.includes(LOCKFILE_PATH,)) return;
  /**
   What the install said.
   */
  const installed = await repository.run({
    program: SYSTEMD_RUN_PROGRAM,
    words: [
      '--user',
      '--scope',
      '--quiet',
      `--property=CPUQuota=${OFFLINE_INSTALL_LIMITS.cpuQuota}`,
      `--property=MemoryMax=${OFFLINE_INSTALL_LIMITS.memoryMax}`,
      ...(OFFLINE_INSTALL_LIMITS.memorySwapMax === undefined ? [] : [`--property=MemorySwapMax=${OFFLINE_INSTALL_LIMITS.memorySwapMax}`,]),
      `--property=TasksMax=${OFFLINE_INSTALL_LIMITS.tasksMax}`,
      `--property=IOWeight=${OFFLINE_INSTALL_LIMITS.ioWeight}`,
      '--',
      'mise',
      ...OFFLINE_INSTALL_WORDS,
    ],
    folder: repository.folder,
    timeoutMilliseconds: repository.timeoutMilliseconds,
    outputBytes: repository.outputBytes,
  },);
  report.line(`offline install in the main checkout: status ${String(installed.status,)}`,);
}

/**
 Says how the list ended, naming every branch that did not land.

 @param outcomes - what each entry came to

 @param report - where to report

 @example
 ```ts
 reportSummary({ outcomes, report, },);
 ```
 */
function reportSummary({
  outcomes,
  report,
}: {
  readonly outcomes: readonly EntryOutcome[];
  readonly report: Report;
},): void {
  /**
   The entries that landed.
   */
  const landed = outcomes.filter(function isLanded(outcome: EntryOutcome,) {
    return outcome.result === 'landed';
  },);
  /**
   The entries that did not.
   */
  const left = outcomes.filter(function isLeft(outcome: EntryOutcome,) {
    return outcome.result !== 'landed';
  },);
  report.line(`landed ${String(landed.length,)} of ${String(outcomes.length,)} branches`,);
  if (left.length > 0) {
    report.line(left.map(function lineOf(outcome: EntryOutcome,) {
      return `${outcome.reason}: ${outcome.branch}`;
    },)
      .join('\n',),);
  }
}
/**
 Lands one entry of a list.

 @param repository - the checkout to ask

 @param entry - the entry

 @param tries - how many times its commit is tried

 @param pauseMilliseconds - milliseconds between two tries

 @param commitTimeoutMilliseconds - milliseconds one commit may take

 @param report - where to report

 @returns what the entry came to

 @example
 ```ts
 const outcome = await landOneEntry({ repository, entry, bounds, report, },);
 ```
 */
async function landOneEntry({
  repository,
  entry,
  tries,
  pauseMilliseconds,
  commitTimeoutMilliseconds,
  report,
}: {
  readonly repository: Repository;
  readonly entry: LandingEntry;
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
  readonly report: Report;
},): Promise<EntryOutcome> {
  /**
   The worktree's own repository, when the entry names one.
   */
  const worktree = entry.worktree === '' ? undefined : worktreeRepositoryOf({
    repository,
    name: entry.worktree,
  },);
  /**
   The branch to land.
   */
  const branch = entry.branch !== ''
    ? entry.branch
    : (worktree === undefined ? '' : await branchOfQuietly(worktree,));
  report.line(`=== ${branch === '' ? entry.worktree : branch}`,);
  if (branch === '') {
    return {
      branch,
      result: 'skipped',
      reason: 'skipped: no branch is checked out in that worktree',
    };
  }
  if (entry.note !== '') report.warning(`${branch}: ${entry.note}`,);
  if ((worktree !== undefined) && (entry.check !== 'none')) {
    /**
     What the check found.
     */
    const checked = await checkABranch({
      repository: worktree,
      kind: entry.check,
      tasks: entry.tasks,
      report,
    },);
    if (!checkPassed(checked,)) {
      return {
        branch,
        result: 'skipped',
        reason: skippedReasonOf({
          branch,
          checked,
        },),
      };
    }
  }
  /**
   What the landing came to.
   */
  const landed = await landABranch({
    repository,
    branch,
    message: entry.message,
    tries,
    pauseMilliseconds,
    commitTimeoutMilliseconds,
    report,
  },);
  if (landed.landing !== 'landed') {
    return {
      branch,
      result: 'not-landed',
      reason: `not landed: ${landed.landing}`,
    };
  }
  if (landed.report
    .commitsNotOnMain
    > 0) {
    return {
      branch,
      result: 'not-landed',
      reason: `landed, but ${String(landed.report
        .commitsNotOnMain,)} commits of the branch are not on main`,
    };
  }
  if (landed.report
    .releasePaths
    .length
    > 0) {
    report.line(`${branch}: ${landed.report
      .releasePaths
      .join(' ',)} are release paths; ${A_RELEASE_GATE_IS_INVALIDATED}`,);
  }
  await installALandedLockfile({
    repository,
    branch,
    report,
  },);
  return {
    branch,
    result: 'landed',
    reason: 'landed',
  };
}

/**
 Lands each entry of a list, one at a time.

 @param repository - the checkout to ask

 @param entries - the entries

 @param tries - how many times its commit is tried

 @param pauseMilliseconds - milliseconds between two tries

 @param commitTimeoutMilliseconds - milliseconds one commit may take

 @param report - where to report

 @returns what each entry came to, and whether the run stopped

 @example
 ```ts
 const outcome = await landAList({ repository, entries, bounds, report, },);
 ```
 */
export async function landAList({
  repository,
  entries,
  tries,
  pauseMilliseconds,
  commitTimeoutMilliseconds,
  report,
}: {
  readonly repository: Repository;
  readonly entries: readonly LandingEntry[];
  readonly tries: number;
  readonly pauseMilliseconds: number;
  readonly commitTimeoutMilliseconds: number;
  readonly report: Report;
},): Promise<ListOutcome> {
  /**
   What each entry came to.
   */
  const outcomes: EntryOutcome[] = [];
  for (const entry of entries) {
    /**
     What this entry came to.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one branch at a time, since each landing changes the main checkout the next check merges
    const outcome = await landOneEntry({
      repository,
      entry,
      tries,
      pauseMilliseconds,
      commitTimeoutMilliseconds,
      report,
    },);
    outcomes.push(outcome,);
    if (outcome.result === 'not-landed') {
      report.line('stopped: the landing did not finish, so later branches were not attempted',);
      return {
        entries: outcomes,
        stopped: true,
      };
    }
  }
  reportSummary({
    outcomes,
    report,
  },);
  return {
    entries: outcomes,
    stopped: false,
  };
}
