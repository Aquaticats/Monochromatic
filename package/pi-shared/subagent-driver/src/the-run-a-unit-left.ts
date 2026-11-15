/**
 What a unit left in a run's folder, and the line a launcher prints when the run's worktree has gone.

 Why: a launcher tells "no unit was started and no run happened" only when none did. Whether a run happened is read from what a
 run leaves, never inferred from a unit's exit status and a missing worktree: a run that ran and failed, and whose worktree
 another launcher of the same name then removed, must be reported as the run it was, with what it kept and where.

 The driver writes `feed.jsonl` from the first moment of a run and `summary.json` last, so either of the two says a run
 happened. The unit's own record is not read: the launcher asks for its unit with `--collect`, and the manager has already
 collected the unit by the time its waiter ends, which was measured here (`ExecMainStartTimestamp=` empty, `Result=success`, for a
 unit that ran and exited 1, asked right after `systemd-run --wait --collect` returned).

 @module
 */

import {
  access,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { endReasonIn, } from './end-of-a-run.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { worktreeGoneWordsOf, } from './file-system-of-a-run.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The file the driver writes from the first moment of a run, which says a run happened.
 */
const FEED_FILE_NAME: string = 'feed.jsonl';

/**
 The file the driver writes last, which says a run happened and how it ended.
 */
const SUMMARY_FILE_NAME: string = 'summary.json';

/**
 What a unit left in one run's folder.

 @example
 ```ts
 const left = await theRunAUnitLeftIn({ runFolder: '/runs/a-1', });
 ```
 */
export type TheRunAUnitLeft = {
  /**
   Whether a run happened at all, which its feed or its summary says.
   */
  readonly ran: boolean;

  /**
   How the run ended, when its summary says.
   */
  readonly endReason?: string;

  /**
   Whether the run's folder holds a summary that is not an ordinary file to read, so how it ended cannot be read.
   */
  readonly summaryIsNotAnOrdinaryFile?: boolean;

  /**
   The files the run left in its folder, each named once, in name order.
   */
  readonly kept: readonly string[];
};

/**
 Reads what a unit left in one run's folder.

 @param runFolder - the run's own folder, which the launcher made before it asked for the unit

 @returns whether a run happened, how it ended, and the files it left

 @throws Error when the folder cannot be read for a reason other than being absent

 @example
 ```ts
 const left = await theRunAUnitLeftIn({ runFolder: '/runs/a-1', });
 ```
 */
export async function theRunAUnitLeftIn({ runFolder, }: { readonly runFolder: string },): Promise<TheRunAUnitLeft> {
  /**
   The names of the files in the run's folder, which a run that never started leaves empty.
   */
  const kept = await namesIn({ folder: runFolder, },);
  /**
   Whether the run wrote its feed or its summary, which are the two files a run writes by itself.
   */
  const ran = kept.includes(FEED_FILE_NAME,) || kept.includes(SUMMARY_FILE_NAME,);
  if (!ran) {
    return {
      ran: false,
      kept,
    };
  }
  /**
   What the run's own summary says about its ending, and whether it could be read at all.
   */
  const {
    endReason,
    summaryIsNotAnOrdinaryFile,
  } = await endReasonIn({ folder: runFolder, },);
  return {
    ran: true,
    ...((endReason === undefined) ? {} : { endReason, }),
    ...((summaryIsNotAnOrdinaryFile === undefined) ? {} : { summaryIsNotAnOrdinaryFile, }),
    kept,
  };
}

/**
 Reads the names of the files in a folder, which is no names when the folder is not there.

 @param folder - the folder to read

 @returns the names, in name order

 @throws Error when the folder cannot be read for a reason other than being absent

 @example
 ```ts
 const names = await namesIn({ folder: '/runs/a-1', });
 ```
 */
async function namesIn({ folder, }: { readonly folder: string },): Promise<readonly string[]> {
  try {
    return (await readdir(folder,)).toSorted();
  }
  catch (error) {
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    return [];
  }
}

/**
 Writes the line a launcher prints when a run's worktree is gone after its unit ended, from what the run left.

 The line says the run happened, how it ended, that another launcher of the same name removed the shared worktree, and where the
 run's records and commits are, so a reader never takes a real run for a start that never happened.

 @param runName - the run's name, which names the worktree every start of it shares

 @param worktree - the worktree that has gone

 @param runFolder - the run's own folder, which holds what it left

 @param branch - the branch the run's work went on

 @param unitStatus - the status the unit ended with

 @param left - what the run left in its folder

 @returns the line to print

 @example
 ```ts
 const line = theRunThatOutlivedItsWorktreeLineOf({ runName: 'a', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', branch: 'fix/a', unitStatus: 1, left: { ran: true, endReason: 'pi-exited', kept: ['feeds.jsonl', 'summary.json'] }, });
 ```
 */
export function theRunThatOutlivedItsWorktreeLineOf({
  runName,
  worktree,
  runFolder,
  branch,
  unitStatus,
  left,
}: {
  /**
   The run's name, which names the worktree every start of it shares.
   */
  readonly runName: string;

  /**
   The worktree that has gone.
   */
  readonly worktree: string;

  /**
   The run's own folder, which holds what it left.
   */
  readonly runFolder: string;

  /**
   The branch the run's work went on.
   */
  readonly branch: string;

  /**
   The status the unit ended with.
   */
  readonly unitStatus: number;

  /**
   What the run left in its folder.
   */
  readonly left: TheRunAUnitLeft;
},): string {
  /**
   How the run's own records say it ended, which is its summary when it wrote one.
   */
  const ending = (left.summaryIsNotAnOrdinaryFile === true)
    ? `the run's summary is not an ordinary file, so how it ended cannot be read, and its unit ended with status ${String(unitStatus,)}`
    : ((left.endReason === undefined)
      ? `the run ended before it wrote a summary, and its unit ended with status ${String(unitStatus,)}`
      : `the run ended as ${left.endReason}, and its unit ended with status ${String(unitStatus,)}`);
  /**
   The files the run left in its folder, which are what a reader can still look at.
   */
  const files = left.kept;
  /**
   Those files as one list, which the line names.
   */
  const keptNames = files.join(', ',);
  /**
   What the line says about those files, which is that there are none when the run left none.
   */
  const kept = (files.length === 0)
    ? `Its run folder ${runFolder} is empty`
    : `What it left is in ${runFolder}: ${keptNames}`;
  return `${ending}, and its worktree ${worktree} is gone: a launcher of another start of the name ${runName} removes a finished worktree, and it removed this one while this launcher waited for the unit. ${kept}. Its branch ${branch} and every commit on it are in the Git directory the worktree named; what the worktree held that was never committed, including the files Git ignores, is not there any more. Read what the run left before starting it again, and start it again only for work it did not do.`;
}

/**
 Says whether a folder is there.

 @param folder - the folder to look for

 @returns whether the folder is there

 @throws Error when the folder's own state cannot be read, which is no answer about whether it is there

 @example
 ```ts
 const there = await folderIsThere({ folder: '/repo/.claude/worktrees/pi-a', });
 ```
 */
async function folderIsThere({ folder, }: { readonly folder: string },): Promise<boolean> {
  try {
    await access(folder,);
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    return false;
  }
}

/**
 Writes the line a launcher prints when the run's worktree has gone after its unit ended, or nothing when the worktree is there.

 A run that happened is reported as that, from what it left in its folder; a unit that never started, and so ran nothing, is
 reported as that. Neither is inferred from the unit's status and a missing folder alone.

 @param worktree - the worktree Git made for the run

 @param runName - the run's name, which names the worktree every start of it shares

 @param runFolder - the run's own folder, which holds what the run left

 @param branch - the branch the run's work went on

 @param unitStatus - the status the unit ended with

 @returns the line to print, or nothing when the worktree is there

 @throws Error when the run's folder cannot be read for a reason other than being absent

 @example
 ```ts
 const { line, } = await theWorktreeThatWentLineIn({ worktree: '/repo/.claude/worktrees/pi-a', runName: 'a', runFolder: '/runs/a-1', branch: 'fix/a', unitStatus: 1, });
 ```
 */
export async function theWorktreeThatWentLineIn({
  worktree,
  runName,
  runFolder,
  branch,
  unitStatus,
}: {
  /**
   The worktree Git made for the run.
   */
  readonly worktree: string;

  /**
   The run's name, which names the worktree every start of it shares.
   */
  readonly runName: string;

  /**
   The run's own folder, which holds what the run left.
   */
  readonly runFolder: string;

  /**
   The branch the run's work went on.
   */
  readonly branch: string;

  /**
   The status the unit ended with.
   */
  readonly unitStatus: number;
},): Promise<{ readonly line?: string }> {
  if (await folderIsThere({ folder: worktree, },)) {
    return {};
  }
  /**
   What the unit left in the run's folder, which says whether a run happened at all.
   */
  const left = await theRunAUnitLeftIn({ runFolder, },);
  if (!left.ran) {
    /**
     The words for a unit that never started, which is the start this launcher refused before asking.
     */
    const gone = worktreeGoneWordsOf({
      worktree,
      runName,
      runFolder,
    },);
    return { line: `the unit did not start: ${gone}`, };
  }
  return {
    line: theRunThatOutlivedItsWorktreeLineOf({
      runName,
      worktree,
      runFolder,
      branch,
      unitStatus,
      left,
    },),
  };
}
