/**
 The worktree a run works in: making it, with its branch started at the commit or branch the caller names, and removing it once
 the run has finished its work and holds nothing uncommitted.

 The owner, 2026-11-10: "You're the coordinator and you're supposed to remove finished worktrees automatically." A worktree
 added on btrfs is a snapshot of `main`, so it keeps every file `main` held when it was taken, build folders included: finished
 worktrees left in place filled the disk. The wrapper's `git worktree remove` refuses a worktree holding uncommitted work unless
 it is forced, and this never forces. Nothing here removes a folder at all: a retry after a killed `git worktree add` prunes the
 registration that attempt may have left and asks git again, so the only thing that ever goes is a registration whose worktree is
 not there.

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
 How many times one start tries to make its worktree. A first attempt killed by its own bound can leave the path registered before
 it ended, so the next attempt starts by asking git to prune a registration whose worktree is not there, and one kill does not
 lose the run. A path that holds a worktree is used as it is instead, so nothing another start may have made is removed.
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
 The path a run's worktree belongs at holds a worktree with work nobody here made, so this start leaves it exactly as it is.

 This is the second start of one run name meeting the worktree the first one made, or a killed attempt of this start leaving a
 partly made worktree behind. Neither is this start's to use or to remove: a worktree one start works in holds work that start has
 not committed, and a removal would take it.
 */
export class WorktreeTakenError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'WorktreeTakenError';
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
 What asking git for a run's worktree came to.
 */
type WorktreeAttempt =
  | {
    /**
     What git answered to the attempt that ended the loop.
     */
    readonly kind: 'an-attempt-answered';

    /**
     The answer that attempt gave.
     */
    readonly answered: CommandOutput;
  }
    | {
    /**
     A worktree stood at the path holding nothing uncommitted, so it is used as it is instead of an attempt being made.
     */
      readonly kind: 'a-worktree-stood-there';
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
   Makes the worktree, and asks git again after an attempt that ended without making one.

   Nothing here removes anything. An attempt killed by its own bound leaves either no folder, an empty folder, or a registration
   behind, and `git worktree prune` clears a registration whose worktree is not there, which is git's own housekeeping and takes no
   folder with it. A path that holds a worktree is a worktree this run works in as it is: another start of this name makes the same
   path on the same branch, so removing it would delete whatever that start has not committed yet.

   @returns what came of the attempts: git's own answer to the attempt that ended the loop, or the word that a worktree stood
   there in place of an attempt

   @example
   ```ts
   const made = await addTheWorktree();
   ```
   */
  async function addTheWorktree(): Promise<WorktreeAttempt> {
    /* oxlint-disable eslint/no-await-in-loop -- one attempt, then what it left cleared, then the next attempt: the second attempt
       depends on the first having ended. See doc/lint/sequential-awaits.md. */
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
        return {
          kind: 'an-attempt-answered',
          answered: attempted,
        };
      }
      // A path that holds a worktree may hold another start's: a start of this name makes this path on this branch too, and the
      // attempt that just ended left the window open for one to take it. A worktree holding nothing uncommitted is used as it is,
      // which is what the next start of this name would do with it; one holding work somebody wrote is left exactly as it is,
      // because using it would work in another start's uncommitted work and removing it would delete that work.
      if (await isThere(join(
        worktree,
        '.git',
      ),)) {
        /**
         What git answered about what the worktree holds.
         */
        const held = await git({
          folder: worktree,
          words: [
            'status',
            '--porcelain',
          ],
        },);
        /**
         What the worktree holds, as git listed it, which is empty for a worktree nobody has written in yet.
         */
        const heldText = held.stdout
          .trim();
        if ((held.status !== 0) || (heldText !== '')) {
          throw new WorktreeTakenError(
            `${worktree} holds a worktree that cannot be shown to hold nothing uncommitted (git status ${String(held.status,)}: ${heldText}), after this start's attempt at making one ended without it, so it was left exactly as it is and nothing was removed. Another start of this run's name may have made it, or this start's own killed attempt may have left it partly made. Look at what it holds, and start this run again once it holds nothing uncommitted.`,
          );
        }
        return { kind: 'a-worktree-stood-there', };
      }
      // A killed add may have registered the path before it ended. Pruning clears such a registration, and it takes no folder and
      // no file with it: what is left in the folder is still there for the attempt that follows to answer about.
      await git({
        folder: checkoutFolder,
        words: [
          'worktree',
          'prune',
          '--expire=now',
        ],
        // Pruning reads every worktree of the repository, so it gets the same bound as the add it follows.
        timeoutMilliseconds: WORKTREE_ADD_TIMEOUT_MILLISECONDS,
      },);
    }
    /* oxlint-enable eslint/no-await-in-loop */
    // The loop returns its last attempt, so this is reached only if the count is ever set below one.
    throw new GitCommandFailedError(`git worktree add was attempted ${String(WORKTREE_ADD_ATTEMPTS,)} times for ${worktree}, which is no number of attempts`,);
  }
  /**
   What git answered about making the worktree, which is nothing when a worktree stood there instead.
   */
  const made = await addTheWorktree();
  if (made.kind === 'a-worktree-stood-there') {
    return 'worktree: already there and holding nothing uncommitted, so it is used as it is (the attempt before this one ended without making a worktree, and a worktree stood at that path, which another start of this name may have made)';
  }
  /**
   What git answered about making the worktree.
   */
  const added = made.answered;
  if (added.status !== 0) {
    /**
     What became of the branch this start made, which a killed attempt left behind.
     */
    const branchLeft = branchWasThere
      ? ''
      : await removeTheBranchThisStartMade({
        git,
        checkoutFolder,
        branch,
      },);
    throw new GitCommandFailedError(
      `git worktree add failed for ${worktree} after ${String(WORKTREE_ADD_ATTEMPTS,)} attempts: ${added.problem
        ?? added.stderr
          .trim()}${branchLeft}`,
    );
  }
  return branchWasThere
    ? `worktree add: status ${String(added.status,)} (existing branch ${branch}, which keeps its own commits, so the base ${base} was not used)`
    : `worktree add: status ${String(added.status,)} (new branch ${branch} at ${base})`;
}

/**
 Removes the branch this start made, when a killed `git worktree add` left it behind.

 Whoever makes a thing cleans it up: a branch this command created moments ago holds no commit beyond its start point and no
 worktree has it checked out, and `git branch --delete` refuses it the moment either stops being true. The forced form is never
 used, so a branch that holds anything unmerged stays, and the line naming the failure says so.

 @param git - how one git command is run

 @param checkoutFolder - the checkout whose branches are read

 @param branch - the branch this start would have made

 @returns what became of the branch, as the end of the line a reader reads

 @example
 ```ts
 const left = await removeTheBranchThisStartMade({ git, checkoutFolder: '/repo', branch: 'test/review-a', },);
 ```
 */
async function removeTheBranchThisStartMade({
  git,
  checkoutFolder,
  branch,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly branch: string;
},): Promise<string> {
  if (!(await branchExists({
    git,
    checkoutFolder,
    branch,
  },))) {
    return `; no branch ${branch} was made before the add ended`;
  }
  /**
   What git answered when asked to delete it.
   */
  const removed = await git({
    folder: checkoutFolder,
    words: [
      'branch',
      '--delete',
      branch,
    ],
  },);
  if (removed.status === 0) {
    return `; the branch ${branch} this start made was removed`;
  }
  return `; the branch ${branch} this start made is still there, because git refused to delete it (git branch --delete answered ${String(removed.status,)}: ${removed.stderr
    .trim()})`;
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
