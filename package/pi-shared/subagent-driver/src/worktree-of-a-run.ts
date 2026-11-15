/**
 The worktree a run works in: the entry a launcher calls, which takes up the worktree that is already at the path when its proof says
 it is this run's, and otherwise makes one.

 **The names a start takes, and what each one derives from.** A start claims a run folder by making it, and the number that claim
 gives it (`X-4`) is the root of the other names: the unit is `pi-X@4.service`, which systemd refuses to make twice, and the proof
 this entry writes names the run folder that made the worktree. The worktree's path is `<checkout>/.claude/worktrees/pi-X`, named
 for the run rather than for one attempt of it, because every start of a name that continues its work (a wake, a second try, a retry
 after a failure) must find the work it left; the claim's own proof, not the path, is what says whose work that is. The branch is the
 caller's name for the work (`--branch`), recorded in the proof and checked whenever a worktree is taken up, because a start asked
 for one branch must never be handed a worktree on another.

 The owner, 2026-11-10: "You're the coordinator and you're supposed to remove finished worktrees automatically." A worktree
 added on btrfs is a snapshot of `main`, so it keeps every file `main` held when it was taken, build folders included: finished
 worktrees left in place filled the disk. The wrapper's `git worktree remove` refuses a worktree holding uncommitted work unless
 it is forced, and nothing here forces.

 @module
 */

import { rmdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  type GitRunner,
  WorktreeFolderOccupiedError,
  isThere,
} from './worktree-git.ts';
import { makeTheWorktree, } from './worktree-making.ts';
import { proofOf, } from './worktree-proof.ts';
import { takeUpTheWorktreeThere, } from './worktree-taking-up.ts';

/**
 The commit or branch a run's new branch starts at when the caller names none, which is what every launcher started from before a
 run could name its own.
 */
export const BASE_OF_A_RUN_NAMING_NONE = 'main';

/**
 Makes a run's worktree, on the branch the caller names, and answers with what it did.

 The path is the run's, and the proof beside it is one making's: this function writes the proof of the claim it is given, before
 asking git to add the worktree and again once git has finished, so the path says which run folder made the worktree, on which
 branch, and whether the making finished.

 A branch this call creates starts at `base`, which is a commit or a branch: a fix for a reviewed branch starts at that branch, and
 a review starts at the commit it reviews. A branch that already exists keeps its own commits, so `base` does not move it: the
 scratch launcher left an existing branch alone the same way, and moving it would rewrite work someone else may be reading. The
 note this answers with says which of the two happened, so a reader is never left to guess which commit a run's branch began at.

 A worktree already at the path is used only when its proof says its making finished on the branch asked for: that is how a wake,
 a second try or a retry of one run finds the work it left. A worktree with no proof, with a proof about another path, with a proof
 for another branch, or with a proof whose making has not finished and may still be under way is left exactly as it is and this
 start refuses. A proof whose making did not finish but whose making is older than that making's own bound names an add killed at
 its bound: its leftover is a partial checkout no run worked in, so it is cleared and the worktree made again.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path, named for the run

 @param branch - the branch the run's work goes on

 @param base - the commit or branch a branch this call creates starts at, which an existing branch ignores

 @param claim - the run folder whose claim this making is, and the run's name, which its proof names

 @param now - where this call reads the time, which decides whether a making that did not finish may still be under way

 @returns what was done, in words for a reader

 @throws WorktreeFolderOccupiedError when the path holds a folder that is no worktree and nothing was deleted to make room

 @throws WorktreeTakenError when a worktree there is not this start's to use, which is left exactly as it is

 @throws GitCommandFailedError when git refused to make the worktree

 @example
 ```ts
 const note = await makeWorktree({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', branch: 'feat/a', base: 'main', claim: { runName: 'a', runFolder: '/runs/a-1' }, now: Date.now, },);
 ```
 */
export async function makeWorktree({
  git,
  checkoutFolder,
  worktree,
  branch,
  base,
  claim,
  now = Date.now,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
  readonly branch: string;
  readonly base: string;
  readonly claim: {
    /**
     The run's name.
     */
    readonly runName: string;

    /**
     The run folder this start claimed, whose name carries the claim's number.
     */
    readonly runFolder: string;
  };
  readonly now?: () => number;
},): Promise<string> {
  if (await isThere(join(
    worktree,
    '.git',
  ),)) {
    /**
     What was to be done with the worktree that stands at the path, which is nothing when this start took it up.
     */
    const {
      cleared,
      note,
    } = await takeUpTheWorktreeThere({
      git,
      checkoutFolder,
      worktree,
      branch,
      claim,
      held: await proofOf({ worktree, },),
      now,
    },);
    if (!cleared) {
      return note ?? '';
    }
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
  return await makeTheWorktree({
    git,
    checkoutFolder,
    worktree,
    branch,
    base,
    claim,
    now,
  },);
}
