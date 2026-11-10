/**
 The worktree one run works in: made from the run's branch when it is not there, with its dependencies linked.

 A folder without a worktree marker is not a worktree, and git run there would act on the main checkout, so the run's branch is
 only ever checked out where git says it is.

 @module
 */

import { rmdir, } from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { worktreeOfARun, } from './pi-runs-folders.ts';
import type { RunCommand, } from './run-command.ts';
import {
  NO_WORKTREE_BRANCH,
  type StartPiRunOptions,
} from './start-pi-run-options.ts';
import { PiRunNotStartedError, } from './pi-run-not-started-error.ts';
/**
 The file that says a folder is a worktree.
 */
const WORKTREE_MARKER = '.git';

/**
 The summary a finished attempt leaves last, which names the session a resume reads.
 */
const SUMMARY_FILE = 'summary.json';

/**
 The branch a new worktree is made from, when its branch does not exist yet.
 */
const WORKTREE_BASE_BRANCH = 'main';

/**
 Reads whether a folder holds a worktree of this repository.

 @param folder - the folder to read

 @param run - how a command is run

 @returns whether git would treat the folder as a worktree

 @example
 ```ts
 const present = await worktreeIsThere({ folder: '/repo/.claude/worktrees/pi-a', run: runCommandWithChildProcess, });
 ```
 */
async function worktreeIsThere({
  folder,
  run,
}: {
  readonly folder: string;
  readonly run: RunCommand;
},): Promise<boolean> {
  /**
   What the folder holds.
   */
  const listed = await run({
    command: 'git',
    words: [
      '-C',
      folder,
      'rev-parse',
      '--git-dir',
    ],
  },);
  return listed.status === 0;
}

/**
 Makes the worktree a run works in, or finds the one already there.

 @param options - what the run was asked for

 @param run - how a command is run

 @returns the folder Pi starts in, and the lines to print about preparing it

 @throws PiRunNotStartedError when git could not make the worktree

 @example
 ```ts
 const prepared = await workingFolderIn({ options, run: runCommandWithChildProcess, });
 ```
 */
export async function workingFolderIn({
  options,
  run,
}: {
  readonly options: StartPiRunOptions;
  readonly run: RunCommand;
},): Promise<{
  readonly folder: string;
  readonly lines: readonly string[];
}> {
  if (options.branch === NO_WORKTREE_BRANCH) {
    return {
      folder: options.repository,
      lines: [],
    };
  }
  /**
   Where this run's branch is checked out.
   */
  const worktree = worktreeOfARun({
    repository: options.repository,
    name: options.name,
  },);
  if (await worktreeIsThere({
    folder: worktree,
    run,
  },)) {
    return {
      folder: worktree,
      lines: [],
    };
  }
  // A folder without a worktree marker is not a worktree: inside the repository, git run there would act on the main checkout.
  // `rmdir` removes only an empty folder and throws on anything else, so nothing anyone wrote can be lost here.
  try {
    await rmdir(worktree,);
  }
  catch (error) {
    // Nothing at that path is the ordinary case, and a folder holding anything is refused rather than emptied.
    if (errorCodeOf(error,) !== 'ENOENT') {
      throw new PiRunNotStartedError(
        `${options.name}: ${worktree} exists and is not empty, so this start left it alone: ${String(error,)}`,
        { cause: error, },
      );
    }
  }
  /**
   Whether the branch already exists, in which case the worktree checks it out rather than making one.
   */
  const branchThere = await run({
    command: 'git',
    words: [
      '-C',
      options.repository,
      'rev-parse',
      '--verify',
      '--quiet',
      `refs/heads/${options.branch}`,
    ],
  },);
  /**
   What git made of the worktree, which is the line a person reads when it failed.
   */
  const added = await run({
    command: 'git',
    words: [
      '-C',
      options.repository,
      'worktree',
      'add',
      ...((branchThere.status === 0)
        ? [
          worktree,
          options.branch,
        ]
        : [
          '-b',
          options.branch,
          worktree,
          WORKTREE_BASE_BRANCH,
        ]),
    ],
  },);
  if (added.status !== 0) {
    throw new PiRunNotStartedError(
      `${options.name}: git could not make ${worktree}: ${added.problem
        ?? added.stderr
          .trim()}`,
    );
  }
  return {
    folder: worktree,
    lines: [`worktree add: status ${String(added.status,)}`,],
  };
}

/**
 Links the worktree's dependencies, which a fresh worktree needs before anything runs in it.

 @param folder - the worktree Pi starts in

 @param run - how a command is run

 @returns the line to print about it

 @example
 ```ts
 const line = await dependenciesIn({ folder: '/repo/.claude/worktrees/pi-a', run: runCommandWithChildProcess, });
 ```
 */
export async function dependenciesIn({
  folder,
  run,
}: {
  readonly folder: string;
  readonly run: RunCommand;
},): Promise<string> {
  /**
   What `pnpm` made of the install, which a start reports and goes on from.
   */
  const linked = await run({
    command: 'mise',
    words: [
      'exec',
      '--',
      'pnpm',
      'install',
      '--offline',
      '--frozen-lockfile',
    ],
    cwd: folder,
  },);
  return `dependencies: status ${String(linked.status,)}`;
}
