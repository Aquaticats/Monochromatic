import {
  readFile,
  realpath,
} from 'node:fs/promises';
import {
  basename,
  isAbsolute,
  join,
  resolve,
} from 'node:path';

import {
  readAdminIds,
  runMetadataGit,
  stripGitLine,
} from './git-observer.ts';
import { WorktreeCopyError, } from './errors.ts';
import type {
  CreatedWorktree,
  WorktreeCopyObservation,
} from './model.ts';

/**
 * Git-file prefix introducing linked-worktree administrative path.
 */
const GITDIR_PREFIX = 'gitdir: ';

/**
 * Main worktree has no linked administrative identity.
 */
const MAIN_WORKTREE_ADMIN_ID: unique symbol = Symbol('main worktree has no linked admin identity',);

/**
 * Registered worktree root is currently absent from filesystem.
 */
const REGISTERED_ROOT_MISSING: unique symbol = Symbol('registered worktree root is absent',);

/**
 * Parses NUL-delimited porcelain output into canonical worktree roots.
 *
 * @param output - `git worktree list --porcelain -z` stdout
 *
 * @returns listed worktree path fields
 *
 * @example
 * ```ts
 * parseWorktreeRoots('worktree /repo\0HEAD abc\0\0');
 * // => ['/repo']
 * ```
 */
function parseWorktreeRoots(output: string,): readonly string[] {
  return output
    .split('\0',)
    .flatMap(function worktreeField(field,): readonly string[] {
      return field.startsWith('worktree ',)
        ? [field.slice('worktree '.length,),]
        : [];
    },);
}

/**
 * Resolves linked-worktree admin identity from root `.git` pointer.
 *
 * @param root - canonical linked-worktree root
 *
 * @returns canonical admin directory basename, or main-worktree sentinel
 *
 * @example
 * ```ts
 * await readLinkedAdminId('/worktrees/topic');
 * // => 'topic'
 * ```
 */
async function readLinkedAdminId(
  root: string,
): Promise<string | typeof MAIN_WORKTREE_ADMIN_ID> {
  try {
    /**
     * Linked-worktree `.git` pointer text.
     */
    const pointer = stripGitLine(await readFile(
      join(
        root,
        '.git',
      ),
      'utf8',
    ),);
    if (!pointer.startsWith(GITDIR_PREFIX,))
      return MAIN_WORKTREE_ADMIN_ID;
    /**
     * Raw absolute or root-relative administrative path.
     */
    const rawAdminPath = pointer.slice(GITDIR_PREFIX.length,);
    /**
     * Canonical linked-worktree administrative path.
     */
    const adminPath = await realpath(isAbsolute(rawAdminPath,)
      ? rawAdminPath
      : resolve(
        root,
        rawAdminPath,
      ),);
    return basename(adminPath,);
  }
  catch (error: unknown) {
    if (Error.isError(error,)
      && ('code' in error)
      && (error.code === 'EISDIR')) {
      return MAIN_WORKTREE_ADMIN_ID;
    }
    throw new WorktreeCopyError(
      `cli-git: could not resolve worktree identity for ${JSON.stringify(root,)}.`,
      error,
    );
  }
}

/**
 * Canonicalizes registered root while tolerating Git-retained missing worktrees.
 *
 * @param root - Git-reported registered worktree path
 *
 * @returns canonical root or missing-root sentinel
 *
 * @example
 * ```ts
 * await canonicalRegisteredRoot('/worktrees/topic');
 * ```
 */
async function canonicalRegisteredRoot(
  root: string,
): Promise<string | typeof REGISTERED_ROOT_MISSING> {
  try {
    return await realpath(root,);
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'ENOENT'))
      return REGISTERED_ROOT_MISSING;
    throw error;
  }
}

/**
 * Reads canonical registered worktree roots keyed by linked admin identity.
 *
 * @param observation - effective repository captured before forwarding
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns linked worktree roots and complete root exclusion set
 *
 * @example
 * ```ts
 * await readRegisteredWorktrees({ observation, gitPath: '/usr/bin/git' });
 * ```
 */
