import { constants, } from 'node:fs';
import {
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  readlink,
  rm,
  symlink,
} from 'node:fs/promises';
import {
  dirname,
  isAbsolute,
  join,
  relative,
  resolve,
} from 'node:path';

import { WorktreeCopyError, } from './errors.ts';
import { assertFinalSourceEquivalence, } from './entry-compare.ts';
import {
  applyEntryModes,
  collectEntryManifest,
} from './entry-manifest.ts';
import {
  filesystemPath,
  readIgnoredRoots,
} from './ignored-paths.ts';
import type {
  StagedWorktreeSnapshot,
  WorktreeCopyEntry,
} from './model.ts';

/**
 * Private staging directory prefix beside destination worktree.
 */
export const STAGE_PREFIX = '.cli-git-worktree-copy-';

/**
 * Private staging directory mode.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 * Copy-on-write request with documented full-copy fallback.
 */
const COPY_MODE = constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE;

/**
 * Reports component-aware native path containment.
 *
 * @param candidate - absolute candidate path
 *
 * @param parent - absolute possible parent path
 *
 * @returns whether candidate equals or descends from parent
 *
 * @example
 * ```ts
 * pathWithin({ candidate: '/repo/cache', parent: '/repo' });
 * // => true
 * ```
 */
function pathWithin({
  candidate,
  parent,
}: Readonly<{
  candidate: string;
  parent: string;
}>,): boolean {
  /**
   * Native relative path from parent.
   */
  const local = relative(
    parent,
    candidate,
  );
  return (local === '') || ((!local.startsWith('..',)) && (!isAbsolute(local,)));
}

/**
 * Filters registered worktrees and private stage out of source traversal.
 *
 * @param sourceRoot - canonical source worktree
 *
 * @param registeredRoots - every current registered worktree root
 *
 * @param stageContainer - private stage potentially nested beneath source
 *
 * @returns absolute roots omitted from source snapshot
 *
 * @example
 * ```ts
 * sourceExclusions({ sourceRoot: '/repo', registeredRoots: ['/repo', '/repo/.agent/wt'], stageContainer: '/tmp/s' });
 * // => ['/repo/.agent/wt']
 * ```
 */
function sourceExclusions({
  sourceRoot,
  registeredRoots,
  stageContainer,
}: Readonly<{
  sourceRoot: string;
  registeredRoots: readonly string[];
  stageContainer: string;
}>,): readonly string[] {
  return [
    ...registeredRoots,
    stageContainer,
  ]
    .map(function normalizeRoot(root,): string {
      return resolve(root,);
    },)
    .filter(function nestedRoot(root,): boolean {
      return (root !== sourceRoot) && pathWithin({
        candidate: root,
        parent: sourceRoot,
      },);
    },);
}

/**
 * Copies validated manifest entries into private destination-filesystem stage.
 *
 * Entry-wise copying remains valid when private stage is nested beneath a
 * selected source directory.
 *
 * @param sourceRoot - canonical source worktree
 *
 * @param stageRoot - private payload root
 *
 * @param entries - parent-first no-follow source manifest
 *
 * @example
 * ```ts
 * await copyManifestEntries({ sourceRoot: '/repo', stageRoot: '/stage', entries });
 * ```
 */
async function copyManifestEntries({
  sourceRoot,
  stageRoot,
  entries,
}: Readonly<{
  sourceRoot: string;
  stageRoot: string;
  entries: readonly WorktreeCopyEntry[];
}>,): Promise<void> {
  for (const entry of entries) {
    /**
     * Source manifest entry path.
     */
    const sourcePath = filesystemPath({
      root: sourceRoot,
      repositoryPath: entry.relativePath,
    },);
    /**
     * Staged manifest entry path.
     */
    const stagePath = filesystemPath({
      root: stageRoot,
      repositoryPath: entry.relativePath,
    },);
    if (entry.kind === 'directory') {
      // oxlint-disable-next-line no-await-in-loop -- parent-first manifest materializes one deterministic directory at a time
      await mkdir(
        stagePath,
        {
          recursive: true,
          mode: PRIVATE_DIRECTORY_MODE,
        },
      );
      continue;
    }
    // oxlint-disable-next-line no-await-in-loop -- selected nested files can require unselected private scaffold parents
    await mkdir(
      dirname(stagePath,),
      {
        recursive: true,
        mode: PRIVATE_DIRECTORY_MODE,
      },
    );
    if (entry.kind === 'file') {
      // oxlint-disable-next-line no-await-in-loop -- file staging remains deterministic and bounded by manifest
      await copyFile(
        sourcePath,
        stagePath,
        COPY_MODE,
      );
      continue;
    }
    // oxlint-disable-next-line no-await-in-loop -- no-follow symbolic-link target must be captured from source entry
    const target = await readlink(sourcePath,);
    // oxlint-disable-next-line no-await-in-loop -- symbolic-link creation follows deterministic manifest order
    await symlink(
      target,
      stagePath,
    );
  }
}

