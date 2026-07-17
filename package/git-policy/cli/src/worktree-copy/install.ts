import { constants, } from 'node:fs';
import {
  chmod,
  copyFile,
  mkdir,
  readlink,
  symlink,
  unlink,
} from 'node:fs/promises';

import {
  entryMatches,
  lstatOrAbsent,
} from './entry-compare.ts';
import { applyEntryModes, } from './entry-manifest.ts';
import { WorktreeCopyError, } from './errors.ts';
import { filesystemPath, } from './ignored-paths.ts';
import { rollbackCreated, } from './install-rollback.ts';
import {
  type JournalState,
  recordEntryIntent,
} from './transaction-journal.ts';
import type {
  InstalledWorktreePath,
  StagedWorktreeSnapshot,
  WorktreeCopyEntry,
} from './model.ts';

/**
 * Exclusive copy-on-write request with full-copy fallback.
 */
const EXCLUSIVE_COPY_MODE = constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE;

/**
 * Temporary writable mode for newly installed selected directories.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 * Asserts every existing destination entry is identical before mutation.
 *
 * @param snapshot - validated staged source state
 *
 * @param destinationRoot - newly registered worktree root
 *
 * @throws {@link WorktreeCopyError} on first differing collision
 *
 * @example
 * ```ts
 * await preflightDestination({ snapshot, destinationRoot: '/wt' });
 * ```
 */
async function preflightDestination({
  snapshot,
  destinationRoot,
}: Readonly<{
  snapshot: StagedWorktreeSnapshot;
  destinationRoot: string;
}>,): Promise<void> {
  for (const entry of snapshot.entries) {
    /**
     * Destination path aligned with staged entry.
     */
    const destinationPath = filesystemPath({
      root: destinationRoot,
      repositoryPath: entry.relativePath,
    },);
    /* oxlint-disable no-await-in-loop -- fail-fast collision order must follow deterministic manifest */
    /**
     * Current destination no-follow metadata or absence.
     */
    const stats = await lstatOrAbsent(destinationPath,);
    /* oxlint-enable no-await-in-loop */
    if ((typeof stats) === 'symbol')
      continue;
    // oxlint-disable-next-line no-await-in-loop -- exact comparison is required before any destination mutation
    if (!(await entryMatches({
      expectedRoot: snapshot.stageRoot,
      actualRoot: destinationRoot,
      entry,
    },))) {
      throw new WorktreeCopyError(
        `cli-git: ignored-state copy would overwrite differing destination entry ${JSON.stringify(entry.relativePath,)} in ${JSON.stringify(destinationRoot,)}.`,
      );
    }
  }
}

/**
 * Creates missing unselected parent directories for one selected entry.
 *
 * @param destinationRoot - newly registered worktree root
 *
 * @param entry - selected staged entry
 *
 * @param created - mutable transaction-owned creation list
 *
 * @example
 * ```ts
 * await ensureParents({ destinationRoot: '/wt', entry, created: [] });
 * ```
 */
async function ensureParents({
  destinationRoot,
  entry,
  created,
}: Readonly<{
  destinationRoot: string;
  entry: WorktreeCopyEntry;
  created: InstalledWorktreePath[];
}>,): Promise<void> {
  /**
   * Selected path components excluding selected entry itself.
   */
  const parentComponents = entry.relativePath
    .split('/')
    .slice(
      0,
      -1,
    );
  /**
   * Ordered parent repository paths from shallow to deep.
   */
  const parentPaths = parentComponents.map(function parentPath(
    _component,
    index,
  ): string {
    return parentComponents
      .slice(
        0,
        index + 1,
      )
      .join('/');
  },);
  for (const current of parentPaths) {
    /**
     * Current native destination parent.
     */
    const destinationPath = filesystemPath({
      root: destinationRoot,
      repositoryPath: current,
    },);
    /* oxlint-disable no-await-in-loop -- parent chain is ordered and each child depends on prior directory */
    /**
     * Current parent no-follow metadata or absence.
     */
    const stats = await lstatOrAbsent(destinationPath,);
    /* oxlint-enable no-await-in-loop */
    if ((typeof stats) !== 'symbol') {
      if (!stats.isDirectory()) {
        throw new WorktreeCopyError(
          `cli-git: ignored-state parent is not a directory: ${JSON.stringify(current,)} in ${JSON.stringify(destinationRoot,)}.`,
        );
      }
      continue;
    }
    // oxlint-disable-next-line no-await-in-loop -- parent chain creation is necessarily sequential
    await mkdir(destinationPath,);
    created.push({
      relativePath: current,
      selected: false,
    },);
  }
}

