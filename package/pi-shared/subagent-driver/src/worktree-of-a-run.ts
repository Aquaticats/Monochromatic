/**
 The worktree a run works in: making it, with its branch started at the commit or branch the caller names, and removing it once
 the run has finished its work and holds nothing uncommitted.

 The owner, 2026-11-10: "You're the coordinator and you're supposed to remove finished worktrees automatically." A worktree
 added on btrfs is a snapshot of `main`, so it keeps every file `main` held when it was taken, build folders included: finished
 worktrees left in place filled the disk. The wrapper's `git worktree remove` refuses a worktree holding uncommitted work unless
 it is forced, and this never forces.

 @module
 */

import {
  access,
  rmdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { CommandOutput, } from './run-command.ts';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 How one git command is run, which a test replaces with the answers a repository would have given.
 */
export type GitRunner = (request: {
  readonly words: readonly string[];
  readonly folder: string;

  /**
   How long this question may take, when it needs a bound of its own: making a worktree copies the checkout, which a loaded
   machine stretches far past what any other git question costs.
   */
  readonly timeoutMilliseconds?: number;
}) => Promise<CommandOutput>;

//region Constants

/**
 How long `git worktree add` may take, in minutes: making a worktree on this machine copies the checkout's ignored state, and the
 wrapper says so while it copies. Fifteen minutes is a bound rather than a measurement, sized for a loaded machine: on
 2026-11-11 several adds ran at once, the wrapper reported one still copying after 110 seconds, and the two-minute bound every
 other git question gets killed it, which lost two reviews.
 */
const WORKTREE_ADD_MINUTES = 15;

/**
 Milliseconds in one minute.
 */
const MILLISECONDS_PER_MINUTE = 60_000;

/**
 How long `git worktree add` may take.
 */
export const WORKTREE_ADD_TIMEOUT_MILLISECONDS: number = WORKTREE_ADD_MINUTES * MILLISECONDS_PER_MINUTE;

/**
 How many times one start tries to make its worktree. A first attempt killed by its own bound leaves a partly made worktree and
 a registered path behind, so the next attempt starts by clearing what this start made, and one kill does not lose the run.
 */
const WORKTREE_ADD_ATTEMPTS = 2;

//endregion Constants

/**
 The commit or branch a run's new branch starts at when the caller names none, which is what every launcher started from before a
 run could name its own.
 */
export const BASE_OF_A_RUN_NAMING_NONE = 'main';

/**
 A worktree that is there but holds something, so it is not a worktree of this run's.
 */
export class WorktreeFolderOccupiedError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'WorktreeFolderOccupiedError';
}

/**
 A git command the launcher needed failed, so the run cannot start.
 */
export class GitCommandFailedError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'GitCommandFailedError';
}

/**
 Says whether a path is there, without reading it.

 @param path - the path to look for

 @returns whether it is there

 @example
 ```ts
 const there = await isThere('/repo/.claude/worktrees/pi-a/.git',);
 ```
 */
