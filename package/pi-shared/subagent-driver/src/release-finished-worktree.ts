/**
 Removing the worktree of a run that finished its work, so the snapshot it holds of the main branch stops pinning the files it
 copied, and keeping every worktree that holds anything of its own.

 The owner, 2026-11-10: "You're the coordinator and you're supposed to remove finished worktrees automatically." and "The
 wrapper should have cleaned up after itself automatically deterministically." A worktree here is a btrfs snapshot of the main
 branch, which keeps that branch's files as they were when it was taken, build folders included, until the worktree is removed:
 460 of them filled this machine's disk on 2026-11-10.

 The branch stays, so a later run of the same name adds its worktree again from it, which is what the start does. Only a run
 that finished its work is removed: one that ended on a cap or on a model error is likely to be resumed, and an add costs
 minutes on this volume. A worktree whose HEAD is detached is kept too, because the commits made there may be reachable from
 that worktree alone.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import type { RunCommand, } from './run-command.ts';

/**
 The ends a run finishes its work on: it came back, or the stop file ended it. Every other end leaves work behind that a
 resume would want.
 */
export const END_REASONS_THAT_FINISHED_THE_WORK: readonly string[] = [
  'idle',
  'stop-file',
];

/**
 How many characters of a porcelain status line come before the path: the two status letters and the space between them.
 */
const PORCELAIN_STATUS_CHARACTERS = 3;

/**
 How many paths of uncommitted work are named when they are why a worktree is kept, which is enough to recognise the work and
 short enough that the answer stays one line.
 */
export const KEPT_PATHS_NAMED = 5;

/**
 The file a finished run leaves last, which says how it ended.
 */
const SUMMARY_FILE = 'summary.json';

/**
 The Git environment every call keeps, so a worktree is read and removed the same way whatever a person's Git is configured to
 do: no helper is asked for a credential, and no terminal is prompted.
 */
