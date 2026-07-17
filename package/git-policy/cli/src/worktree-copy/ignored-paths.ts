import { readdir, } from 'node:fs/promises';
import {
  isAbsolute,
  join,
  relative,
  resolve,
} from 'node:path';

import { SubprocessError, } from 'nano-spawn';

import { WorktreeCopyError, } from './errors.ts';
import { runMetadataGit, } from './git-observer.ts';

/**
 * Git directory marker appended by `ls-files --directory`.
 */
const DIRECTORY_SUFFIX = '/';

/**
 * Reports component-aware repository-path containment.
 *
 * @param candidate - candidate repository path
 *
 * @param parent - possible ancestor repository path
 *
 * @returns whether candidate equals or descends from parent
 *
 * @example
 * ```ts
 * isRepositoryPathWithin({ candidate: 'a/b', parent: 'a' });
 * // => true
 * ```
 */
export function isRepositoryPathWithin({
  candidate,
  parent,
}: Readonly<{
  candidate: string;
  parent: string;
}>,): boolean {
  return (candidate === parent) || candidate.startsWith(`${parent}/`,);
}

/**
 * Validates Git-produced repository path before filesystem interpolation.
 *
 * @param repositoryPath - slash-separated Git path
 *
 * @throws {@link WorktreeCopyError} when path could escape worktree root
 *
 * @example
 * ```ts
 * assertSafeRepositoryPath('node_modules/pkg');
 * ```
 */
export function assertSafeRepositoryPath(repositoryPath: string,): void {
  /**
   * Slash-delimited path components.
   */
  const components = repositoryPath.split('/',);
  if ((repositoryPath === '')
    || isAbsolute(repositoryPath,)
    || components.some(function unsafeComponent(component,): boolean {
      return (component === '') || (component === '.')
        || (component === '..');
    },)) {
    throw new WorktreeCopyError(
      `cli-git: Git returned unsafe ignored path ${JSON.stringify(repositoryPath,)}.`,
    );
  }
}

/**
 * Resolves safe repository path beneath filesystem root.
 *
 * @param root - absolute worktree or staging root
 *
 * @param repositoryPath - validated slash-separated repository path
 *
 * @returns native filesystem path
 *
 * @example
 * ```ts
 * filesystemPath({ root: '/repo', repositoryPath: 'a/b' });
 * // => '/repo/a/b'
 * ```
 */
export function filesystemPath({
  root,
  repositoryPath,
}: Readonly<{
  root: string;
  repositoryPath: string;
}>,): string {
  assertSafeRepositoryPath(repositoryPath,);
  return join(
    root,
    ...repositoryPath.split('/',),
  );
}

/**
 * Removes nested duplicate roots after Git ignore enumeration.
 *
 * @param paths - safe ignored repository paths
 *
 * @returns sorted roots without descendants of another selected root
 *
 * @example
 * ```ts
 * collapseIgnoredRoots(['cache/a', 'cache']);
 * // => ['cache']
 * ```
 */
function collapseIgnoredRoots(paths: readonly string[],): readonly string[] {
  /**
   * Sorted unique candidate paths.
   */
  const candidates = [...new Set(paths,),]
    .toSorted();
  /**
   * Retained roots without descendants.
   */
  const roots: string[] = [];
  for (const candidate of candidates) {
    if (!roots.some(function ownsCandidate(root,): boolean {
      return isRepositoryPathWithin({
        candidate,
        parent: root,
      },);
    },)) {
      roots.push(candidate,);
    }
  }
  return roots;
}

/**
 * Reports native path containment without following filesystem entries.
 *
 * @param candidate - absolute candidate path
 *
 * @param parent - absolute possible ancestor path
 *
 * @returns whether candidate equals or descends from parent
 *
 * @example
 * ```ts
 * isFilesystemPathWithin({ candidate: '/repo/cache', parent: '/repo' });
 * // => true
 * ```
 */
function isFilesystemPathWithin({
  candidate,
  parent,
}: Readonly<{
  candidate: string;
  parent: string;
}>,): boolean {
  /**
   * Native path from parent to candidate.
   */
  const local = relative(
    parent,
    candidate,
  );
  return (local === '') || ((!local.startsWith('..',)) && (!isAbsolute(local,)));
}

/**
 * Reports whether one special source path is ignored by standard Git stack.
 *
 * @param sourceRoot - canonical source worktree root
 *
 * @param repositoryPath - safe special filesystem path
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns whether Git classifies path as ignored
 *
 * @example
 * ```ts
 * await isIgnoredSpecialPath({ sourceRoot: '/repo', repositoryPath: 'pipe', gitPath: '/usr/bin/git' });
 * ```
 */
async function isIgnoredSpecialPath({
  sourceRoot,
  repositoryPath,
  gitPath,
}: Readonly<{
  sourceRoot: string;
  repositoryPath: string;
  gitPath: string;
}>,): Promise<boolean> {
  try {
    await runMetadataGit({
      gitPath,
      args: [
        '-C',
        sourceRoot,
        'check-ignore',
        '--quiet',
        '--no-index',
        '--',
        repositoryPath,
      ],
      cwd: sourceRoot,
    },);
    return true;
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError && (error.exitCode === 1))
      return false;
    throw error;
  }
}

