import {
  readFile,
  readdir,
  realpath,
} from 'node:fs/promises';
import {
  basename,
  isAbsolute,
  join,
  resolve,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import nanoSpawn, { SubprocessError, } from 'nano-spawn';

import { parseGlobalOptions, } from '../parse-global-options.ts';
import { WorktreeCopyError, } from './errors.ts';
import type {
  CreatedWorktree,
  WorktreeCopyObservation,
} from './model.ts';

/** Logger root for linked-worktree registration observation. */
const l = tagged({ tag: 'cli-git', },);

/** Git-file prefix introducing linked-worktree administrative path. */
const GITDIR_PREFIX = 'gitdir: ';

/** No effective repository existed before real Git ran. */
export const WORKTREE_COPY_NOT_APPLICABLE: unique symbol = Symbol(
  'worktree copy has no effective repository',
);

/**
 * Removes one Git-produced trailing line break.
 *
 * @param output - captured Git stdout
 *
 * @returns output without terminal LF or CRLF
 *
 * @example
 * ```ts
 * stripGitLine('/repo/.git\n');
 * // => '/repo/.git'
 * ```
 */
function stripGitLine(output: string,): string {
  if (output.endsWith('\r\n',))
    return output.slice(0, -2,);
  if (output.endsWith('\n',))
    return output.slice(0, -1,);
  return output;
}

/**
 * Reads linked-worktree administrative directory identities.
 *
 * @param adminRoot - common Git worktrees directory
 *
 * @returns directory basenames present at observation time
 *
 * @throws {@link WorktreeCopyError} when administration cannot be inspected
 *
 * @example
 * ```ts
 * await readAdminIds('/repo/.git/worktrees');
 * // => Set { 'topic' }
 * ```
 */
async function readAdminIds(adminRoot: string,): Promise<ReadonlySet<string>> {
  try {
    /** Directory entries beneath common worktree administration. */
    const entries = await readdir(adminRoot, { withFileTypes: true, },);
    return new Set(entries
      .filter(function isDirectory(entry,): boolean {
        return entry.isDirectory();
      },)
      .map(function entryName(entry,): string {
        return entry.name;
      },),);
  }
  catch (error: unknown) {
    if (Error.isError(error,)
      && ('code' in error)
      && (error.code === 'ENOENT')) {
      return new Set();
    }
    throw new WorktreeCopyError(
      `cli-git: could not inspect linked-worktree administration at ${JSON.stringify(adminRoot,)}.`,
      error,
    );
  }
}

/**
 * Runs read-only real-Git metadata command with captured streams.
 *
 * @param gitPath - absolute real-Git executable
 *
 * @param args - exact metadata argv
 *
 * @param cwd - command working directory
 *
 * @returns captured stdout
 *
 * @example
 * ```ts
 * await runMetadataGit({ gitPath: '/usr/bin/git', args: ['--version'], cwd: '/tmp' });
 * ```
 */
async function runMetadataGit({
  gitPath,
  args,
  cwd,
}: Readonly<{
  gitPath: string;
  args: readonly string[];
  cwd: string;
}>,): Promise<string> {
  /** Captured metadata subprocess result. */
  const result = await nanoSpawn(
    gitPath,
    args,
    {
      cwd,
      stderr: 'pipe',
      stdout: 'pipe',
    },
  );
  return result.stdout;
}

/**
 * Resolves current source worktree root or bare-repository absence.
 *
 * @param gitPath - absolute real-Git executable
 *
 * @param preSubcommandArgs - original Git global option region
 *
 * @param invocationCwd - wrapper process working directory
 *
 * @returns canonical worktree root, or undefined for bare repository
 *
 * @example
 * ```ts
 * await resolveSourceRoot({ gitPath: '/usr/bin/git', preSubcommandArgs: [], invocationCwd: '/repo' });
 * // => '/repo'
 * ```
 */
async function resolveSourceRoot({
  gitPath,
  preSubcommandArgs,
  invocationCwd,
}: Readonly<{
  gitPath: string;
  preSubcommandArgs: readonly string[];
  invocationCwd: string;
}>,): Promise<string | undefined> {
  try {
    /** Git-reported absolute worktree root. */
    const output = await runMetadataGit({
      gitPath,
      args: [
        ...preSubcommandArgs,
        'rev-parse',
        '--path-format=absolute',
        '--show-toplevel',
      ],
      cwd: invocationCwd,
    },);
    return await realpath(stripGitLine(output,),);
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError)
      return undefined;
    throw error;
  }
}

/**
 * Captures effective repository and linked-worktree identities before real Git.
 *
 * @param args - forwarded Git argv
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns repository observation or not-applicable sentinel
 *
 * @example
 * ```ts
 * await observeWorktreeRepository({ args: ['status'], gitPath: '/usr/bin/git' });
 * ```
 */
