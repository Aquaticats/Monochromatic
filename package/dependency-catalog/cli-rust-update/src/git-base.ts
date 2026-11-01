/**
 Reads lockfiles as they were at a git revision, for the verification's base.

 @module
 */

import spawn, { SubprocessError, } from 'nano-spawn';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { LOCKFILE_NAME, } from './crates.ts';
import { l, } from './logger.ts';
import type { ReadBaseLock, } from './verify.ts';

//region Errors

/**
 Thrown when the base revision does not name a commit in the repository.

 @example
 ```ts
 throw new BaseRevisionError({ base: 'HEAD~1', detail: 'unknown revision', });
 ```
 */
export class BaseRevisionError extends Error {
  /**
   Builds the error.

   @param base - revision that failed to resolve

   @param detail - git's own message
   */
  constructor({
    base,
    detail,
  }: {
    readonly base: string;
    readonly detail: string
  },) {
    super([
      `The verification base ${JSON.stringify(base,)} is not a commit in this repository: ${detail.trim()}`,
      'Pass --base with a revision that exists locally (in CI, fetch it first),',
      'or omit --base to compare with HEAD.',
    ].join('\n',),);
    this.name = 'BaseRevisionError';
  }
}

//endregion Errors

//region Reader

/**
 Runs git and reports whether it exited 0, keeping its output either way.

 @param root - repository to run in

 @param args - git arguments

 @returns exit outcome with stdout and stderr

 @throws Error when git cannot start at all

 @example
 ```ts
 await runGit({ root, args: ['cat-file', '-e', 'HEAD:Cargo.lock',], });
 ```
 */
async function runGit(
  {
    root,
    args,
  }: {
    readonly root: string;
    readonly args: readonly string[]
  },
): Promise<{
  readonly ok: boolean;
  readonly stdout: string;
  readonly stderr: string
}> {
  try {
    /**
     Finished run.
     */
    const result = await spawn(
      'git',
      [...args,],
      { cwd: root, },
    );
    return {
      ok: true,
      stdout: result.stdout,
      stderr: result.stderr,
    };
  } catch (error) {
    if ((error instanceof SubprocessError) && (error.exitCode !== undefined)) {
      return {
        ok: false,
        stdout: error.stdout,
        stderr: error.stderr,
      };
    }
    throw error;
  }
}

/**
 Lockfile reader for one base revision.

 Resolves the revision once, so a bad `--base` fails before any lockfile is read;
 a crate whose lockfile did not exist at the base reads as empty.

 @param root - repository root

 @param base - git revision to compare with

 @returns reader of one crate's lockfile at the base

 @throws BaseRevisionError when the revision does not name a commit

 @example
 ```ts
 const readBaseLock = await createGitBaseLockReader({ root, base: 'HEAD', });
 ```
 */
export async function createGitBaseLockReader(
  {
    root,
    base,
  }: {
    readonly root: string;
    readonly base: string
  },
): Promise<ReadBaseLock> {
  /**
   Logger for the reader.
   */
  const log = tagged({
    tag: createGitBaseLockReader.name,
    l,
  },);
  /**
   Revision lookup.
   */
  const resolved = await runGit({
    root,
    args: [
      'rev-parse',
      '--verify',
      '--end-of-options',
      `${base}^{commit}`,
    ],
  },);
  if (!resolved.ok) {
    log.error(`base revision ${base} did not resolve: ${resolved.stderr
      .trim()}`,);
    throw new BaseRevisionError({
      base,
      detail: resolved.stderr,
    },);
  }
  /**
   Commit the base names.
   */
  const commit = resolved.stdout
    .trim();
  log.info(`comparing lockfiles with ${base} (${commit})`,);
  return async function readBaseLock(crateDir: string,): Promise<string> {
    /**
     Object name of the lockfile at the base.
     */
    const object = `${commit}:${crateDir}/${LOCKFILE_NAME}`;
    /**
     Lockfile text at the base, when it existed there.
     */
    const shown = await runGit({
      root,
      args: [
        'show',
        object,
      ],
    },);
    if (!shown.ok) {
      log.debug(`${crateDir}/${LOCKFILE_NAME} is absent at ${commit} (${shown.stderr
        .trim()}); every version counts as new`,);
      return '';
    }
    // nano-spawn drops the final newline; restore it so the text matches the file.
    return `${shown.stdout}\n`;
  };
}

//endregion Reader
