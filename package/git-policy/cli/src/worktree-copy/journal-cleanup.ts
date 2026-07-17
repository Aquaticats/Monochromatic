import type {
  Dirent,
  Stats,
} from 'node:fs';
import {
  chmod,
  lstat,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { WorktreeCopyError, } from './errors.ts';

/**
 * Private writable stage-directory mode used during cleanup.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 * Private stage cleanup path is already absent.
 */
const STAGE_PATH_ABSENT: unique symbol = Symbol('worktree-copy stage path is absent',);

/**
 * Reads no-follow stage metadata or completed-cleanup absence.
 *
 * @param path - private stage path
 *
 * @returns no-follow metadata or stage-path absence sentinel
 *
 * @example
 * ```ts
 * await lstatStageOrAbsent('/worktrees/.cli-git-worktree-copy-id');
 * ```
 */
async function lstatStageOrAbsent(
  path: string,
): Promise<Readonly<Stats> | typeof STAGE_PATH_ABSENT> {
  try {
    return await lstat(path,);
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'ENOENT'))
      return STAGE_PATH_ABSENT;
    throw error;
  }
}

/**
 * Restores owner traversal and mutation on staged directories before removal.
 *
 * Source directory modes can intentionally omit owner write permission.
 * Private stage ownership permits restoring only its directory modes before removal.
 *
 * @param stageContainer - validated private stage root
 *
 * @example
 * ```ts
 * await prepareStageRemoval('/worktrees/.cli-git-worktree-copy-id');
 * ```
 */
export async function prepareStageRemoval(
  stageContainer: string,
): Promise<void> {
  /**
   * Pending no-follow private directories.
   */
  const pending: string[] = [stageContainer,];
  while (pending.length > 0) {
    /**
     * Current private directory candidate.
     */
    const directory = pending.pop();
    if (directory === undefined)
      throw new WorktreeCopyError('cli-git: private stage cleanup lost pending directory.',);
    /**
     * Current no-follow metadata or completed-cleanup absence.
     */
    // oxlint-disable-next-line no-await-in-loop -- no-follow cleanup walk remains bounded by private stage
    const stats = await lstatStageOrAbsent(directory,);
    if ((typeof stats) === 'symbol')
      continue;
    if (!stats.isDirectory())
      continue;
    // oxlint-disable-next-line no-await-in-loop -- owner mode restoration is required before child enumeration
    await chmod(
      directory,
      PRIVATE_DIRECTORY_MODE,
    );
    /**
     * Immediate entries after restoring private owner access.
     */
    // oxlint-disable-next-line no-await-in-loop -- child discovery follows restored private directory mode
    const entries = await readdir(
      directory,
      { withFileTypes: true, },
    );
    entries.filter(function childDirectory(entry: Readonly<Dirent>,): boolean {
      return entry.isDirectory();
    },)
      .forEach(function queueDirectory(entry: Readonly<Dirent>,): void {
        pending.push(join(
          directory,
          entry.name,
        ),);
      },);
  }
}
