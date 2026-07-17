import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import nanoSpawn, {
  type Result,
  SubprocessError,
} from 'nano-spawn';

/**
 * Absolute real-Git executable used only for disposable fixture setup.
 */
const REAL_GIT_PATH = '/usr/bin/git';

/**
 * Built cli-git artifact exercised at consumer boundary.
 */
export const WRAPPER_PATH = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
  'node',
  'index.mjs',
);

/**
 * Executable fixture mode.
 */
const EXECUTABLE_MODE = 0o755;

/**
 * Portable permission-bit mask.
 */
const PERMISSION_BITS = 0o7777;

/**
 * Fixture author identity.
 */
const TEST_USER_NAME = 'cli-git worktree copy test';

/**
 * Fixture author address.
 */
const TEST_USER_EMAIL = 'worktree-copy@example.invalid';

/**
 * Disposable test directory with automatic recursive cleanup.
 */
type TempDirectory = Readonly<{
  /**
   * Absolute temporary root.
   */
  path: string;
  /**
   * Removes complete fixture after test settles.
   */
  [Symbol.asyncDispose]: () => Promise<void>;
}>;

/**
 * Captured wrapper success or subprocess failure.
 */
type WrapperOutcome = Readonly<{
  /**
   * Outcome discriminant.
   */
  kind: 'success';
  /**
   * Successful captured process result.
   */
  result: Result;
}> | Readonly<{
  /**
   * Outcome discriminant.
   */
  kind: 'failure';
  /**
   * Failed captured process result.
   */
  error: SubprocessError;
}>;

/**
 * Restores owner access on fixture directories before recursive cleanup.
 *
 * @param root - disposable fixture root
 *
 * @example
 * ```ts
 * await prepareFixtureCleanup('/tmp/fixture');
 * ```
 */
async function prepareFixtureCleanup(root: string,): Promise<void> {
  /**
   * Pending fixture directories.
   */
  const pending: string[] = [root,];
  while (pending.length > 0) {
    /**
     * Current fixture directory.
     */
    const directory = pending.pop();
    if (directory === undefined)
      throw new Error('Fixture cleanup lost pending directory.',);
    // oxlint-disable-next-line no-await-in-loop -- cleanup restores each bounded fixture directory before traversal
    await chmod(
      directory,
      0o700,
    );
    // oxlint-disable-next-line no-await-in-loop -- directory traversal follows restored owner mode
    const entries = await readdir(
      directory,
      { withFileTypes: true, },
    );
    entries.filter(function childDirectory(entry,): boolean {
      return entry.isDirectory();
    },)
      .forEach(function queueDirectory(entry,): void {
        pending.push(join(
          directory,
          entry.name,
        ),);
      },);
  }
}

/**
 * Creates one disposable filesystem root.
 *
 * @returns asynchronously disposable temporary directory
 *
 * @example
 * ```ts
 * await using fixture = await createTempDirectory();
 * ```
 */
export async function createTempDirectory(): Promise<TempDirectory> {
  /**
   * Unique fixture root.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    'cli-git-worktree-copy-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await prepareFixtureCleanup(path,);
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 * Runs real Git without wrapper behavior.
 *
 * @param cwd - subprocess working directory
 *
 * @param args - exact real-Git arguments
 *
 * @returns captured successful result
 *
 * @example
 * ```ts
 * await runRealGit({ cwd: '/repo', args: ['status', '--short'] });
 * ```
 */
export function runRealGit({
  cwd,
  args,
}: Readonly<{
  cwd: string;
  args: readonly string[];
}>,): Promise<Result> {
  return nanoSpawn(
    REAL_GIT_PATH,
    [...args,],
    { cwd, },
  );
}

/**
 * Runs built shadow Git entry through Node.
 *
 * @param cwd - subprocess working directory
 *
 * @param args - exact wrapper arguments
 *
 * @returns captured successful result
 *
 * @example
 * ```ts
 * await runWrapper({ cwd: '/repo', args: ['status', '--short'] });
 * ```
 */
function runWrapper({
  cwd,
  args,
}: Readonly<{
  cwd: string;
  args: readonly string[];
}>,): Promise<Result> {
  return nanoSpawn(
    'node',
    [
      WRAPPER_PATH,
      ...args,
    ],
    { cwd, },
  );
}

/**
 * Captures built wrapper success or subprocess failure without mutable catch state.
 *
 * @param cwd - subprocess working directory
 *
 * @param args - exact wrapper arguments
 *
 * @returns discriminated captured outcome
 *
 * @example
 * ```ts
 * await captureWrapper({ cwd: '/repo', args: ['status'] });
 * ```
 */
export async function captureWrapper({
  cwd,
  args,
}: Readonly<{
  cwd: string;
  args: readonly string[];
}>,): Promise<WrapperOutcome> {
  try {
    return {
      kind: 'success',
      result: await runWrapper({
        cwd,
        args,
      },),
    };
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError) {
      return {
        kind: 'failure',
        error,
      };
    }
    throw error;
  }
}

