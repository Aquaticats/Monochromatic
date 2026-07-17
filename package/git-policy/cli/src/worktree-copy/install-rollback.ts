import {
  lstat,
  rmdir,
  unlink,
} from 'node:fs/promises';

import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';

import { entryMatches, } from './entry-compare.ts';
import { filesystemPath, } from './ignored-paths.ts';
import type {
  InstalledWorktreePath,
  StagedWorktreeSnapshot,
} from './model.ts';

/**
 * Reports current relation to exact post-creation filesystem identity.
 *
 * @param path - current destination path
 *
 * @param installed - durable post-creation identity
 *
 * @returns absent, exact identity match, or changed identity
 *
 * @example
 * ```ts
 * await installedIdentityState({ path: '/wt/cache', installed });
 * ```
 */
async function installedIdentityState({
  path,
  installed,
}: Readonly<{
  path: string;
  installed: InstalledWorktreePath;
}>,): Promise<'absent' | 'changed' | 'match'> {
  try {
    /**
     * Current no-follow destination identity.
     */
    const stats = await lstat(
      path,
      { bigint: true, },
    );
    return (stats.dev.toString() === installed.device)
      && (stats.ino.toString() === installed.inode)
      ? 'match'
      : 'changed';
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'ENOENT'))
      return 'absent';
    throw error;
  }
}

/**
 * Removes unchanged entries created by failed installation.
 *
 * @param snapshot - validated stage retained for ownership comparison
 *
 * @param destinationRoot - partially populated worktree
 *
 * @param created - paths created by current installation
 *
 * @returns paths retained because ownership could not be proven
 *
 * @example
 * ```ts
 * await rollbackCreated({ snapshot, destinationRoot: '/wt', created });
 * ```
 */
export async function rollbackCreated({
  snapshot,
  destinationRoot,
  created,
}: Readonly<{
  snapshot: StagedWorktreeSnapshot;
  destinationRoot: string;
  created: readonly InstalledWorktreePath[];
}>,): Promise<readonly string[]> {
  /**
   * Selected expected entries indexed for exact comparison.
   */
  const selectedByPath = new Map(snapshot.entries
    .map(function indexEntry(entry,) {
    return [
      entry.relativePath,
      entry,
    ] as const;
  },),);
  /**
   * Paths retained after conservative rollback.
   */
  const retained: string[] = [];
  for (const installed of created.toReversed()) {
    /**
     * Native installed destination path.
     */
    const destinationPath = filesystemPath({
      root: destinationRoot,
      repositoryPath: installed.relativePath,
    },);
    /**
     * Selected expected entry, absent for scaffolding directory.
     */
    const expected = selectedByPath.get(installed.relativePath,);
    try {
      /**
       * Current relation to proven post-creation identity.
       */
      // oxlint-disable-next-line no-await-in-loop -- exact identity gates every destructive rollback candidate
      const initialIdentity = await installedIdentityState({
        path: destinationPath,
        installed,
      },);
      if (initialIdentity === 'absent')
        continue;
      if (initialIdentity === 'changed') {
        retained.push(`${installed.relativePath}: filesystem identity changed`,);
        continue;
      }
      if (expected !== undefined) {
        /* oxlint-disable no-await-in-loop -- ownership proof precedes each destructive rollback step */
        /**
         * Whether installed selected entry still equals private snapshot.
         */
        const unchanged = await entryMatches({
          expectedRoot: snapshot.stageRoot,
          actualRoot: destinationRoot,
          entry: expected,
        },);
        /* oxlint-enable no-await-in-loop */
        if (!unchanged) {
          retained.push(installed.relativePath,);
          continue;
        }
      }
      /**
       * Identity recheck narrowing replacement race before removal.
       */
      // oxlint-disable-next-line no-await-in-loop -- replacement check must immediately precede destructive rollback
      const finalIdentity = await installedIdentityState({
        path: destinationPath,
        installed,
      },);
      if (finalIdentity !== 'match') {
        if (finalIdentity === 'changed')
          retained.push(`${installed.relativePath}: filesystem identity changed`,);
        continue;
      }
      // oxlint-disable-next-line no-await-in-loop -- child-first rollback requires sequential path removal
      await ((expected?.kind === 'directory') || (!installed.selected)
        ? rmdir(destinationPath,)
        : unlink(destinationPath,));
    }
    catch (error: unknown) {
      retained.push(`${installed.relativePath}: ${caughtValueText(error,)}`,);
    }
  }
  return retained;
}
