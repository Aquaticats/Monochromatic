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

import { dirname, } from 'node:path';
import {
  type GitRunner,
  WorktreeTakenError,
} from './worktree-git.ts';
import { makeTheWorktree, } from './worktree-making.ts';
import {
  type ProofMaker,
  type WorktreeProof,
  claimProof,
  makerOfThisProcess,
} from './worktree-proof.ts';
import { settleWhatStandsAtThePath, } from './worktree-taking-up.ts';

/**
 The commit or branch a run's new branch starts at when the caller names none, which is what every launcher started from before a
 run could name its own.
 */
export const BASE_OF_A_RUN_NAMING_NONE = 'main';

/**
 Makes a run's worktree, on the branch the caller names, and answers with what it did.

 **The names a start takes, and what each one derives from.** A start claims a run folder by making it, and the number that claim
 gives it (`X-4`) is the root of the other names: the unit is `pi-X@4.service`, which systemd refuses to make twice, and the proof
 this entry writes names the run folder that made the worktree. The worktree's path is `<checkout>/.claude/worktrees/pi-X`, named
 for the run rather than for one attempt of it, because every start of a name that continues its work must find the work it left; the
 proof, not the path, says whose work that is. The branch is the caller's name for the work (`--branch`), recorded in the proof and
 checked whenever a worktree is taken up.

 Whoever stands at the path is decided by `worktree-taking-up.ts` from the proof beside it: a worktree this start may take up is, a
 leftover whose maker is shown gone is cleared, and everything else is left exactly as it is while this start refuses. Once the path
 is this start's, the proof is claimed with one exclusive create, so two starts can never both believe they are making it.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path, named for the run

 @param branch - the branch the run's work goes on

 @param base - the commit or branch a branch this call creates starts at, which an existing branch ignores

 @param claim - the run folder whose claim this making is, and the run's name, which its proof names

 @param runsFolder - the folder the runs live in, where the summary of the run a proof names is read

 @param now - where this call reads the time, which decides whether a sighting has reached the bound

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param maker - the process this start is, which a proof this call writes names; read from `processFolder` when it is not given

 @returns what was done, in words for a reader

 @throws WorktreeTakenError when what stands at the path is not this start's to use or to remove

 @throws GitCommandFailedError when git refused to make the worktree

 @example
 ```ts
 const note = await makeWorktree({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', branch: 'feat/a', base: 'main', claim: { runName: 'a', runFolder: '/runs/a-1' }, runsFolder: '/runs', now: Date.now, processFolder: '/proc', maker, },);
 ```
 */
export async function makeWorktree({
  git,
  checkoutFolder,
  worktree,
  branch,
  base,
  claim,
  runsFolder,
  now = Date.now,
  processFolder = '/proc',
  maker,
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
  readonly runsFolder?: string;
  readonly now?: () => number;
  readonly processFolder?: string;
  readonly maker?: ProofMaker;
},): Promise<string> {
  /**
   This process, as a proof names it.
   */
  const theMaker = maker ?? await makerOfThisProcess({ processFolder, },);
  /**
   The folder the runs live in, which is the folder the claimed run folder was made in unless the caller says otherwise.
   */
  const runsFolderUsed = runsFolder ?? dirname(claim.runFolder,);
  /**
   What stands at the path, and what this start may do with it.
   */
  const decision = await settleWhatStandsAtThePath({
    git,
    checkoutFolder,
    worktree,
    branch,
    claim,
    runsFolder: runsFolderUsed,
    now,
    processFolder,
  },);
  if (decision.kind === 'note') {
    return decision.note;
  }
  /**
   The proof this start writes, which names its run folder, its branch and itself.
   */
  const proof: WorktreeProof = {
    path: worktree,
    runName: claim.runName,
    runFolder: claim.runFolder
      .split('/',)
      .at(-1,)
      ?? claim.runFolder,
    branch,
    state: 'making',
    writtenAt: new Date(now(),).toISOString(),
    maker: theMaker,
  };
  if (!(await claimProof({
    worktree,
    proof,
  },))) {
    // Two starts wanted the path in the same instant and this one lost: it reads what the winner left, and it never makes anything
    // without the proof.
    /**
     What the winner's proof says this start may do.
     */
    const afterTheOther = await settleWhatStandsAtThePath({
      git,
      checkoutFolder,
      worktree,
      branch,
      claim,
      runsFolder: runsFolderUsed,
      now,
      processFolder,
    },);
    if (afterTheOther.kind === 'note') {
      return afterTheOther.note;
    }
    throw new WorktreeTakenError(
      `${worktree} was claimed by another start of ${claim.runName} in the instant this start wanted it, so nothing was made there. Start this run again, or start it under another name.`,
    );
  }
  return await makeTheWorktree({
    git,
    checkoutFolder,
    worktree,
    branch,
    base,
    claim,
    now,
    maker: theMaker,
  },);
}
