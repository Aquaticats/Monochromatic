/**
 What the end of a run says: why it ended, whether its worktree's HEAD is a branch, and what the worktree holds uncommitted.

 These are the readings the step after a unit ended decides with, kept together so that step reads as decisions rather than as git
 and JSON.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  fieldsOf,
  isRecord,
} from './read-pi-fields.ts';
import type { RunCommand, } from './run-command.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Reads why a run ended from the summary it wrote.

 @param folder - the run's folder

 @returns the end reason, or nothing when no summary is there

 @example
 ```ts
 const reason = await endReasonIn({ folder: '/runs/a-1', },);
 ```
 */
export async function endReasonIn({
  folder,
}: {
  readonly folder: string;
},): Promise<{ readonly endReason?: string }> {
  try {
    /**
     The summary, which the driver writes whole and last.
     */
    const shown = await readFile(
      join(
        folder,
        'summary.json',
      ),
      'utf8',
    );
    /**
     What the run's summary holds, of any shape.
     */
    const summary: unknown = JSON.parse(shown,);
    if (!isRecord(summary,)) {
      throw new Error(`the summary of ${folder} is no JSON object, so why that run ended cannot be read`,);
    }
    /**
     Why that run ended, as its summary says.
     */
    const { endReason, } = fieldsOf(summary,);
    return ((typeof endReason) === 'string') ? { endReason, } : {};
  }
  catch (error) {
    // A run that wrote no summary ended before the driver could write one, so there is no end reason to weigh, and the file's own
    // absence is what says so.
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    return {};
  }
}

/**
 Says whether the worktree's HEAD is a branch, which a removal needs: a detached HEAD's commits may be reachable from this
 worktree alone.

 @param readOnlyInMain - whether the run works in the checkout itself, which has no worktree of its own

 @param run - how a command is run, which a test replaces

 @param worktree - the worktree the run worked in

 @returns whether the worktree's HEAD is a branch

 @example
 ```ts
 const onABranch = await onABranchNow({ readOnlyInMain: false, run: runCommandWithChildProcess, worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function onABranchNow({
  readOnlyInMain,
  run,
  worktree,
}: {
  readonly readOnlyInMain: boolean;
  readonly run: RunCommand;
  readonly worktree: string;
},): Promise<boolean> {
  if (readOnlyInMain) {
    return false;
  }
  /**
   What git answered about the worktree's HEAD.
   */
  const answered = await run({
    command: 'git',
    words: [
      'symbolic-ref',
      '--quiet',
      'HEAD',
    ],
    folder: worktree,
  },);
  return (answered.status === 0) && (answered.stdout
    .trim()
    !== '');
}

/**
 Reads the worktree's git status, when a removal turns on it.

 @param readOnlyInMain - whether the run works in the checkout itself, whose files are no worktree's

 @param endReason - why the run ended, or nothing when it wrote no summary

 @param onABranch - whether the worktree's HEAD is a branch

 @param run - how a command is run, which a test replaces

 @param worktree - the worktree the run worked in

 @returns how status ended and what it printed, or the same fields as a status nobody needed

 @example
 ```ts
 const status = await statusOf({ readOnlyInMain: false, endReason: 'idle', onABranch: true, run, worktree: '/repo/.claude/worktrees/pi-a', },);
 ```
 */
export async function statusOf({
  readOnlyInMain,
  endReason,
  onABranch,
  run,
  worktree,
}: {
  readonly readOnlyInMain: boolean;
  readonly endReason?: string;
  readonly onABranch: boolean;
  readonly run: RunCommand;
  readonly worktree: string;
},): Promise<{
  readonly status: number;
  readonly statusText: string
}> {
  if (
    readOnlyInMain
      || (!onABranch)
      || ((endReason !== 'idle') && (endReason !== 'stop-file'))
  ) {
    return {
      status: 0,
      statusText: '',
    };
  }
  /**
   What git said about the worktree's files.
   */
  const shown = await run({
    command: 'git',
    words: [
      '--no-optional-locks',
      'status',
      '--porcelain=v1',
      '--untracked-files=normal',
    ],
    folder: worktree,
  },);
  return {
    status: shown.status,
    statusText: shown.stdout,
  };
}
