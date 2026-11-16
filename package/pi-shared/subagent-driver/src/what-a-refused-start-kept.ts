/**
 What a refused start left behind, so that a person and a later start can read why a worktree and a run folder are there.

 Why: a start makes the worktree and the run folder before it asks the manager last time and before it reads the folders a unit may
 write, and either of those can refuse. Without a record the leftover worktree and the empty run folder are a mystery, and the next
 start of the same name silently reuses the worktree. The record is one small file in the run's own folder, beside the summary a run
 writes, named for what it says.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 What the run's own folder calls a refused start's record.
 */
export const REFUSED_START_RECORD_NAME: string = 'refused-start.json';

/**
 Writes what a refused start left, in the run's own folder: the words of the refusal, the worktree and the branch, so a person and a
 later start find why the leftovers are there.

 @param runFolder - the run's own folder, which this start made

 @param worktree - where the run would have worked

 @param branch - the branch the run's worktree is on

 @param reason - what refused the start, in the words the refusal printed

 @throws the error the write gives, such as a run folder that is not there

 @example
 ```ts
 await recordARefusedStart({ runFolder: '/runs/a-1', worktree: '/repo/.claude/worktrees/pi-a', branch: 'fix/run-a', reason: 'no user manager started a unit of its own', });
 ```
 */
export async function recordARefusedStart({
  runFolder,
  worktree,
  branch,
  reason,
}: {
  /**
   The run's own folder, which this start made.
   */
  readonly runFolder: string;

  /**
   Where the run would have worked.
   */
  readonly worktree: string;

  /**
   The branch the run's worktree is on.
   */
  readonly branch: string;

  /**
   What refused the start, in the words the refusal printed.
   */
  readonly reason: string;
},): Promise<void> {
  await writeFile(
    join(
      runFolder,
      REFUSED_START_RECORD_NAME,
    ),
    `${JSON.stringify(
      {
      refusedAt: new Date().toISOString(),
      reason,
      worktree,
      branch,
      whatIsKept: 'the worktree and this run folder are kept: the worktree holds the branch and whatever was committed on it, and this file says why no run of this folder started',
    },
      undefined,
      2,
    )}\n`,
    { encoding: 'utf8', },
  );
}