const GIT_ENVIRONMENT: Readonly<Record<string, string>> = {
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 No run folder holds a summary, as a symbol so no summary's end can be mistaken for it.
 */
const NO_SUMMARY: unique symbol = Symbol('the run folder holds no summary',);

/**
 What releasing a worktree did.
 */
export type WorktreeRelease = {
  /**
   Whether the worktree is gone.
   */
  readonly released: boolean;

  /**
   The lines to print, each saying what became of the worktree and why.
   */
  readonly lines: readonly string[];
};

/**
 Reads whether a run finished its work, which is the only kind of run whose worktree is removed.

 @param endReason - what the run's summary says it ended on, or {@link NO_SUMMARY}

 @returns whether the worktree may be removed

 @example
 ```ts
 const finished = endedWithItsWork({ endReason: 'idle', });
 ```
 */
export function endedWithItsWork({
  endReason,
}: {
  readonly endReason: string | typeof NO_SUMMARY;
},): boolean {
  return ((typeof endReason) === 'string')
    && END_REASONS_THAT_FINISHED_THE_WORK.includes(endReason,);
}

/**
 Reads the summary a run folder holds, or says it holds none to read.

 @param runFolder - the folder the run wrote into

 @returns the summary's text, or {@link NO_SUMMARY}

 @throws the error a read gave for a summary that is there and cannot be read

 @example
 ```ts
 const text = await summaryTextOf({ runFolder: '/runs/a-1', });
 ```
 */
async function summaryTextOf({
  runFolder,
}: {
  readonly runFolder: string;
},): Promise<string | typeof NO_SUMMARY> {
  try {
    return await readFile(
      join(
        runFolder,
        SUMMARY_FILE,
      ),
      { encoding: 'utf8', },
    );
  }
  catch (error) {
    // A run whose summary was never written says nothing about having finished, so its worktree is kept.
    if (errorCodeOf(error,) === 'ENOENT') {
      return NO_SUMMARY;
    }
    throw error;
  }
}

/**
 Reads how the run whose folder this is ended.

 @param runFolder - the folder the run wrote into

 @returns the end the summary names, or {@link NO_SUMMARY} when it left none to read

 @throws the error a read gave for a summary that is there and cannot be read, or one that is not JSON

 @example
 ```ts
 const endReason = await endReasonIn({ runFolder: '/runs/a-1', });
 ```
 */
export async function endReasonIn({
  runFolder,
}: {
  readonly runFolder: string;
},): Promise<string | typeof NO_SUMMARY> {
  /**
   What the run left last, or {@link NO_SUMMARY} when it left none to read.
   */
  const text = await summaryTextOf({ runFolder, },);
  if ((typeof text) === 'symbol') {
    return NO_SUMMARY;
  }
  /**
   What the summary says it ended on.
   */
  const {endReason} = (JSON.parse(text,) as { readonly endReason?: unknown });
  return ((typeof endReason) === 'string') ? endReason : NO_SUMMARY;
}

/**
 Reads the paths that hold uncommitted work, when any do.

 @param run - how a command is run

 @param worktree - the worktree to read

 @returns the paths, empty when the worktree holds nothing uncommitted

 @throws the error a failed Git command gave, which is read as work kept rather than as work absent

 @example
 ```ts
 const paths = await uncommittedPathsIn({ run: runCommandWithChildProcess, worktree: '/repo/.claude/worktrees/pi-a', });
 ```
 */
async function uncommittedPathsIn({
  run,
  worktree,
}: {
  readonly run: RunCommand;
  readonly worktree: string;
},): Promise<readonly string[] | typeof STATUS_UNREADABLE> {
  /**
   What Git said of the worktree.
   */
  const shown = await run({
    command: 'git',
    words: [
      '--no-optional-locks',
      '-C',
      worktree,
      'status',
      '--porcelain=v1',
      '--untracked-files=normal',
    ],
    environment: GIT_ENVIRONMENT,
  },);
  if (shown.status !== 0) {
    return STATUS_UNREADABLE;
  }
  return shown.stdout
    .split('\n',)
    .filter(function holdsAPath(line,) {
      return line.trim() !== '';
    },)
    .map(function asPath(line,) {
      // A porcelain line is a status field, a space, and the path the work is in.
      return line.slice(PORCELAIN_STATUS_CHARACTERS,);
    },);
}

/**
 Git's answer could not be read, as a symbol so no path list can be mistaken for it.
 */
const STATUS_UNREADABLE: unique symbol = Symbol('Git could not read the worktree',);

/**
 Releases the worktree of a run that finished its work, and says what became of it.

 @param run - how a command is run

 @param repository - the checkout whose worktrees this one belongs to

 @param worktree - the worktree the run worked in

 @param runFolder - the folder the run wrote into, which says how it ended

 @returns whether the worktree is gone, and the lines that say why

 @throws the error a read of the summary gave

 @example
 ```ts
 const release = await releaseFinishedWorktree({ run: runCommandWithChildProcess, repository: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', });
 ```
 */
export async function releaseFinishedWorktree({
  run,
  repository,
  worktree,
  runFolder,
}: {
  readonly run: RunCommand;
  readonly repository: string;
  readonly worktree: string;
  readonly runFolder: string;
},): Promise<WorktreeRelease> {
  /**
   What the run ended on.
   */
  const endReason = await endReasonIn({ runFolder, },);
  if (!endedWithItsWork({ endReason, },)) {
    return {
      released: false,
      lines: [`worktree kept (the run ended ${((typeof endReason) === 'string') ? endReason : 'with no summary to say'})`,],
    };
  }
  /**
   Whether the worktree has a branch checked out, which a detached HEAD does not.
   */
  const onABranch = await run({
    command: 'git',
    words: [
      '-C',
      worktree,
      'symbolic-ref',
      '--quiet',
      'HEAD',
    ],
    environment: GIT_ENVIRONMENT,
  },);
  if (onABranch.status !== 0) {
    return {
      released: false,
      lines: ['worktree kept (its HEAD is detached, so commits made there may be reachable from it alone)',],
    };
  }
  /**
   The paths the worktree holds uncommitted work in, or Git's silence.
   */
  const uncommitted = await uncommittedPathsIn({
    run,
    worktree,
  },);
  if ((typeof uncommitted) === 'symbol') {
    return {
      released: false,
      lines: ['worktree kept (Git could not read its status, so nothing is removed on a guess)',],
    };
  }
  if (uncommitted.length > 0) {
    return {
      released: false,
      lines: [`worktree kept, it holds uncommitted work: ${uncommitted.slice(
        0,
        KEPT_PATHS_NAMED,
      )
        .join(', ',)}`,],
    };
  }
  // The wrapper's own `git worktree remove`, and never `--force`: a worktree that turned out to hold something is refused there
  // rather than emptied here, so nothing anyone wrote is lost even if this read was raced.
  /**
   What Git made of the removal.
   */
  const removed = await run({
    command: 'git',
    words: [
      '-C',
      repository,
      'worktree',
      'remove',
      worktree,
    ],
    environment: GIT_ENVIRONMENT,
  },);
  if (removed.status !== 0) {
    return {
      released: false,
      lines: [`worktree kept (Git refused to remove it): ${removed.problem
        ?? removed.stderr
          .trim()}`,],
    };
  }
  return {
    released: true,
    lines: ['worktree removed, and its branch kept'],
  };
}