/**
 * Discovers special entries omitted by Git's index-oriented ls-files walk.
 *
 * @param sourceRoot - canonical source worktree root
 *
 * @param selectedRoots - ignored roots already returned by Git
 *
 * @param excludedRoots - registered nested worktree roots
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns ignored socket, FIFO, and device repository paths
 *
 * @example
 * ```ts
 * await readIgnoredSpecialPaths({ sourceRoot: '/repo', selectedRoots: [], excludedRoots: [], gitPath: '/usr/bin/git' });
 * ```
 */
async function readIgnoredSpecialPaths({
  sourceRoot,
  selectedRoots,
  excludedRoots,
  gitPath,
}: Readonly<{
  sourceRoot: string;
  selectedRoots: readonly string[];
  excludedRoots: readonly string[];
  gitPath: string;
}>,): Promise<readonly string[]> {
  /**
   * Pending repository directories outside already-selected ignored trees.
   */
  const pending: string[] = ['.',];
  /**
   * Special repository paths requiring direct Git ignore classification.
   */
  const specialPaths: string[] = [];
  while (pending.length > 0) {
    /**
     * Current repository directory marker.
     */
    const current = pending.pop();
    if (current === undefined)
      throw new WorktreeCopyError('cli-git: special-entry walk lost pending directory.',);
    /**
     * Current native source directory.
     */
    const directoryPath = current === '.'
      ? sourceRoot
      : filesystemPath({ root: sourceRoot, repositoryPath: current, },);
    // oxlint-disable-next-line no-await-in-loop -- structural filesystem walk remains sequential and no-follow
    const entries = await readdir(
      directoryPath,
      { withFileTypes: true, },
    );
    for (const entry of entries) {
      /**
       * Child repository path.
       */
      const repositoryPath = current === '.'
        ? entry.name
        : `${current}/${entry.name}`;
      if (repositoryPath === '.git')
        continue;
      /**
       * Child native source path.
       */
      const sourcePath = resolve(filesystemPath({
        root: sourceRoot,
        repositoryPath,
      },),);
      if (excludedRoots.some(function excludedRoot(root,): boolean {
        return isFilesystemPathWithin({
          candidate: sourcePath,
          parent: root,
        },);
      },)) {
        continue;
      }
      if (selectedRoots.some(function selectedAncestor(root,): boolean {
        return isRepositoryPathWithin({
          candidate: repositoryPath,
          parent: root,
        },);
      },)) {
        continue;
      }
      if (entry.isDirectory()) {
        pending.push(repositoryPath,);
        continue;
      }
      if ((!entry.isFile()) && (!entry.isSymbolicLink()))
        specialPaths.push(repositoryPath,);
    }
  }
  /**
   * Ignore decisions aligned with discovered special paths.
   */
  const ignored = await Promise.all(specialPaths.map(function ignoredSpecialPath(
    repositoryPath,
  ): Promise<boolean> {
    return isIgnoredSpecialPath({
      sourceRoot,
      repositoryPath,
      gitPath,
    },);
  },),);
  return specialPaths.filter(function ignoredSpecial(
    _repositoryPath,
    index,
  ): boolean {
    return ignored[index] === true;
  },);
}

/**
 * Reads every existing source path ignored by Git's standard exclusion stack.
 *
 * @param sourceRoot - canonical source worktree root
 *
 * @param gitPath - absolute real-Git executable
 *
 * @param excludedRoots - registered roots excluded from special-entry walk
 *
 * @returns collapsed ignored roots in repository path form
 *
 * @example
 * ```ts
 * await readIgnoredRoots({ sourceRoot: '/repo', gitPath: '/usr/bin/git' });
 * // => ['node_modules']
 * ```
 */
export async function readIgnoredRoots({
  sourceRoot,
  gitPath,
  excludedRoots = [],
}: Readonly<{
  sourceRoot: string;
  gitPath: string;
  excludedRoots?: readonly string[];
}>,): Promise<readonly string[]> {
  /**
   * NUL-delimited ignored untracked path output.
   */
  const output = await runMetadataGit({
    gitPath,
    args: [
      '-C',
      sourceRoot,
      'ls-files',
      '--others',
      '--ignored',
      '--exclude-standard',
      '--directory',
      '-z',
    ],
    cwd: sourceRoot,
  },);
  /**
   * Safe ignored roots with Git directory markers removed.
   */
  const paths = output
    .split('\0',)
    .filter(function nonempty(path,): boolean {
      return path !== '';
    },)
    .map(function removeDirectoryMarker(path,): string {
      return path.endsWith(DIRECTORY_SUFFIX,)
        ? path.slice(
          0,
          -DIRECTORY_SUFFIX.length,
        )
        : path;
    },);
  paths.forEach(assertSafeRepositoryPath,);
  /**
   * Ignored special paths omitted by Git's index-oriented listing.
   */
  const specialPaths = await readIgnoredSpecialPaths({
    sourceRoot,
    selectedRoots: paths,
    excludedRoots,
    gitPath,
  },);
  return collapseIgnoredRoots([
    ...paths,
    ...specialPaths,
  ],);
}