/**
 * Requires successful captured wrapper result.
 *
 * @param outcome - captured wrapper outcome
 *
 * @returns successful process result
 *
 * @throws when wrapper failed
 *
 * @example
 * ```ts
 * requireSuccess(await captureWrapper(options));
 * ```
 */
export function requireSuccess(outcome: WrapperOutcome,): Result {
  if (outcome.kind === 'failure')
    throw outcome.error;
  return outcome.result;
}

/**
 * Requires failed captured wrapper result.
 *
 * @param outcome - captured wrapper outcome
 *
 * @returns subprocess failure
 *
 * @throws when wrapper succeeded
 *
 * @example
 * ```ts
 * requireFailure(await captureWrapper(options));
 * ```
 */
export function requireFailure(outcome: WrapperOutcome,): SubprocessError {
  if (outcome.kind === 'success')
    throw new Error('Expected built cli-git wrapper to fail.',);
  return outcome.error;
}

/**
 * Initializes disposable repository with one committed tracked file.
 *
 * @param repositoryRoot - empty target directory
 *
 * @example
 * ```ts
 * await initializeRepository('/tmp/repo');
 * ```
 */
export async function initializeRepository(repositoryRoot: string,): Promise<void> {
  await mkdir(
    repositoryRoot,
    { recursive: true, },
  );
  await runRealGit({
    cwd: repositoryRoot,
    args: [
      'init',
      '--initial-branch=main',
    ],
  },);
  await Promise.all([
    runRealGit({
      cwd: repositoryRoot,
      args: [
        'config',
        'user.name',
        TEST_USER_NAME,
      ],
    },),
    runRealGit({
      cwd: repositoryRoot,
      args: [
        'config',
        'user.email',
        TEST_USER_EMAIL,
      ],
    },),
    writeFile(
      join(repositoryRoot, 'tracked.txt',),
      'tracked\n',
    ),
  ],);
  await runRealGit({
    cwd: repositoryRoot,
    args: [
      'add',
      '--',
      'tracked.txt',
    ],
  },);
  await runRealGit({
    cwd: repositoryRoot,
    args: [
      'commit',
      '--no-gpg-sign',
      '-m',
      'initialize fixture',
      '--',
      'tracked.txt',
    ],
  },);
}

/**
 * Commits exact fixture paths with real Git.
 *
 * @param repositoryRoot - fixture repository root
 *
 * @param message - commit subject
 *
 * @param paths - exact repository paths
 *
 * @example
 * ```ts
 * await commitPaths({ repositoryRoot: '/repo', message: 'fixture', paths: ['file'] });
 * ```
 */
export async function commitPaths({
  repositoryRoot,
  message,
  paths,
}: Readonly<{
  repositoryRoot: string;
  message: string;
  paths: readonly string[];
}>,): Promise<void> {
  await runRealGit({
    cwd: repositoryRoot,
    args: [
      'add',
      '--',
      ...paths,
    ],
  },);
  await runRealGit({
    cwd: repositoryRoot,
    args: [
      'commit',
      '--no-gpg-sign',
      '-m',
      message,
      '--',
      ...paths,
    ],
  },);
}

/**
 * Returns portable permission bits for no-follow path.
 *
 * @param path - exact filesystem path
 *
 * @returns portable permission bits
 *
 * @example
 * ```ts
 * await permissionMode('/repo/file');
 * ```
 */
export async function permissionMode(path: string,): Promise<number> {
  return (await lstat(path,)).mode & PERMISSION_BITS;
}

/**
 * Writes executable Node post-checkout hook fixture.
 *
 * @param repositoryRoot - fixture repository root
 *
 * @param body - CommonJS hook statements
 *
 * @example
 * ```ts
 * await writePostCheckoutHook({ repositoryRoot: '/repo', body: "process.exitCode = 1;" });
 * ```
 */
export async function writePostCheckoutHook({
  repositoryRoot,
  body,
}: Readonly<{
  repositoryRoot: string;
  body: string;
}>,): Promise<void> {
  /**
   * Shared linked-worktree hook path.
   */
  const hookPath = join(
    repositoryRoot,
    '.git',
    'hooks',
    'post-checkout',
  );
  await writeFile(
    hookPath,
    `#!/usr/bin/env node\n${body}\n`,
    { mode: EXECUTABLE_MODE, },
  );
  await chmod(
    hookPath,
    EXECUTABLE_MODE,
  );
}

/**
 * Returns cli-git ignored-state success summary lines.
 *
 * @param stderr - complete wrapper stderr
 *
 * @returns matching summary lines
 *
 * @example
 * ```ts
 * copySummaryLines(result.stderr);
 * ```
 */
export function copySummaryLines(stderr: string,): readonly string[] {
  return stderr.split('\n',)
    .filter(function copySummary(line,): boolean {
      return line.startsWith('cli-git: copied ',);
    },);
}
