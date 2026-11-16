/**
 Making a run's worktree: the two attempts one start is given, the proof each attempt writes beside the path, and the branch this
 start made when both attempts fail.

 A first attempt killed by its own bound leaves the proof this start wrote saying the making did not finish, and that proof is this
 start's own, so only what this start's own making left is cleared before the second attempt.

 @module
 */

import {
  basename,
} from 'node:path';
import type { CommandOutput, } from './run-command.ts';
import {
  type GitRunner,
  GitCommandFailedError,
  WORKTREE_ADD_ATTEMPTS,
  WORKTREE_ADD_TIMEOUT_MILLISECONDS,
  branchExists,
} from './worktree-git.ts';
import {
  type ProofMaker,
  writeProof,
} from './worktree-proof.ts';
import { clearWhatALeftoverLeft, } from './worktree-taking-up.ts';

/**
 Writes the proof of this making, asks git for the worktree, and answers with what it did.

 A first attempt killed by its own bound leaves the proof this start wrote saying the making did not finish, and that proof is this
 start's own, so only what this start's own making left is cleared before the second attempt. Nothing a proof of another claim names
 is cleared here.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path

 @param branch - the branch the run's work goes on

 @param base - the commit or branch a branch this call creates starts at

 @param claim - the run folder whose claim this making is

 @param now - where this call reads the time

 @returns what was done, in words for a reader

 @throws GitCommandFailedError when every attempt failed

 @example
 ```ts
 const note = await makeTheWorktree({ git, checkoutFolder, worktree, branch, base, claim, now: Date.now });
 ```
 */
export async function makeTheWorktree({
  git,
  checkoutFolder,
  worktree,
  branch,
  base,
  claim,
  now,
  maker,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
  readonly branch: string;
  readonly base: string;
  readonly claim: {
    readonly runName: string;
    readonly runFolder: string;
  };
  readonly now: () => number;
  readonly maker: ProofMaker;
}): Promise<string> {
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

   @returns what git answered to the attempt that ended the loop

   @example
   ```ts
   const added = await addTheWorktree();
   ```
   */
  async function addTheWorktree(): Promise<CommandOutput> {
    /* oxlint-disable eslint/no-await-in-loop -- one attempt, then what this start's own attempt left cleared, then the attempt
       after it: the second attempt depends on the first having ended. See doc/lint/sequential-awaits.md. */
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
      await writeProof({
        worktree,
        proof: {
          path: worktree,
          runName: claim.runName,
          runFolder: basename(claim.runFolder,),
          branch,
          state: 'making',
          writtenAt: new Date(now(),).toISOString(),
          maker,
        },
      },);
      // A killed add may have registered the path before it ended. Pruning clears such a registration, and it takes no folder and
      // no file with it: what is left in the folder is still there for the attempt that follows to answer about.
      await git({
        folder: checkoutFolder,
        words: [
          'worktree',
          'prune',
          '--expire=now',
        ],
        timeoutMilliseconds: WORKTREE_ADD_TIMEOUT_MILLISECONDS,
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
      await clearWhatALeftoverLeft({
        git,
        checkoutFolder,
        worktree,
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
  await writeProof({
    worktree,
    proof: {
      path: worktree,
      runName: claim.runName,
      runFolder: basename(claim.runFolder,),
      branch,
      state: 'made',
      writtenAt: new Date(now(),).toISOString(),
      maker,
    },
  },);
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
