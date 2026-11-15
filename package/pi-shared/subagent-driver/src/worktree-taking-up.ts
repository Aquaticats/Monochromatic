/**
 What is done with a worktree that stands at the path a start wants: taken up when its proof shows it is this run's on the branch
 asked for, cleared when its proof says its making was killed at its own bound, and otherwise left exactly as it is while the start
 refuses with words that name the path.

 Nothing here takes a worktree on a `.git` and a clean status: a path holding both says nothing about who made it, so what decides is
 the proof beside it, which names the run folder that made it, the branch, and whether that making finished.

 @module
 */

import { rm, } from 'node:fs/promises';
import {
  A_MAKING_GIVEN_UP_MILLISECONDS,
  type GitRunner,
  WORKTREE_ADD_TIMEOUT_MILLISECONDS,
  WorktreeTakenError,
} from './worktree-git.ts';
import {
  type WorktreeProof,
  worktreeProofFileOf,
} from './worktree-proof.ts';

/**
 Clears what one making of this start left at the path: the registration git may have written, and the partial checkout a killed add
 leaves.

 Only a making's own leftover reaches this: the proof that says the making did not finish is the one this start wrote moments ago, or
 one older than a making's own bound, whose add was killed at it.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path

 @example
 ```ts
 await clearWhatThisMakingLeft({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a' },);
 ```
 */
export async function clearWhatThisMakingLeft({
  git,
  checkoutFolder,
  worktree,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
},): Promise<void> {
  await git({
    folder: checkoutFolder,
    words: [
      'worktree',
      'prune',
      '--expire=now',
    ],
    timeoutMilliseconds: WORKTREE_ADD_TIMEOUT_MILLISECONDS,
  },);
  await rm(
    worktree,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Takes up the worktree that stands at the path when its proof shows it is this run's to use, clears a making whose add was killed at
 its bound, and refuses with words that name the path for everything else.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path

 @param branch - the branch the run's work goes on

 @param claim - the run folder whose claim this making is

 @param held - the proof beside the path, or the problem reading it gave

 @param now - where this call reads the time

 @returns what was done: the note for a worktree taken up, or the word that a killed making's leftover was cleared and the making
 must go on

 @throws WorktreeTakenError when the worktree is not this start's to use

 @example
 ```ts
 const { cleared } = await takeUpTheWorktreeThere({ git, checkoutFolder, worktree, branch, claim, held, now: Date.now });
 ```
 */
export async function takeUpTheWorktreeThere({
  git,
  checkoutFolder,
  worktree,
  branch,
  claim,
  held,
  now,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
  readonly branch: string;
  readonly claim: {
    readonly runName: string;
    readonly runFolder: string;
  };
  readonly held: {
    readonly proof?: WorktreeProof;
    readonly problem?: string
  };
  readonly now: () => number;
}): Promise<{
  readonly cleared: boolean;
  readonly note?: string
}> {
  /**
   What is left of a making whose add was killed at its bound, which is a making that cannot still be under way.
   */
  const makingGivenUp = (held.proof !== undefined)
    && (held.proof
      .state
      === 'making')
    && ((now() - Date.parse(held.proof
      .writtenAt,)) > A_MAKING_GIVEN_UP_MILLISECONDS);
  if (makingGivenUp) {
    // The making's own bound has passed, so its add was killed: what stands at the path is a partial checkout no run worked in.
    await clearWhatThisMakingLeft({
      git,
      checkoutFolder,
      worktree,
    },);
    return { cleared: true, };
  }
  if (held.proof === undefined) {
    throw new WorktreeTakenError(
      `${worktree} holds a worktree that no proof says this run made${(held.problem === undefined) ? '' : ` (${held.problem})`}, so it was left exactly as it is and this start refused: its files are another start's or a person's until a proof says otherwise. A later start of ${claim.runName} meets it again; a person removes ${worktree} and ${worktreeProofFileOf({ worktree, },)} when nothing there is wanted.`,
    );
  }
  if (held.proof
    .state
    !== 'made') {
    throw new WorktreeTakenError(
      `${worktree} holds a worktree whose making has not finished (run folder ${held.proof
        .runFolder} wrote its proof at ${held.proof
          .writtenAt}), so it was left exactly as it is and this start refused. A later start of ${claim.runName} clears what that killed making left once the making's own bound has passed, and makes the worktree again; until then nothing here uses it.`,
    );
  }
  if (held.proof
    .branch
    !== branch) {
    throw new WorktreeTakenError(
      `${worktree} holds the worktree run folder ${held.proof
        .runFolder} made on branch ${held.proof
          .branch}, and this start asks for branch ${branch}, so it was left exactly as it is and this start refused: working in it would put this start's commits on ${held.proof
            .branch}. A person removes ${worktree} when that worktree's run is done with it, or this start names another run so it gets a worktree of its own.`,
    );
  }
  return {
    cleared: false,
    note: `worktree: already there, made by run folder ${held.proof
      .runFolder} on branch ${held.proof
        .branch}, and taken up because that is the branch this start asked for`,
  };
}
