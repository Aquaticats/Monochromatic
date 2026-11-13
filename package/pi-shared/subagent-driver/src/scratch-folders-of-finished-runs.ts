/**
 What the command that removes finished worktrees does with one run's scratch folder: empties it when a run of that name has ended
 and nothing of it still works there, and says why it left it alone otherwise.

 The launcher's own end step empties the folder when it runs, and a launcher killed before that step leaves the folder for this
 command: a folder nothing owns is what left two days of scratch on this machine on 2026-11-13, 68.2 GB of it its own data.

 @module
 */

import { decideScratchRemoval, } from './finished-worktrees.ts';
import type { RunCommand, } from './run-command.ts';
import { emptyTheScratchFolderAtTheLowestDiskPriority, } from './scratch-folder-emptied-at-the-lowest-disk-priority.ts';
import { scratchFolderOf, } from './scratch-folder-of-a-run.ts';

/**
 Says what became of one finished run's scratch folder, in the lines the command prints.

 @param name - the run's name

 @param endReason - why the run ended, as the run's summary gives it, or nothing when no summary says

 @param briefQueued - whether a brief was written after its latest run ended

 @param unitState - the state systemd answered for the run's unit

 @param inUseFolder - the folder of the run's a process still works in, or empty text when none does

 @param apply - whether the command was asked to do it rather than to list it

 @param scratchRoot - the root every run's scratch folder lives under

 @param run - how one command is run

 @returns the lines, which are this command's own for a folder it left alone and the emptying's own for one it emptied

 @example
 ```ts
 const lines = await scratchFolderLinesOf({ name: 'a', endReason: 'idle', briefQueued: false, unitState: 'inactive', inUseFolder: '', apply: true, scratchRoot: '/home/a/temp/agent', run: runCommandWithChildProcess, },);
 ```
 */
export async function scratchFolderLinesOf({
  name,
  endReason,
  briefQueued,
  unitState,
  inUseFolder,
  apply,
  scratchRoot,
  run,
}: {
  readonly name: string;
  readonly endReason?: unknown;
  readonly briefQueued: boolean;
  readonly unitState: string;
  readonly inUseFolder: string;
  readonly apply: boolean;
  readonly scratchRoot: string;
  readonly run: RunCommand;
},): Promise<readonly string[]> {
  /**
   What is to be done with this run's scratch folder.
   */
  const decision = decideScratchRemoval({
    ...((typeof endReason) === 'string') ? { endReason, } : {},
    briefQueued,
    unitState,
    inUseFolder,
  },);
  if (decision.action === 'skip') {
    return [`${name}: scratch folder left alone, ${decision.reason}`,];
  }
  /**
   The folder the launcher gave that run.
   */
  const folder = scratchFolderOf({
    scratchRoot,
    runName: name,
  },);
  if (!apply) {
    return [`${name}: would empty its scratch folder at ${folder} except keep/`,];
  }
  return await emptyTheScratchFolderAtTheLowestDiskPriority({
    run,
    scratchRoot,
    runName: name,
  },);
}
