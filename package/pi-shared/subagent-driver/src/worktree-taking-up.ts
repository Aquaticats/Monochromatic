/**
 What is done with whatever stands at the path a start wants, decided from the proof beside it and from the run that wrote that proof.

 A worktree is taken up when the proof beside it says its making finished, on the branch this start asked for, and the run that made it
 has ended: a wake, a second try or a retry of one name finds the work it left, and a start that arrives while that run still works is
 refused. A proof that says a making did not finish is a licence to remove what stands there only when its maker is shown gone, which
 the proof's own process record answers; a proof with no maker, from an earlier version of this package, is never such a licence.
 A path that holds a worktree or files with no proof at all is a person's until a bound passes: the first start that wants it records
 the sighting and refuses, and a start a day later removes it and makes its own. Nothing here takes a worktree on a `.git` and a clean
 status.

 @module
 */

import {
  access,
  rmdir,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type GitRunner,
  WORKTREE_ADD_TIMEOUT_MILLISECONDS,
  WorktreeTakenError,
} from './worktree-git.ts';
import {
  itsMakerMayStillBeMaking,
  moveTheProofAway,
  proofOf,
  worktreeProofFileOf,
} from './worktree-proof.ts';
import {
  UNPROVEN_WORKTREE_KEPT_MILLISECONDS,
  moveTheSightingAway,
  readSighting,
  unprovenSightingFileOf,
  writeSighting,
} from './unproven-worktree-sighting.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

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
 What a start does with the path it wants.

 A `make` is the word to go on and make the worktree; a `note` is the line a start that took up the worktree already there prints.
 Every other answer is a refusal, thrown with the words a reader needs.
 */
export type StandingDecision =
  | { readonly kind: 'make' }
    | {
      readonly kind: 'note';
      readonly note: string
    };

/**
 Clears what a making left at the path: the registration git may have written, and the partial checkout a killed add leaves.

 Only proven leftovers reach this: a making this start owns, a making whose maker is shown gone, or a worktree a person left unproven
 for longer than the bound.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path

 @example
 ```ts
 await clearWhatALeftoverLeft({ git, checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a' },);
 ```
 */
