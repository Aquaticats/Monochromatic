import { constants, } from 'node:fs';
import {
  cp,
  mkdir,
  mkdtemp,
  rm,
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
import type { StagedWorktreeSnapshot, } from './model.ts';

/** Private staging directory prefix beside destination worktree. */
const STAGE_PREFIX = '.cli-git-worktree-copy-';

/** Private staging directory mode. */
const PRIVATE_DIRECTORY_MODE = 0o700;

/** Copy-on-write request with documented full-copy fallback. */
const COPY_MODE = constants.COPYFILE_FICLONE;

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
  /** Native relative path from parent. */
  const local = relative(parent, candidate,);
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
  return [...registeredRoots, stageContainer,]
    .map(function normalizeRoot(root,): string {
      return resolve(root,);
    },)
    .filter(function nestedRoot(root,): boolean {
      return (root !== sourceRoot) && pathWithin({ candidate: root, parent: sourceRoot, },);
    },);
}

/**
 * Copies selected roots into private destination-filesystem stage.
 *
 * @param sourceRoot - canonical source worktree
 *
 * @param stageRoot - private payload root
 *
 * @param selectedRoots - Git-selected ignored roots
 *
 * @param excludedSourceRoots - nested registered worktrees and stage
 *
 * @returns nothing after private copy completes
 *
 * @example
 * ```ts
 * await copySelectedRoots({ sourceRoot: '/repo', stageRoot: '/stage', selectedRoots: ['cache'], excludedSourceRoots: [] });
 * ```
 */
async function copySelectedRoots({
  sourceRoot,
  stageRoot,
  selectedRoots,
  excludedSourceRoots,
}: Readonly<{
  sourceRoot: string;
  stageRoot: string;
  selectedRoots: readonly string[];
  excludedSourceRoots: readonly string[];
}>,): Promise<void> {
  for (const repositoryPath of selectedRoots) {
    /** Source selected root path. */
    const sourcePath = filesystemPath({ root: sourceRoot, repositoryPath, },);
    /** Staged selected root path. */
    const stagePath = filesystemPath({ root: stageRoot, repositoryPath, },);
    // oxlint-disable-next-line no-await-in-loop -- selected roots may share ancestors and must materialize deterministically
    await mkdir(dirname(stagePath,), {
      recursive: true,
      mode: PRIVATE_DIRECTORY_MODE,
    },);
    // oxlint-disable-next-line no-await-in-loop -- deterministic staging avoids overlapping root writes
    await cp(
      sourcePath,
      stagePath,
      {
        recursive: true,
        force: false,
        errorOnExist: true,
        dereference: false,
        verbatimSymlinks: true,
        preserveTimestamps: false,
        mode: COPY_MODE,
        filter(source,): boolean {
          /** Normalized source candidate supplied by Node copy traversal. */
          const normalizedSource = resolve(source,);
          return !excludedSourceRoots.some(function excludesSource(excludedRoot,): boolean {
            return pathWithin({ candidate: normalizedSource, parent: excludedRoot, },);
          },);
        },
      },
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
  /** Git-selected ignored roots before private staging begins. */
  const selectedRoots = await readIgnoredRoots({ sourceRoot, gitPath, },);
  /** Private stage on destination filesystem for copy-on-write and local install. */
  const stageContainer = await mkdtemp(join(
    dirname(destinationRoot,),
    STAGE_PREFIX,
  ),);
  /** Private payload root mirroring source repository paths. */
  const stageRoot = join(stageContainer, 'payload',);
  await mkdir(stageRoot, { mode: PRIVATE_DIRECTORY_MODE, },);
  /** Source subtrees omitted from recursive ignored roots. */
  const excludedSourceRoots = sourceExclusions({
    sourceRoot,
    registeredRoots,
    stageContainer,
  },);

  try {
    /** Initial exact source manifest before content copy. */
    const entries = await collectEntryManifest({
      root: sourceRoot,
      selectedRoots,
      excludedRoots: excludedSourceRoots,
    },);
    await copySelectedRoots({
      sourceRoot,
      stageRoot,
      selectedRoots,
      excludedSourceRoots,
    },);
    await applyEntryModes({ root: stageRoot, entries, },);
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
    await rm(stageContainer, { recursive: true, force: true, },);
    if (error instanceof WorktreeCopyError)
      throw error;
    throw new WorktreeCopyError(
      `cli-git: could not stage ignored state from ${JSON.stringify(sourceRoot,)}.`,
      error,
    );
  }
}
