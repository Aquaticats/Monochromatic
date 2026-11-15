/**
 The git questions one making of a worktree asks, and the failures they answer with: how a command is run, how long an add may take,
 how many times one start tries, when a making that did not finish has to be taken for one that ended, and the errors a start stops
 with.

 Everything that makes, takes up or removes a worktree asks git through this module, so the bound one question gets and the error a
 failed one answers with are the same wherever they are read.

 @module
 */

import {
  access,
} from 'node:fs/promises';
import type { CommandOutput, } from './run-command.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';

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
 Seconds added to a making's own bound before its proof is taken for a making that ended: an add killed at its bound writes its
 proof a moment before that bound, so the bound alone would leave a making that was killed at its last instant looking under way.
 */
const MAKING_GIVEN_UP_MARGIN_SECONDS = 60;

/**
 The time after which a proof that says its making did not finish cannot belong to a making still under way: a making is bounded by
 {@link WORKTREE_ADD_TIMEOUT_MILLISECONDS}, so a proof older than that names a making whose add was killed at its bound.
 */
export const A_MAKING_GIVEN_UP_MILLISECONDS: number = WORKTREE_ADD_TIMEOUT_MILLISECONDS
  + (MILLISECONDS_PER_SECOND * MAKING_GIVEN_UP_MARGIN_SECONDS);

/**
 How many times one start tries to make its worktree. A first attempt killed by its own bound leaves its own proof saying the making
 did not finish, and the proof is this start's, so the attempt after it clears only what this start's own making left. A path whose
 proof says another claim's making did not finish is never cleared here.
 */
export const WORKTREE_ADD_ATTEMPTS = 2;

//endregion Constants

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
 The path a run's worktree belongs at holds a worktree this start may not use, so it is left exactly as it is.

 That is a worktree whose proof is missing, unreadable or about another path; one whose proof says another branch; or one whose
 making has not finished and may still be under way. The files there are another start's, or a making's, until a proof says
 otherwise, so nothing here removes them and the start refuses with words that name the path.
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
export async function isThere(path: string,): Promise<boolean> {
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