export async function observeWorktreeRepository({
  args,
  gitPath,
}: Readonly<{
  args: readonly string[];
  gitPath: string;
}>,): Promise<WorktreeCopyObservation | typeof WORKTREE_COPY_NOT_APPLICABLE> {
  /** Effective cwd and subcommand position for original invocation. */
  const {
    effectiveCwd,
    subcommandIndex,
  } = parseGlobalOptions(args,);
  /** Exact original global option region selecting effective repository. */
  const preSubcommandArgs = args.slice(0, subcommandIndex,);
  /** Wrapper process cwd from which Git applies original `-C` options. */
  const invocationCwd = process.cwd();
  /** Tagged observer logger. */
  const rl = tagged({ tag: observeWorktreeRepository.name, l, },);

  try {
    /** Git-reported absolute common directory. */
    const commonOutput = await runMetadataGit({
      gitPath,
      args: [
        ...preSubcommandArgs,
        'rev-parse',
        '--path-format=absolute',
        '--git-common-dir',
      ],
      cwd: invocationCwd,
    },);
    /** Canonical common directory anchoring administrative identity. */
    const commonDir = await realpath(stripGitLine(commonOutput,),);
    /** Common linked-worktree administrative root. */
    const adminRoot = join(commonDir, 'worktrees',);
    /** Existing linked-worktree identities. */
    const beforeAdminIds = await readAdminIds(adminRoot,);
    /** Canonical source root, absent for bare repositories. */
    const sourceRoot = await resolveSourceRoot({
      gitPath,
      preSubcommandArgs,
      invocationCwd,
    },);
    rl.debug(
      `captured ${String(beforeAdminIds.size,)} linked-worktree identities under ${commonDir}`,
    );
    return {
      adminRoot,
      beforeAdminIds,
      commonDir,
      effectiveCwd,
      sourceRoot,
    };
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError) {
      rl.debug('effective invocation has no repository; worktree copy observation is not applicable',);
      return WORKTREE_COPY_NOT_APPLICABLE;
    }
    throw error;
  }
}

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
 * @returns canonical admin directory basename, or undefined for main worktree
 *
 * @example
 * ```ts
 * await readLinkedAdminId('/worktrees/topic');
 * // => 'topic'
 * ```
 */
async function readLinkedAdminId(root: string,): Promise<string | undefined> {
  try {
    /** Linked-worktree `.git` pointer text. */
    const pointer = stripGitLine(await readFile(join(root, '.git',), 'utf8',),);
    if (!pointer.startsWith(GITDIR_PREFIX,))
      return undefined;
    /** Raw absolute or root-relative administrative path. */
    const rawAdminPath = pointer.slice(GITDIR_PREFIX.length,);
    /** Canonical linked-worktree administrative path. */
    const adminPath = await realpath(isAbsolute(rawAdminPath,)
      ? rawAdminPath
      : resolve(root, rawAdminPath,),);
    return basename(adminPath,);
  }
  catch (error: unknown) {
    if (Error.isError(error,)
      && ('code' in error)
      && ((error.code === 'EISDIR') || (error.code === 'ENOENT'))) {
      return undefined;
    }
    throw new WorktreeCopyError(
      `cli-git: could not resolve worktree identity for ${JSON.stringify(root,)}.`,
      error,
    );
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
  /** NUL-delimited stable worktree inventory. */
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
  /** Canonical roots still present on filesystem. */
  const roots = await Promise.all(parseWorktreeRoots(output,)
    .map(async function canonicalRoot(root,): Promise<string> {
      return realpath(root,);
    },),);
  /** Optional linked admin identities aligned with roots. */
  const adminIds = await Promise.all(roots.map(readLinkedAdminId,),);
  /** Linked-worktree identity map excluding main worktree. */
  const rootsByAdminId = new Map(adminIds.flatMap(function indexedRoot(
    adminId,
    index,
  ): readonly (readonly [string, string])[] {
    /** Root aligned with current admin identity. */
    const root = roots[index];
    return (adminId === undefined) || (root === undefined)
      ? []
      : [[adminId, root,],];
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
  /** Linked-worktree identities after real Git returned. */
  const afterAdminIds = await readAdminIds(observation.adminRoot,);
  /** Newly present administrative identities. */
  const createdAdminIds = [...afterAdminIds,]
    .filter(function wasAbsent(adminId,): boolean {
      return !observation.beforeAdminIds.has(adminId,);
    },)
    .sort();
  if (createdAdminIds.length === 0) {
    return {
      created: [],
      registeredRoots: [],
    };
  }
  /** Registered worktree paths after creation. */
  const registered = await readRegisteredWorktrees({ observation, gitPath, },);
  /** Created roots resolved from stable admin identities. */
  const created = createdAdminIds.map(function createdWorktree(adminId,): CreatedWorktree {
    /** Created root associated with new admin directory. */
    const root = registered.rootsByAdminId.get(adminId,);
    if (root === undefined) {
      throw new WorktreeCopyError(
        `cli-git: linked worktree ${JSON.stringify(adminId,)} was registered but its root could not be resolved.`,
      );
    }
    return { adminId, root, };
  },);
  return {
    created,
    registeredRoots: registered.roots,
  };
}
