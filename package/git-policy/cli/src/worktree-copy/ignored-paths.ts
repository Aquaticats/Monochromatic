import {
  isAbsolute,
  join,
} from 'node:path';

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
 * Reads every existing source path ignored by Git's standard exclusion stack.
 *
 * @param sourceRoot - canonical source worktree root
 *
 * @param gitPath - absolute real-Git executable
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
}: Readonly<{
  sourceRoot: string;
  gitPath: string;
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
  return collapseIgnoredRoots(paths,);
}
