import type { Dirent, } from 'node:fs';
import {
  readdir,
  realpath,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import nanoSpawn, { SubprocessError, } from 'nano-spawn';

import { parseGlobalOptions, } from '../parse-global-options.ts';
import { WorktreeCopyError, } from './errors.ts';
import {
  BARE_REPOSITORY_SOURCE,
  type WorktreeCopyObservation,
} from './model.ts';

/**
 * Logger root for linked-worktree registration observation.
 */
const l = tagged({ tag: 'cli-git', },);

/**
 * No effective repository existed before real Git ran.
 */
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
export function stripGitLine(output: string,): string {
  if (output.endsWith('\r\n',))
    return output.slice(
      0,
      -2,
    );
  if (output.endsWith('\n',))
    return output.slice(
      0,
      -1,
    );
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
export async function readAdminIds(adminRoot: string,): Promise<ReadonlySet<string>> {
  try {
    /**
     * Directory entries beneath common worktree administration.
     */
    const entries = await readdir(
      adminRoot,
      { withFileTypes: true, },
    );
    return new Set(entries
      .filter(function isDirectory(entry: Readonly<Dirent>,): boolean {
        return entry.isDirectory();
      },)
      .map(function entryName(entry: Readonly<Dirent>,): string {
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
export async function runMetadataGit({
  gitPath,
  args,
  cwd,
}: Readonly<{
  gitPath: string;
  args: readonly string[];
  cwd: string;
}>,): Promise<string> {
  /**
   * Captured metadata subprocess result.
   */
  const result = await nanoSpawn(
    gitPath,
    [...args,],
    {
      cwd,
      stderr: 'pipe',
      stdout: 'pipe',
    },
  );
  return result.stdout;
}

/**
 * Resolves common directory or repository absence from original global options.
 *
 * @param gitPath - absolute real-Git executable
 *
 * @param preSubcommandArgs - original Git global option region
 *
 * @param invocationCwd - wrapper process working directory
 *
 * @returns canonical common directory, or not-applicable sentinel
 *
 * @example
 * ```ts
 * await resolveCommonDir({ gitPath: '/usr/bin/git', preSubcommandArgs: [], invocationCwd: '/repo' });
 * // => '/repo/.git'
 * ```
 */
async function resolveCommonDir({
  gitPath,
  preSubcommandArgs,
  invocationCwd,
}: Readonly<{
  gitPath: string;
  preSubcommandArgs: readonly string[];
  invocationCwd: string;
}>,): Promise<string | typeof WORKTREE_COPY_NOT_APPLICABLE> {
  try {
    /**
     * Git-reported absolute common directory.
     */
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
    /**
     * Git-reported common path without line terminator.
     */
    const commonPath = stripGitLine(commonOutput,);
    if (commonPath === '')
      return WORKTREE_COPY_NOT_APPLICABLE;
    return await realpath(commonPath,);
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError
      || (Error.isError(error,) && ('code' in error) && (error.code === 'ENOENT')))
      return WORKTREE_COPY_NOT_APPLICABLE;
    throw error;
  }
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
 * @returns canonical worktree root, or bare-repository sentinel
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
}>,): Promise<string | typeof BARE_REPOSITORY_SOURCE> {
  /**
   * Git-reported bare-repository state.
   */
  const bareOutput = await runMetadataGit({
    gitPath,
    args: [
      ...preSubcommandArgs,
      'rev-parse',
      '--is-bare-repository',
    ],
    cwd: invocationCwd,
  },);
  if (stripGitLine(bareOutput,) === 'true')
    return BARE_REPOSITORY_SOURCE;
  /**
   * Git-reported absolute worktree root.
   */
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
  return realpath(stripGitLine(output,),);
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
  /**
   * Effective cwd and subcommand position for original invocation.
   */
  const {
    effectiveCwd,
    subcommandIndex,
  } = parseGlobalOptions(args,);
  /**
   * Exact original global option region selecting effective repository.
   */
  const preSubcommandArgs = args.slice(
    0,
    subcommandIndex,
  );
  /**
   * Wrapper process cwd from which Git applies original `-C` options.
   */
  const invocationCwd = process.cwd();
  /**
   * Tagged observer logger.
   */
  const rl = tagged({
    tag: observeWorktreeRepository.name,
    l,
  },);

  /**
   * Canonical common directory anchoring administrative identity.
   */
  const commonDir = await resolveCommonDir({
    gitPath,
    preSubcommandArgs,
    invocationCwd,
  },);
  if ((typeof commonDir) === 'symbol') {
    rl.debug('effective invocation has no repository; worktree copy observation is not applicable',);
    return WORKTREE_COPY_NOT_APPLICABLE;
  }
  /**
   * Common linked-worktree administrative root.
   */
  const adminRoot = join(
    commonDir,
    'worktrees',
  );
  /**
   * Existing linked-worktree identities.
   */
  const beforeAdminIds = await readAdminIds(adminRoot,);
  /**
   * Canonical source root or bare-repository sentinel.
   */
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
    ...((typeof sourceRoot) === 'symbol'
      ? {}
      : { sourceRoot, }),
  };
}