/**
 * Creates exact ignored-state snapshot staged beside destination worktree.
 *
 * @param sourceRoot - canonical source worktree
 *
 * @param destinationRoot - canonical newly created worktree
 *
 * @param registeredRoots - canonical roots excluded from recursive source copy
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns validated private snapshot
 *
 * @throws {@link WorktreeCopyError} when source is unsupported or changes
 *
 * @example
 * ```ts
 * await stageIgnoredSnapshot({ sourceRoot: '/repo', destinationRoot: '/wt', registeredRoots: ['/repo', '/wt'], gitPath: '/usr/bin/git' });
 * ```
 */
export async function stageIgnoredSnapshot({
  sourceRoot,
  destinationRoot,
  registeredRoots,
  gitPath,
}: Readonly<{
  sourceRoot: string;
  destinationRoot: string;
  registeredRoots: readonly string[];
  gitPath: string;
}>,): Promise<StagedWorktreeSnapshot> {
  /**
   * Existing registered roots nested strictly beneath source worktree.
   */
  const nestedRegisteredRoots = registeredRoots
    .map(function normalizedRegisteredRoot(root,): string {
      return resolve(root,);
    },)
    .filter(function nestedRegisteredRoot(root,): boolean {
      return (root !== sourceRoot) && pathWithin({
        candidate: root,
        parent: sourceRoot,
      },);
    },);
  /**
   * Git-selected ignored roots before private staging begins.
   */
  const gitSelectedRoots = await readIgnoredRoots({
    sourceRoot,
    gitPath,
    excludedRoots: nestedRegisteredRoots,
  },);
  /**
   * Private stage on destination filesystem for copy-on-write and local install.
   */
  const stageContainer = await mkdtemp(join(
    dirname(destinationRoot,),
    STAGE_PREFIX,
  ),);
  try {
    await chmod(
      stageContainer,
      PRIVATE_DIRECTORY_MODE,
    );
  }
  catch (error: unknown) {
    await rm(
      stageContainer,
      {
        recursive: true,
        force: true,
      },
    );
    throw error;
  }
  /**
   * Private payload root mirroring source repository paths.
   */
  const stageRoot = join(
    stageContainer,
    'payload',
  );
  await mkdir(
    stageRoot,
    { mode: PRIVATE_DIRECTORY_MODE, },
  );
  /**
   * Source subtrees omitted from recursive ignored roots.
   */
  const excludedSourceRoots = sourceExclusions({
    sourceRoot,
    registeredRoots,
    stageContainer,
  },);
  /**
   * Selected roots not themselves registered worktrees or private staging.
   */
  const selectedRoots = gitSelectedRoots.filter(function retainedRoot(
    repositoryPath,
  ): boolean {
    if (repositoryPath.split('/')
      .some(function privateStageComponent(component,): boolean {
        return component.startsWith(STAGE_PREFIX,);
      },)) {
      return false;
    }
    /**
     * Native selected source root.
     */
    const sourcePath = resolve(filesystemPath({
      root: sourceRoot,
      repositoryPath,
    },),);
    return !excludedSourceRoots.some(function excludesRoot(excludedRoot,): boolean {
      return pathWithin({
        candidate: sourcePath,
        parent: excludedRoot,
      },);
    },);
  },);

  try {
    /**
     * Initial exact source manifest before content copy.
     */
    const entries = await collectEntryManifest({
      root: sourceRoot,
      selectedRoots,
      excludedRoots: excludedSourceRoots,
    },);
    await copyManifestEntries({
      sourceRoot,
      stageRoot,
      entries,
    },);
    await applyEntryModes({
      root: stageRoot,
      entries,
    },);
    await assertFinalSourceEquivalence({
      sourceRoot,
      stageRoot,
      selectedRoots,
      excludedSourceRoots,
      stagedEntries: entries,
    },);
    return {
      entries,
      selectedRoots,
      sourceRoot,
      stageContainer,
      stageRoot,
    };
  }
  catch (error: unknown) {
    await rm(
      stageContainer,
      {
        recursive: true,
        force: true,
      },
    );
    if (error instanceof WorktreeCopyError)
      throw error;
    throw new WorktreeCopyError(
      `cli-git: could not stage ignored state from ${JSON.stringify(sourceRoot,)}.`,
      error,
    );
  }
}