/**
 * Creates one absent selected entry from private stage.
 *
 * @param snapshot - validated staged source state
 *
 * @param destinationRoot - newly registered worktree root
 *
 * @param entry - absent selected entry
 *
 * @example
 * ```ts
 * await createSelectedEntry({ snapshot, destinationRoot: '/wt', entry });
 * ```
 */
async function createSelectedEntry({
  snapshot,
  destinationRoot,
  entry,
}: Readonly<{
  snapshot: StagedWorktreeSnapshot;
  destinationRoot: string;
  entry: WorktreeCopyEntry;
}>,): Promise<void> {
  /**
   * Staged expected filesystem path.
   */
  const stagePath = filesystemPath({
    root: snapshot.stageRoot,
    repositoryPath: entry.relativePath,
  },);
  /**
   * Destination filesystem path.
   */
  const destinationPath = filesystemPath({
    root: destinationRoot,
    repositoryPath: entry.relativePath,
  },);
  if (entry.kind === 'directory') {
    await mkdir(
      destinationPath,
      { mode: PRIVATE_DIRECTORY_MODE, },
    );
    return;
  }
  if (entry.kind === 'file') {
    await copyFile(
      stagePath,
      destinationPath,
      EXCLUSIVE_COPY_MODE,
    );
    try {
      await chmod(
        destinationPath,
        entry.mode,
      );
      return;
    }
    catch (error: unknown) {
      await unlink(destinationPath,);
      throw error;
    }
  }
  /**
   * Exact staged symbolic-link target text.
   */
  const target = await readlink(stagePath,);
  await symlink(
    target,
    destinationPath,
  );
}

/**
 * Installs validated ignored snapshot without overwriting destination state.
 *
 * @param snapshot - validated private source snapshot
 *
 * @param destinationRoot - newly registered worktree root
 *
 * @param journalState - mutable durable transaction state
 *
 * @mutates journalState - records each selected path intent through {@link recordEntryIntent}
 *
 * @returns count of newly installed selected entries
 *
 * @throws {@link WorktreeCopyError} after ownership-checked rollback on failure
 *
 * @example
 * ```ts
 * await installSnapshot({ snapshot, destinationRoot: '/wt', journalState });
 * ```
 */
export async function installSnapshot({
  snapshot,
  destinationRoot,
  journalState,
}: Readonly<{
  snapshot: StagedWorktreeSnapshot;
  destinationRoot: string;
  journalState: JournalState;
}>,): Promise<number> {
  /**
   * Prior selected intents conservatively owned only when they still equal stage.
   */
  const created: InstalledWorktreePath[] = journalState.pending
    .record
    .intendedEntries
    .map(function priorIntent(relativePath,): InstalledWorktreePath {
      return {
        relativePath,
        selected: true,
      };
    },);
  try {
    await preflightDestination({
      snapshot,
      destinationRoot,
    },);
    for (const entry of snapshot.entries) {
      // oxlint-disable-next-line no-await-in-loop -- manifest parent order and rollback ownership require sequential install
      await ensureParents({
        destinationRoot,
        entry,
        created,
      },);
      /**
       * Current destination path after parent creation.
       */
      const destinationPath = filesystemPath({
        root: destinationRoot,
        repositoryPath: entry.relativePath,
      },);
      // oxlint-disable-next-line no-await-in-loop -- destination can change between preflight and exact exclusive creation
      if ((typeof await lstatOrAbsent(destinationPath,)) !== 'symbol')
        continue;
      // oxlint-disable-next-line no-await-in-loop -- intent must be durable before selected destination mutation
      await recordEntryIntent({
        state: journalState,
        relativePath: entry.relativePath,
      },);
      if (!created.some(function sameIntent(installed,): boolean {
        return installed.selected && (installed.relativePath === entry.relativePath);
      },)) {
        created.push({
          relativePath: entry.relativePath,
          selected: true,
        },);
      }
      // oxlint-disable-next-line no-await-in-loop -- deterministic parent-before-child installation
      await createSelectedEntry({
        snapshot,
        destinationRoot,
        entry,
      },);
    }
    await applyEntryModes({
      root: destinationRoot,
      entries: snapshot.entries,
    },);
    return created.filter(function selectedEntry(installed,): boolean {
      return installed.selected;
    },)
      .length;
  }
  catch (error: unknown) {
    /**
     * Paths that rollback could not safely remove.
     */
    const retained = await rollbackCreated({
      snapshot,
      destinationRoot,
      created,
    },);
    /**
     * Incomplete rollback suffix retaining exact paths.
     */
    const suffix = retained.length === 0
      ? ''
      : ` Rollback retained: ${retained.map(function quotedPath(path,): string {
          return JSON.stringify(path,);
        },)
        .join(', ',)}.`;
    throw new WorktreeCopyError(
      `cli-git: ignored-state installation failed for ${JSON.stringify(destinationRoot,)}.${suffix}`,
      error,
    );
  }
}
