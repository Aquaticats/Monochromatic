/**
 A table of what a move would strand in every worktree the repository knows: uncommitted changes, untracked files, an
 unfinished merge, rebase, cherry-pick, revert or bisect, and a held index lock. Read-only: `git --no-optional-locks`
 never writes.

 @module
 */

import { createHash, } from 'node:crypto';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { pathExists, } from './completion-records.ts';
import {
  MILLISECONDS_PER_SECOND,
  SCAN_MAX_OUTPUT_BYTES,
  STATUS_FAILURE_LIMIT,
} from './constants.ts';
import { MoveStopError, } from './errors.ts';
import { execFileAsync, } from './exec-file-async.ts';

/**
 The table text and how many worktrees have something a move would strand.
 */
export type UncommittedWorkTable = {
  readonly text: string;
  readonly worktrees: number;
  readonly dirty: number;
};

/**
 Git's own program, never the repository's wrapper.
 */
const GIT = '/usr/bin/git';

/**
 Header of the table; the guided runner reads these columns by position.
 */
const HEADER = 'path\tstate\tchanged\tuntracked\tunfinished\tindexLock\thead\tindexSha256';

/**
 Files or folders in the Git directory that mark an unfinished operation.
 */
const UNFINISHED_MARKERS: readonly string[] = [
  'MERGE_HEAD',
  'CHERRY_PICK_HEAD',
  'REVERT_HEAD',
  'rebase-merge',
  'rebase-apply',
  'BISECT_LOG',
];

/**
 Prefix of an untracked file in `git status --porcelain` output.
 */
const UNTRACKED_PREFIX = '?? ';

/**
 Result of one Git call: success flag, standard output and the failure text.
 */
type GitResult = {
  readonly ok: boolean;
  readonly stdout: string;
  readonly failure: string;
};

/**
 Runs Git in a folder with a time limit.

 @param directory - the folder
 
 @param args - Git arguments after `-C <folder>`
 
 @param timeoutMilliseconds - limit for this call
 
 @returns the result; a failure carries its message instead of throwing
 */
async function git(
  {
    directory,
    args,
    timeoutMilliseconds,
  }: {
    readonly directory: string;
    readonly args: readonly string[];
    readonly timeoutMilliseconds: number;
  },
): Promise<GitResult> {
  try {
    /**
     What the call returned.
     */
    const result = await execFileAsync(
      GIT,
      [
        '--no-optional-locks',
        '-C',
        directory,
        ...args,
      ],
      {
        encoding: 'utf8',
        timeout: timeoutMilliseconds,
        maxBuffer: SCAN_MAX_OUTPUT_BYTES,
      },
    );
    return {
      ok: true,
      stdout: result.stdout,
      failure: '',
    };
  }
  catch (error) {
    return {
      ok: false,
      stdout: '',
      failure: String(error,),
    };
  }
}

/**
 Builds the row of one worktree.

 @param path - the worktree folder
 
 @param timeoutMilliseconds - limit for each Git call
 
 @returns the tab-separated row and whether the worktree is dirty
 */