export async function readRegisteredWorktrees({
  observation,
  gitPath,
}: Readonly<{
  observation: WorktreeCopyObservation;
  gitPath: string;
}>,): Promise<Readonly<{
  rootsByAdminId: ReadonlyMap<string, string>;
  roots: readonly string[];
}>> {
  /**
   * NUL-delimited stable worktree inventory.
   */
  const output = await runMetadataGit({
    gitPath,
    args: [
      '--git-dir',
      observation.commonDir,
      'worktree',
      'list',
      '--porcelain',
      '-z',
    ],
    cwd: observation.effectiveCwd,
  },);
  /**
   * Canonical roots still present on filesystem.
   */
  const rootResults = await Promise.all(parseWorktreeRoots(output,)
    .map(canonicalRegisteredRoot,),);
  /**
   * Existing canonical roots, omitting Git-retained missing registrations.
   */
  const roots = rootResults.filter(function existingRoot(
    root,
  ): root is string {
    return (typeof root) === 'string';
  },);
  /**
   * Optional linked admin identities aligned with existing roots.
   */
  const adminIds = await Promise.all(roots.map(function linkedAdminId(
    root,
  ): Promise<string | typeof MAIN_WORKTREE_ADMIN_ID> {
    return (observation.sourceRoot === undefined) && (root === observation.commonDir)
      ? Promise.resolve(MAIN_WORKTREE_ADMIN_ID,)
      : readLinkedAdminId(root,);
  },),);
  /**
   * Linked-worktree identity map excluding main worktree.
   */
  const rootsByAdminId = new Map(adminIds.flatMap(function indexedRoot(
    adminId,
    index,
  ): readonly (readonly [
    string,
    string
  ])[] {
    /**
     * Root aligned with current admin identity.
     */
    const root = roots[index];
    return ((typeof adminId) === 'symbol') || (root === undefined)
      ? []
      : [[
        adminId,
        root,
      ],];
  },),);
  return {
    rootsByAdminId,
    roots,
  };
}

/**
 * Finds worktrees newly registered since initial observation.
 *
 * @param observation - effective repository captured before real Git
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns created worktrees and complete registered root set
 *
 * @example
 * ```ts
 * await findCreatedWorktrees({ observation, gitPath: '/usr/bin/git' });
 * ```
 */
export async function findCreatedWorktrees({
  observation,
  gitPath,
}: Readonly<{
  observation: WorktreeCopyObservation;
  gitPath: string;
}>,): Promise<Readonly<{
  created: readonly CreatedWorktree[];
  registeredRoots: readonly string[];
}>> {
  /**
   * Linked-worktree identities after real Git returned.
   */
  const afterAdminIds = await readAdminIds(observation.adminRoot,);
  /**
   * Newly present administrative identities.
   */
  const createdAdminIds = [...afterAdminIds,]
    .filter(function wasAbsent(adminId,): boolean {
      return !observation.beforeAdminIds
        .has(adminId,);
    },)
    .toSorted();
  if (createdAdminIds.length === 0) {
    return {
      created: [],
      registeredRoots: [],
    };
  }
  /**
   * Registered worktree paths after creation.
   */
  const registered = await readRegisteredWorktrees({
    observation,
    gitPath,
  },);
  /**
   * Created roots resolved from stable admin identities.
   */
  const created = createdAdminIds.map(function createdWorktree(adminId,): CreatedWorktree {
    /**
     * Created root associated with new admin directory.
     */
    const root = registered.rootsByAdminId
      .get(adminId,);
    if (root === undefined) {
      throw new WorktreeCopyError(
        `cli-git: linked worktree ${JSON.stringify(adminId,)} was registered but its root could not be resolved.`,
      );
    }
    return {
      adminId,
      root,
    };
  },);
  return {
    created,
    registeredRoots: registered.roots,
  };
}