export async function clearWhatALeftoverLeft({
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
 Says whether the run folder a proof names has ended, which is the moment its worktree may be taken up by another start.

 A run's driver writes the run's summary as it ends, so a folder holding one is a run that ended; a folder with no summary is a run
 that may still be working, and a worktree it made is not another start's to take.

 @param runFolderName - name of the run folder recorded in the proof

 @param runsFolder - parent directory containing the named run folder

 @returns whether that run ended

 @example
 ```ts
 const ended = await theRunAProofNamesHasEnded({ runFolderName: 'copy-route-1', runsFolder: '/runs', },);
 ```
 */
async function theRunAProofNamesHasEnded({
  runFolderName,
  runsFolder,
}: {
  readonly runFolderName: string;
  readonly runsFolder: string;
},): Promise<boolean> {
  return await isThere(join(
    runsFolder,
    runFolderName,
    'summary.json',
  ),);
}

/**
 Decides what to do with whatever stands at a start's worktree path.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path

 @param branch - the branch the run's work goes on

 @param claim - the run folder whose claim this start is

 @param runsFolder - the folder the runs live in, where the summary of the run a proof names is read

 @param now - where this call reads the time

 @param processFolder - the folder holding one entry per process

 @returns the word to go on making, or the note for a worktree taken up

 @throws WorktreeTakenError when what stands there is not this start's to use or to remove

 @example
 ```ts
 const decision = await settleWhatStandsAtThePath({ git, checkoutFolder: '/repo', worktree, branch, claim, runsFolder, now: Date.now, processFolder: '/proc', },);
 ```
 */
export async function settleWhatStandsAtThePath({
  git,
  checkoutFolder,
  worktree,
  branch,
  claim,
  runsFolder,
  now,
  processFolder,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
  readonly branch: string;
  readonly claim: {
    readonly runName: string;
    readonly runFolder: string;
  };
  readonly runsFolder: string;
  readonly now: () => number;
  readonly processFolder: string;
},): Promise<StandingDecision> {
  /**
   The name of the run folder this start claimed, which its own proof carries.
   */
  const ownFolderName = claim.runFolder
    .split('/',)
    .at(-1,)
    ?? claim.runFolder;
  /**
   The proof beside the path, or the problem reading one gave.
   */
  const held = await proofOf({ worktree, },);
  if (held.proof === undefined) {
    return await theUnprovenPathDecision({
      git,
      checkoutFolder,
      worktree,
      ownFolderName,
      now,
      held,
    },);
  }
  /**
   The proof that stands beside the path.
   */
  const {proof} = held;
  if ((proof.runFolder === ownFolderName) && (proof.path === worktree)) {
    // This start's own proof: its making may have been interrupted in this same call, and what it left is its own.
    return { kind: 'make', };
  }
  if (proof.state === 'made') {
    if (proof.branch !== branch) {
      throw new WorktreeTakenError(
        `${worktree} holds the worktree run folder ${proof.runFolder} made on branch ${proof.branch}, and this start asks for branch ${branch}, so it was left exactly as it is and this start refused: working in it would put this start's commits on ${proof.branch}. A person removes ${worktree} and ${worktreeProofFileOf({ worktree, },)} when that worktree's run is done with it, or this start names another run so it gets a worktree of its own.`,
      );
    }
    if (!(await theRunAProofNamesHasEnded({
      runFolderName: proof.runFolder,
      runsFolder,
    },))) {
      throw new WorktreeTakenError(
        `${worktree} holds the worktree run folder ${proof.runFolder} made on branch ${proof.branch}, and that run has written no summary, so it may still be working in it: the worktree was left exactly as it is and this start refused. A start of this name may take it up once that run has ended; a person removes ${worktree} and ${worktreeProofFileOf({ worktree, },)} when nothing there is wanted.`,
      );
    }
    return {
      kind: 'note',
      note: `worktree: already there, made by run folder ${proof.runFolder} on branch ${proof.branch}, whose run has ended, and taken up`,
    };
  }
  // The proof says a making did not finish. It is a licence to remove what stands there only when its maker is shown gone.
  if (await itsMakerMayStillBeMaking({
    proof,
    processFolder,
  },)) {
    throw new WorktreeTakenError(
      `${worktree} holds a worktree whose making has not finished and whose maker may still be making it (run folder ${proof.runFolder} wrote its proof at ${proof.writtenAt}${(proof.maker === undefined) ? ', and the proof names no maker' : `, from process ${String(proof.maker
        .pid,)}`}), so it was left exactly as it is and this start refused. A later start removes it once its maker is gone; a person removes ${worktree} and ${worktreeProofFileOf({ worktree, },)} when nothing there is wanted.`,
    );
  }
  if (!(await moveTheProofAway({ worktree, },))) {
    // Another start moved the proof first, so this one leaves the path to it.
    throw new WorktreeTakenError(
      `${worktree} holds a worktree whose proof another start is taking over, so it was left exactly as it is and this start refused. Start this run again when that start has finished.`,
    );
  }
  await clearWhatALeftoverLeft({
    git,
    checkoutFolder,
    worktree,
  },);
  return { kind: 'make', };
}

/**
 Decides what to do with a path that holds no proof this package wrote.

 An empty folder is no worktree and holds nothing, so it is removed and the making goes on. Anything else, with `.git` or without it,
 is a worktree or a folder a person made: the first start that wants the path records the sighting and refuses, and a start a bound
 later removes it and makes its own.

 @param git - how one git command is run

 @param checkoutFolder - the checkout the worktree is made in

 @param worktree - the worktree's path

 @param ownFolderName - name of the run folder this start claimed

 @param now - where this call reads the time

 @param held - what reading a proof gave, which carries the problem when a file there is no proof

 @returns the word to go on making

 @throws WorktreeTakenError when the path holds something a person made, or one whose sighting has not reached the bound

 @example
 ```ts
 const decision = await theUnprovenPathDecision({ git, checkoutFolder, worktree, ownFolderName, now, held, },);
 ```
 */
async function theUnprovenPathDecision({
  git,
  checkoutFolder,
  worktree,
  ownFolderName,
  now,
  held,
}: {
  readonly git: GitRunner;
  readonly checkoutFolder: string;
  readonly worktree: string;
  readonly ownFolderName: string;
  readonly now: () => number;
  readonly held: {
    readonly problem?: string
  };
}): Promise<StandingDecision> {
  if (!(await isThere(worktree,))) {
    return { kind: 'make', };
  }
  try {
    // An empty folder is no worktree, and rmdir removes only an empty one, so nothing anyone wrote can be lost here.
    await rmdir(worktree,);
    return { kind: 'make', };
  }
  catch (error) {
    if ((errorCodeOf(error,) !== 'ENOTEMPTY') && (errorCodeOf(error,) !== 'EEXIST')) {
      throw error;
    }
  }
  /**
   The record of the first start that wanted this path, or the problem reading one gave.
   */
  const {
    sighting,
    problem,
  } = await readSighting({ worktree, },);
  if (sighting === undefined) {
    /**
     Whether this start wrote the first sighting, which decides what it may say about when the bound passes.
     */
    const first = await writeSighting({
      worktree,
      sighting: {
        path: worktree,
        firstSeenAt: new Date(now(),).toISOString(),
        seenBy: ownFolderName,
      },
    },);
    throw new WorktreeTakenError(
      `${worktree} holds something no proof says this run made${(held.problem ?? problem) === undefined ? '' : ` (${String(held.problem ?? problem,)})`}, so it was left exactly as it is and this start refused: its files are a person's, or a worktree made before this package wrote proofs, until a proof says otherwise.${first ? ` This start recorded the first sighting and a start of this name a day later removes it and makes its own worktree.` : ' A start of this name a day after the first sighting removes it and makes its own worktree.'} A person removes ${worktree} and ${worktreeProofFileOf({ worktree, },)} when nothing there is wanted, which is the way to start this run now.`,
    );
  }
  /**
   How long ago the first sighting was written.
   */
  const age = now() - Date.parse(sighting.firstSeenAt,);
  if (!(age >= UNPROVEN_WORKTREE_KEPT_MILLISECONDS)) {
    throw new WorktreeTakenError(
      `${worktree} holds something no proof says this run made, and a start of this name first saw it at ${sighting.firstSeenAt} (run folder ${sighting.seenBy}), so it was left exactly as it is and this start refused. A start of this name removes it once a day has passed since that sighting; a person removes ${worktree} and ${unprovenSightingFileOf({ worktree, },)} when nothing there is wanted, which is the way to start this run now.`,
    );
  }
  if (!(await moveTheSightingAway({ worktree, },))) {
    throw new WorktreeTakenError(
      `${worktree} holds something no proof says this run made, and another start is taking it over, so it was left exactly as it is and this start refused. Start this run again when that start has finished.`,
    );
  }
  await clearWhatALeftoverLeft({
    git,
    checkoutFolder,
    worktree,
  },);
  return { kind: 'make', };
}
