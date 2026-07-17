import type { Stats, } from 'node:fs';
import {
  lstat,
  open,
  readlink,
} from 'node:fs/promises';

import { WorktreeCopyError, } from './errors.ts';
import { collectEntryManifest, } from './entry-manifest.ts';
import { filesystemPath, } from './ignored-paths.ts';
import type { WorktreeCopyEntry, } from './model.ts';

/**
 * Bytes in one kibibyte.
 */
const KIBIBYTE_BYTES = 1_024;

/**
 * Kibibytes in one comparison buffer.
 */
const COMPARE_BUFFER_KIBIBYTES = 64;

/**
 * Byte comparison buffer size balancing allocation and read syscall count.
 */
const COMPARE_BUFFER_BYTES = COMPARE_BUFFER_KIBIBYTES * KIBIBYTE_BYTES;

/**
 * Portable permission and special-mode bits retained by copy contract.
 */
const PERMISSION_BITS = 0o7777;

/**
 * Missing filesystem entry sentinel distinct from unsupported entry kinds.
 */
export const WORKTREE_COPY_ENTRY_ABSENT: unique symbol = Symbol('worktree copy entry absent',);

/**
 * Reads no-follow metadata or explicit absence.
 *
 * @param path - filesystem path to inspect
 *
 * @returns lstat metadata or absence sentinel
 *
 * @example
 * ```ts
 * await lstatOrAbsent('/missing');
 * // => WORKTREE_COPY_ENTRY_ABSENT
 * ```
 */
export async function lstatOrAbsent(
  path: string,
): Promise<Readonly<Stats> | typeof WORKTREE_COPY_ENTRY_ABSENT> {
  try {
    return await lstat(path,);
  }
  catch (error: unknown) {
    if (Error.isError(error,)
      && ('code' in error)
      && (error.code === 'ENOENT')) {
      return WORKTREE_COPY_ENTRY_ABSENT;
    }
    throw error;
  }
}

/**
 * Compares regular files without loading complete content into memory.
 *
 * @param leftPath - first regular file
 *
 * @param rightPath - second regular file
 *
 * @returns whether exact bytes match
 *
 * @example
 * ```ts
 * await regularFileBytesEqual({ leftPath: '/a', rightPath: '/b' });
 * ```
 */
async function regularFileBytesEqual({
  leftPath,
  rightPath,
}: Readonly<{
  leftPath: string;
  rightPath: string;
}>,): Promise<boolean> {
  /**
   * Open left file handle disposed after comparison.
   */
  await using left = await open(
    leftPath,
    'r',
  );
  /**
   * Open right file handle disposed after comparison.
   */
  await using right = await open(
    rightPath,
    'r',
  );
  /**
   * Stable initial file sizes from open handles.
   */
  const [leftStats, rightStats,] = await Promise.all([
    left.stat({ bigint: true, },),
    right.stat({ bigint: true, },),
  ],);
  if (leftStats.size !== rightStats.size)
    return false;
  /**
   * Reusable independent read buffers.
   */
  const leftBuffer = Buffer.allocUnsafe(COMPARE_BUFFER_BYTES,);
  /**
   * Reusable right-side read buffer.
   */
  const rightBuffer = Buffer.allocUnsafe(COMPARE_BUFFER_BYTES,);
  for (let position = 0n; position < leftStats.size;) {
    /**
     * Remaining byte count capped to buffer capacity.
     */
    const remaining = leftStats.size - position;
    /**
     * Current read length.
     */
    const length = Number(remaining < BigInt(COMPARE_BUFFER_BYTES,)
      ? remaining
      : BigInt(COMPARE_BUFFER_BYTES,));
    /**
     * Concurrent reads at identical offset.
     */
    // oxlint-disable-next-line no-await-in-loop -- bounded streaming comparison advances one exact chunk at a time
    const [leftRead, rightRead,] = await Promise.all([
      left.read(
        leftBuffer,
        {
          length,
          position,
        },
      ),
      right.read(
        rightBuffer,
        {
          length,
          position,
        },
      ),
    ],);
    if ((leftRead.bytesRead !== length)
      || (rightRead.bytesRead !== length)
      || (!leftBuffer.subarray(
        0,
        length,
      )
        .equals(rightBuffer.subarray(
          0,
          length,
        ),))) {
      return false;
    }
    position += BigInt(length,);
  }
  return true;
}