async function isThere(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 The end reasons of a run that finished its work, which are the only runs whose worktree is removed.
 */
const FINISHED_END_REASONS: ReadonlySet<string> = new Set<string>([
  'idle',
  'stop-file',
]);

/**
 The first lines of `git status` a note carries, so a long listing stays readable.
 */
const STATUS_LINES_SHOWN = 5;

/**
 What is to be done with a run's worktree once its unit has ended.
 */
export type WorktreeOutcome =
  | {
    /**
     The worktree is removed, which this command then does.
     */
    readonly action: 'remove';
  }
    | {
      readonly action: 'keep';

    /**
     The line that says why it stays.
     */
      readonly line: string;
    }
    | {
    /**
     Nothing is done and nothing is said: the run works in the checkout itself.
     */
      readonly action: 'keep-quietly';
    };

/**
 Says whether the branch named on a run's command line already exists, which decides whether its worktree adds it or starts it
 from `main`.

 @param git - how one git command is run

 @param checkoutFolder - the checkout whose branches are read

 @param branch - the branch the run's work goes on

 @returns whether the branch exists

 @example
 ```ts
 const there = await branchExists({ git, checkoutFolder: '/repo', branch: 'feat/x', },);
 ```
 */
export async function branchExists({
  git,
  checkoutFolder,
  branch,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly branch: string;
},): Promise<boolean> {
  /**
   What git answered about the branch.
   */
  const answer = await git({
    folder: checkoutFolder,
    words: [
      'rev-parse',
      '--verify',
      '--quiet',
      `refs/heads/${branch}`,
    ],
  },);
  return answer.status === 0;
}

/**
 Makes a run's worktree, unless it is already there.

 A branch this call creates starts at `base`, which is a commit or a branch: a fix for a reviewed branch starts at that branch, and
 a review starts at the commit it reviews. A branch that already exists keeps its own commits, so `base` does not move it: the
 scratch launcher left an existing branch alone the same way, and moving it would rewrite work someone else may be reading. The
 note this answers with says which of the two happened, so a reader is never left to guess which commit a run's branch began at.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path

 @param branch - the branch the run's work goes on

 @param base - the commit or branch a branch this call creates starts at, which an existing branch ignores

 @returns what was done, in words for a reader

 @throws WorktreeFolderOccupiedError when the path holds a folder that is no worktree and nothing was deleted to make room

 @throws GitCommandFailedError when git refused to make the worktree

 @example
 ```ts
 const note = await makeWorktree({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', branch: 'feat/a', base: 'main', },);
 ```
 */
export async function makeWorktree({
  git,
  checkoutFolder,
  worktree,
  branch,
  base,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
  readonly branch: string;
  readonly base: string;
},): Promise<string> {
  if (await isThere(join(
    worktree,
    '.git',
  ),)) {
    return 'worktree: already there';
  }
  if (await isThere(worktree,)) {
    try {
      // A folder inside the repository without `.git` is no worktree, and git run there would act on the checkout's own files.
      // rmdir removes only an empty folder, so nothing anyone wrote can be lost here.
      await rmdir(worktree,);
    }
    catch (error) {
      throw new WorktreeFolderOccupiedError(
        `${worktree} is there but is no worktree (it holds no .git), and it holds something, so it was left as it is. Remove what is in it if nothing there is wanted. ${String(error,)}`,
        { cause: error, },
      );
    }
  }
  /**
   Whether the branch the run works on was there before this start asked anything, which says which of the two things the add
   did: one killed after `-b` leaves its branch behind, so a later attempt takes that branch instead of asking again.
   */
  const branchWasThere = await branchExists({
    git,
    checkoutFolder,
    branch,
  },);

  /**
   Makes the worktree, and clears what a killed attempt left before the next one.

   @returns what git answered to the attempt that ended the loop

   @throws GitCommandFailedError when every attempt failed, which cannot happen because the last one returns its answer

   @example
   ```ts
   const added = await addTheWorktree();
   ```
   */
  async function addTheWorktree(): Promise<CommandOutput> {
    /* oxlint-disable eslint/no-await-in-loop -- one attempt, then the clearing of what it left, then the next attempt: the
       second attempt depends on the first having ended. See doc/lint/sequential-awaits.md. */
    for (let attempt = 1; attempt <= WORKTREE_ADD_ATTEMPTS; attempt += 1) {
      // Asked again on each attempt: an attempt killed after `-b` made the branch leaves it there, and the next attempt must
      // take that branch rather than fail on a name that already exists.
      /**
       Whether the branch the run works on is there now.
       */
      const branchIsThereNow = (attempt === 1)
        ? branchWasThere
        : await branchExists({
          git,
          checkoutFolder,
          branch,
        },);
      /**
       What git answered this attempt.
       */
      const attempted = await git({
        folder: checkoutFolder,
        words: branchIsThereNow
          ? [
            'worktree',
            'add',
            worktree,
            branch,
          ]
          : [
            'worktree',
            'add',
            '-b',
            branch,
            worktree,
            base,
          ],
        timeoutMilliseconds: WORKTREE_ADD_TIMEOUT_MILLISECONDS,
      },);
      if ((attempted.status === 0) || (attempt === WORKTREE_ADD_ATTEMPTS)) {
        return attempted;
      }
      // A failed add may have made the folder and registered the path before its bound killed it. Both are this start's own,
      // because the folder held no worktree when this began, so they are cleared and the attempt is made again.
      await git({
        folder: checkoutFolder,
        words: [
          'worktree',
          'remove',
          '--force',
          worktree,
        ],
        // Removing what a killed add left is the same copy the add was making, so it gets the same bound.
        timeoutMilliseconds: WORKTREE_ADD_TIMEOUT_MILLISECONDS,
      },);
    }
    /* oxlint-enable eslint/no-await-in-loop */
    // The loop returns its last attempt, so this is reached only if the count is ever set below one.
    throw new GitCommandFailedError(`git worktree add was attempted ${String(WORKTREE_ADD_ATTEMPTS,)} times for ${worktree}, which is no number of attempts`,);
  }
  /**
   What git answered about making the worktree.
   */
  const added = await addTheWorktree();
  if (added.status !== 0) {
    throw new GitCommandFailedError(
      `git worktree add failed for ${worktree} after ${String(WORKTREE_ADD_ATTEMPTS,)} attempts: ${added.problem
        ?? added.stderr
          .trim()}`,
    );
  }
  return branchWasThere
    ? `worktree add: status ${String(added.status,)} (existing branch ${branch}, which keeps its own commits, so the base ${base} was not used)`
    : `worktree add: status ${String(added.status,)} (new branch ${branch} at ${base})`;
}

/**
 Says what is to be done with a run's worktree, given how the run ended and whether the worktree holds anything.

 Only a run that finished its work on a branch, with nothing uncommitted, has its worktree removed: one that ended on a cap or an
 error is likely to be resumed, and a detached HEAD's commits may be reachable from this worktree alone.

 @param readOnlyInMain - whether the run works in the checkout itself

 @param endReason - why the run ended, or nothing when it wrote no summary

 @param onABranch - whether the worktree's HEAD is a branch rather than detached

 @param status - how `git status` ended, which matters only when it printed nothing

 @param statusText - what `git status --porcelain` printed

 @returns what to do, and why it was decided so

 @example
 ```ts
 const outcome = worktreeOutcomeOf({ readOnlyInMain: false, endReason: 'idle', onABranch: true, status: 0, statusText: '', },);
 ```
 */
export function worktreeOutcomeOf({
  readOnlyInMain,
  endReason,
  onABranch,
  status,
  statusText,
}: {
  readonly readOnlyInMain: boolean;
  readonly endReason?: string;
  readonly onABranch: boolean;
  readonly status: number;
  readonly statusText: string;
},): WorktreeOutcome {
  if (readOnlyInMain) {
    return { action: 'keep-quietly', };
  }
  /**
   Whether the run finished its work, which is the first thing a removal needs.
   */
  const finishedItsWork = (endReason !== undefined) && FINISHED_END_REASONS.has(endReason,);
  if ((!finishedItsWork) || (!onABranch)) {
    return {
      action: 'keep',
      line: `worktree kept (run ended ${String(endReason,)}, ${onABranch ? 'on a branch' : 'detached HEAD'})`,
    };
  }
  if ((status !== 0) || (statusText.trim() !== '')) {
    return {
      action: 'keep',
      line: `worktree kept, it holds uncommitted work (git status ${String(status,)}): ${statusText.trim()
        .split('\n',)
        .slice(
          0,
          STATUS_LINES_SHOWN,
        )
        .join(', ',)}`,
    };
  }
  return { action: 'remove', };
}

/**
 Removes a run's worktree, never by force.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree was made in

 @param worktree - the worktree's path

 @returns what git answered, in words for a reader

 @example
 ```ts
 const note = await removeWorktree({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function removeWorktree({
  git,
  checkoutFolder,
  worktree,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
},): Promise<string> {
  /**
   What git answered about removing the worktree.
   */
  const removed = await git({
    folder: checkoutFolder,
    words: [
      'worktree',
      'remove',
      worktree,
    ],
  },);
  return `worktree remove status ${String(removed.status,)}${(removed.status === 0) ? '' : `: ${removed.stderr
    .trim()}`}`;
}
