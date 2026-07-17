import type { Stats, } from 'node:fs';
import {
  chmod,
  lstat,
  readdir,
} from 'node:fs/promises';
import {
  isAbsolute,
  relative,
  resolve,
} from 'node:path';

import { WorktreeCopyError, } from './errors.ts';
import { filesystemPath, } from './ignored-paths.ts';
import type {
  WorktreeCopyEntry,
  WorktreeCopyEntryKind,
} from './model.ts';

/**
 * Portable permission and special-mode bit mask retained by copy contract.
 */
const PERMISSION_BITS = 0o7777;

/**
 * Pending structural filesystem walk item.
 *
 * @example
 * ```ts
 * const item: WalkItem = { filesystemPath: '/repo/cache', relativePath: 'cache' };
 * ```
 */
type WalkItem = Readonly<{
  /**
   * Absolute no-follow path under walked root.
   */
  filesystemPath: string;
  /**
   * Repository path retained in manifest.
   */
  relativePath: string;
}>;

/**
 * Reports component-aware native path containment.
 *
 * @param candidate - absolute candidate path
 *
 * @param parent - absolute possible ancestor path
 *
 * @returns whether candidate equals or descends from parent
 *
 * @example
 * ```ts
 * isFilesystemPathWithin({ candidate: '/repo/a', parent: '/repo' });
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
 * Classifies supported no-follow filesystem metadata.
 *
 * @param stats - lstat result
 *
 * @param path - diagnostic filesystem path
 *
 * @returns supported copy entry kind
 *
 * @throws {@link WorktreeCopyError} for sockets, FIFOs, and device nodes
 *
 * @example
 * ```ts
 * entryKind({ stats, path: '/repo/file' });
 * // => 'file'
 * ```
 */
function entryKind({
  stats,
  path,
}: Readonly<{
  stats: Readonly<Stats>;
  path: string;
}>,): WorktreeCopyEntryKind {
  if (stats.isDirectory())
    return 'directory';
  if (stats.isFile())
    return 'file';
  if (stats.isSymbolicLink())
    return 'symlink';
  throw new WorktreeCopyError(
    `cli-git: ignored path ${JSON.stringify(path,)} has unsupported filesystem type.`,
  );
}

/**
 * Collects deterministic no-follow manifest for selected ignored roots.
 *
 * @param root - source or staging filesystem root
 *
 * @param selectedRoots - selected repository paths
 *
 * @param excludedRoots - absolute directory roots omitted with descendants
 *
 * @returns parent-before-child manifest
 *
 * @example
 * ```ts
 * await collectEntryManifest({ root: '/repo', selectedRoots: ['cache'], excludedRoots: [] });
 * ```
 */
export async function collectEntryManifest({
  root,
  selectedRoots,
  excludedRoots,
}: Readonly<{
  root: string;
  selectedRoots: readonly string[];
  excludedRoots: readonly string[];
}>,): Promise<readonly WorktreeCopyEntry[]> {
  /**
   * Pending work stack, reversed so lexical first path is visited first.
   */
  const pending: WalkItem[] = selectedRoots
    .toReversed()
    .map(function rootItem(relativePath,): WalkItem {
      return {
        filesystemPath: filesystemPath({
          root,
          repositoryPath: relativePath,
        },),
        relativePath,
      };
    },);
  /**
   * Deterministic entries collected parent before child.
   */
  const entries: WorktreeCopyEntry[] = [];

  while (pending.length > 0) {
    /**
     * Current structural walk item.
     */
    const item = pending.pop();
    if (item === undefined)
      throw new WorktreeCopyError('cli-git: ignored-state walk lost pending entry.',);
    /**
     * Lexically normalized current path for exclusion comparison.
     */
    const normalizedPath = resolve(item.filesystemPath,);
    if (excludedRoots.some(function excludesPath(excludedRoot,): boolean {
      return isFilesystemPathWithin({
        candidate: normalizedPath,
        parent: excludedRoot,
      },);
    },)) {
      continue;
    }
    /**
     * No-follow source metadata.
     */
    // oxlint-disable-next-line no-await-in-loop -- structural walk follows one bounded filesystem entry at a time
    const stats = await lstat(item.filesystemPath,);
    /**
     * Supported structural entry kind.
     */
    const kind = entryKind({
      stats,
      path: item.filesystemPath,
    },);
    entries.push({
      kind,
      mode: stats.mode & PERMISSION_BITS,
      relativePath: item.relativePath,
    },);
    if (kind !== 'directory')
      continue;
    /**
     * Lexically ordered immediate child names.
     */
    // oxlint-disable-next-line no-await-in-loop -- directory children are discovered only after no-follow type classification
    const childNames = (await readdir(item.filesystemPath,))
      .toSorted();
    childNames
      .toReversed()
      .forEach(function pushChild(childName,): void {
        /**
         * Child repository path preserving Git slash separator.
         */
        const childRelativePath = `${item.relativePath}/${childName}`;
        pending.push({
          filesystemPath: filesystemPath({
            root,
            repositoryPath: childRelativePath,
          },),
          relativePath: childRelativePath,
        },);
      },);
  }
  return entries;
}

/**
 * Applies retained permission bits to staged files and directories.
 *
 * Directories are changed child-first after file modes so restrictive parent
 * modes cannot prevent finishing descendant setup.
 *
 * @param root - staging payload root
 *
 * @param entries - staged manifest carrying source modes
 *
 * @example
 * ```ts
 * await applyEntryModes({ root: '/stage', entries });
 * ```
 */
export async function applyEntryModes({
  root,
  entries,
}: Readonly<{
  root: string;
  entries: readonly WorktreeCopyEntry[];
}>,): Promise<void> {
  /**
   * Files whose modes can be applied independently.
   */
  const files = entries.filter(function isFile(entry,): boolean {
    return entry.kind === 'file';
  },);
  /**
   * Directories applied deepest first.
   */
  const directories = entries
    .filter(function isDirectory(entry,): boolean {
      return entry.kind === 'directory';
    },)
    .toReversed();
  for (const entry of [
    ...files,
    ...directories,
  ]) {
    // oxlint-disable-next-line no-await-in-loop -- mode order preserves access through restrictive parent directories
    await chmod(
      filesystemPath({
        root,
        repositoryPath: entry.relativePath,
      },),
      entry.mode,
    );
  }
}