/**
 * Reports whether one expected entry exactly matches destination entry.
 *
 * @param expectedRoot - root carrying expected content
 *
 * @param actualRoot - root carrying compared content
 *
 * @param entry - expected kind, mode, and repository path
 *
 * @returns whether type, portable mode, target, and bytes match
 *
 * @example
 * ```ts
 * await entryMatches({ expectedRoot: '/stage', actualRoot: '/target', entry });
 * ```
 */
export async function entryMatches({
  expectedRoot,
  actualRoot,
  entry,
}: Readonly<{
  expectedRoot: string;
  actualRoot: string;
  entry: WorktreeCopyEntry;
}>,): Promise<boolean> {
  /**
   * Expected staged filesystem path.
   */
  const expectedPath = filesystemPath({
    root: expectedRoot,
    repositoryPath: entry.relativePath,
  },);
  /**
   * Actual compared filesystem path.
   */
  const actualPath = filesystemPath({
    root: actualRoot,
    repositoryPath: entry.relativePath,
  },);
  /**
   * Actual no-follow metadata or absence.
   */
  const actualStats = await lstatOrAbsent(actualPath,);
  if ((typeof actualStats) === 'symbol')
    return false;
  /**
   * Portable actual permission bits.
   */
  const actualMode = actualStats.mode & PERMISSION_BITS;
  if (actualMode !== entry.mode)
    return false;
  if (entry.kind === 'directory')
    return actualStats.isDirectory();
  if (entry.kind === 'file') {
    return actualStats.isFile()
      && await regularFileBytesEqual({
        leftPath: expectedPath,
        rightPath: actualPath,
      },);
  }
  return actualStats.isSymbolicLink()
    && ((await readlink(expectedPath,)) === (await readlink(actualPath,)));
}

/**
 * Asserts staged manifest exactly matches source after staging completed.
 *
 * @param sourceRoot - source worktree root
 *
 * @param stageRoot - private snapshot payload root
 *
 * @param selectedRoots - Git-selected ignored roots
 *
 * @param excludedSourceRoots - nested worktree and staging exclusions
 *
 * @param stagedEntries - initial source entries materialized in staging
 *
 * @throws {@link WorktreeCopyError} when source changed during transfer
 *
 * @example
 * ```ts
 * await assertFinalSourceEquivalence({ sourceRoot, stageRoot, selectedRoots, excludedSourceRoots, stagedEntries });
 * ```
 */
export async function assertFinalSourceEquivalence({
  sourceRoot,
  stageRoot,
  selectedRoots,
  excludedSourceRoots,
  stagedEntries,
}: Readonly<{
  sourceRoot: string;
  stageRoot: string;
  selectedRoots: readonly string[];
  excludedSourceRoots: readonly string[];
  stagedEntries: readonly WorktreeCopyEntry[];
}>,): Promise<void> {
  /**
   * Final source manifest after staging.
   */
  const finalSourceEntries = await collectEntryManifest({
    root: sourceRoot,
    selectedRoots,
    excludedRoots: excludedSourceRoots,
  },);
  /**
   * Manifest materialized in private stage.
   */
  const finalStageEntries = await collectEntryManifest({
    root: stageRoot,
    selectedRoots,
    excludedRoots: [],
  },);
  if ((finalSourceEntries.length !== stagedEntries.length)
    || (finalStageEntries.length !== stagedEntries.length)) {
    throw new WorktreeCopyError('cli-git: ignored source paths changed while snapshot was staged.',);
  }
  for (const [index, stagedEntry,] of stagedEntries.entries()) {
    /**
     * Final source entry aligned with initial staged manifest.
     */
    const sourceEntry = finalSourceEntries[index];
    /**
     * Final stage entry aligned with initial staged manifest.
     */
    const stageEntry = finalStageEntries[index];
    if ((sourceEntry === undefined)
      || (stageEntry === undefined)
      || (sourceEntry.kind !== stagedEntry.kind)
      || (sourceEntry.mode !== stagedEntry.mode)
      || (sourceEntry.relativePath !== stagedEntry.relativePath)
      || (stageEntry.kind !== stagedEntry.kind)
      || (stageEntry.mode !== stagedEntry.mode)
      || (stageEntry.relativePath !== stagedEntry.relativePath)
      // oxlint-disable-next-line no-await-in-loop -- deterministic fail-fast validation avoids reading later large files
      || (!(await entryMatches({
        expectedRoot: stageRoot,
        actualRoot: sourceRoot,
        entry: stagedEntry,
      },)))) {
      throw new WorktreeCopyError(
        `cli-git: ignored source entry changed while copying: ${JSON.stringify(stagedEntry.relativePath,)}.`,
      );
    }
  }
}
