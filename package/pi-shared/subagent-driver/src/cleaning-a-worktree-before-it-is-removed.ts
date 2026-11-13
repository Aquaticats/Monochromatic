/**
 Running the caller's clean command inside one worktree before the sweep removes it, within a bound, so a command that waits on a
 lock, a pipe or the network cannot hold the whole sweep.

 The sweep's own reads and its worktree removal are bounded, and this was the one command in the loop that was not: `cargo clean`
 waiting on Cargo's package-cache lock, which on this machine is an everyday wait of ten minutes and more, held the sweep for as long
 as it waited and no later worktree was reached.

 @module
 */

import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import {
  type ShownCommandEnding,
  runCommandShowingItsOutputWithin,
} from './running-a-command-with-the-callers-streams.ts';

/**
 Seconds the caller's clean command may take inside one worktree before the sweep ends it and goes on to the next worktree.

 A clean is given up quickly because the sweep does not need it to succeed: the worktree removal that follows frees the same tree,
 one way or the other, so what a given-up clean costs is the difference between deleting a build tree before the removal and during
 it, not the space itself. What it saves is the wait, and the waits are long: Cargo's package-cache lock, which a concurrent build
 holds, was measured at ten minutes and more on this machine on 2026-11-13.

 The length is what a clean that is doing its own work needs, from the one deletion rate measured here: 2.0 GiB and 22,000 entries in
 1.38 s through this package's emptying command, which is 1.45 GiB a second, so four seconds is about 5.8 GiB of deleting. A build tree
 here is larger than that (the pinned nightly's build folders held 372 GB for about 40 agent worktrees, about 9 GB each), and a clean
 that is still going at four seconds is one that is either slower than this disk deletes at, or waiting on something that is not the
 disk at all. Both cases are what this bound is for: the sweep needs the wait gone more than it needs the space freed early.
 */
export const CLEAN_COMMAND_BOUND_SECONDS = 4;

/**
 Runs the caller's clean command inside one worktree, within its bound, and says in one line what became of it.

 @param cleanWords - the words of the clean command, which is none when the caller named none

 @param cleanFolder - the folder inside the worktree the clean runs in, which is empty when the caller named none

 @param worktreeFolder - the worktree the clean runs in

 @param boundMilliseconds - how long the clean may take, which is the bound beside this module's own constant unless a test shortens it

 @returns the words the sweep's line carries for this worktree

 @example
 ```ts
 const freed = await cleanTheWorktreeIn({ cleanWords: ['cargo', 'clean',], cleanFolder: '.', worktreeFolder: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function cleanTheWorktreeIn({
  cleanWords,
  cleanFolder,
  worktreeFolder,
  boundMilliseconds = CLEAN_COMMAND_BOUND_SECONDS * MILLISECONDS_PER_SECOND,
}: {
  readonly cleanWords: readonly string[];
  readonly cleanFolder: string;
  readonly worktreeFolder: string;
  readonly boundMilliseconds?: number;
},): Promise<string> {
  if ((cleanFolder === '') || (cleanWords.length === 0)) {
    return 'no clean command';
  }
  /**
   What the clean command answered within its bound.
   */
  const cleaned: ShownCommandEnding = await runCommandShowingItsOutputWithin({
    command: cleanWords[0] ?? '',
    words: cleanWords.slice(1,),
    folder: `${worktreeFolder}/${cleanFolder}`,
    timeoutMilliseconds: boundMilliseconds,
  },);
  if (cleaned.givenUp === true) {
    return `clean given up after ${String(Math.round(boundMilliseconds / MILLISECONDS_PER_SECOND,),)} s, and the sweep went on`;
  }
  if (cleaned.startProblem !== undefined) {
    return `clean could not be started: ${cleaned.startProblem}`;
  }
  return `clean ended with status ${String(cleaned.status,)}`;
}