async function rowOf(
  {
    path,
    timeoutMilliseconds,
  }: {
    readonly path: string;
    readonly timeoutMilliseconds: number;
  },
): Promise<{
  readonly row: string;
  readonly dirty: boolean
}> {
  if (!await pathExists(path,)) {
    return {
      row: `${path}\tmissing\t\t\t\t\t\t`,
      dirty: false,
    };
  }
  /**
   The exit status or result of the call.
   */
  const status = await git({
    directory: path,
    args: [
      'status',
      '--porcelain=v1',
      '--untracked-files=all',
    ],
    timeoutMilliseconds,
  },);
  if (!status.ok) {
    return {
      row: `${path}\tstatus-failed:${status.failure
        .slice(
          0,
          STATUS_FAILURE_LIMIT,
        )}\t\t\t\t\t\t`,
      dirty: false,
    };
  }
  /**
   The lines of text this step works on.
   */
  const lines = status.stdout
    .split('\n',)
    .filter(function notBlank(line,) {
      return line !== '';
    },);
  /**
   The number of untracked files.
   */
  const untracked = lines.filter(function isUntracked(line,) {
    return line.startsWith(UNTRACKED_PREFIX,);
  },)
    .length;
  /**
   The number of changed files.
   */
  const changed = lines.length - untracked;
  /**
   The worktree's Git directory.
   */
  const gitDirectory =
    (await git({
      directory: path,
      args: [
        'rev-parse',
        '--absolute-git-dir',
      ],
      timeoutMilliseconds,
    },)).stdout
      .trim();
  /**
   Which unfinished-operation markers exist.
   */
  const present = await Promise.all(UNFINISHED_MARKERS.map(async function exists(name,) {
    return (gitDirectory !== '') && await pathExists(join(
      gitDirectory,
      name,
    ),);
  },),);
  /**
   The unfinished operations found.
   */
  const unfinished = UNFINISHED_MARKERS.filter(function isPresent(
    _name,
    index,
  ) {
    return present[index] === true;
  },)
    .join(',',);
  /**
   Whether an index lock is held.
   */
  const indexLock = (gitDirectory !== '') && await pathExists(join(
    gitDirectory,
    'index.lock',
  ),);
  /**
   The commit HEAD names.
   */
  const head = (await git({
    directory: path,
    args: [
      'rev-parse',
      'HEAD',
    ],
    timeoutMilliseconds,
  },)).stdout
    .trim();
  /**
   The path of the worktree's index file.
   */
  const indexPath = join(
    gitDirectory,
    'index',
  );
  /**
   The SHA-256 of the index file, or empty when there is none.
   */
  const indexSha256 = (gitDirectory !== '') && await pathExists(indexPath,)
    ? createHash('sha256',)
      .update(await readFile(indexPath,),)
      .digest('hex',)
    : '';
  return {
    row:
      `${path}\tok\t${String(changed,)}\t${String(untracked,)}\t${unfinished}\t${String(indexLock,)}\t${head}\t${indexSha256}`,
    dirty: (changed > 0) || (untracked > 0)
      || (unfinished !== '')
      || indexLock,
  };
}

/**
 Builds the table for every worktree Git registers, the main one included.

 @param mainWorktree - the main worktree folder
 
 @param perWorktreeSeconds - time limit of each Git call
 
 @returns the table text and counts
 
 @throws MoveStopError when Git cannot list the worktrees

 @example
 ```ts
 import { buildUncommittedWorkTable, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = await buildUncommittedWorkTable({ mainWorktree, perWorktreeSeconds, },);
 ```
 */
export async function buildUncommittedWorkTable(
  {
    mainWorktree,
    perWorktreeSeconds,
  }: {
    readonly mainWorktree: string;
    readonly perWorktreeSeconds: number;
  },
): Promise<UncommittedWorkTable> {
  /**
   The limit of each Git call, in milliseconds.
   */
  const timeoutMilliseconds = perWorktreeSeconds * MILLISECONDS_PER_SECOND;
  /**
   The output of the listing command.
   */
  const listing = await git({
    directory: mainWorktree,
    args: [
      'worktree',
      'list',
      '--porcelain',
    ],
    timeoutMilliseconds,
  },);
  if (!listing.ok) {
    throw new MoveStopError(`cannot list worktrees: ${listing.failure}`,);
  }
  /**
   The paths of the registered worktrees.
   */
  const paths = listing.stdout
    .split('\n',)
    .filter(function isWorktreeLine(line,) {
      return line.startsWith('worktree ',);
    },)
    .map(function stripLabel(line,) {
      return line.slice('worktree '.length,);
    },);
  /**
   The rows this step builds or reads.
   */
  const rows = [];
  for (const path of paths) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one Git status at a time, so the scan adds a bounded load while agents work; see doc/lint/sequential-awaits.md
    rows.push(await rowOf({
      path,
      timeoutMilliseconds,
    },),);
  }
  return {
    text: `${
      [
        HEADER,
        ...rows.map(function toText({ row, },) {
          return row;
        },),
      ].join('\n',)
    }\n`,
    worktrees: paths.length,
    dirty: rows.filter(function isDirty({ dirty, },) {
      return dirty;
    },)
      .length,
  };
}
