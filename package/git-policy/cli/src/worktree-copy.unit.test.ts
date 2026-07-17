import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  readlink,
  rm,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
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
const WRAPPER_PATH = join(
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
 * Creates one disposable filesystem root.
 *
 * @returns asynchronously disposable temporary directory
 *
 * @example
 * ```ts
 * await using fixture = await createTempDirectory();
 * ```
 */
async function createTempDirectory(): Promise<TempDirectory> {
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
function runRealGit({
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
async function captureWrapper({
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
function requireSuccess(outcome: WrapperOutcome,): Result {
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
function requireFailure(outcome: WrapperOutcome,): SubprocessError {
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
async function initializeRepository(repositoryRoot: string,): Promise<void> {
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
async function commitPaths({
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
async function permissionMode(path: string,): Promise<number> {
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
async function writePostCheckoutHook({
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
function copySummaryLines(stderr: string,): readonly string[] {
  return stderr.split('\n',)
    .filter(function copySummary(line,): boolean {
      return line.startsWith('cli-git: copied ',);
    },);
}

await describe({
  name: 'automatic ignored-state worktree copying',
  concurrency: 1,
  children: [
    it({
      name: 'copies the standard ignore stack, structure, symlinks, and permission bits',
      fn: async () => {
        await using fixture = await createTempDirectory();
        /**
         * Source repository root.
         */
        const repositoryRoot = join(fixture.path, 'repository',);
        /**
         * New linked-worktree root.
         */
        const destinationRoot = join(fixture.path, 'topic',);
        await initializeRepository(repositoryRoot,);
        await Promise.all([
          writeFile(
            join(repositoryRoot, '.gitignore',),
            'cache/\nfrom-tree.txt\nlink.txt\n',
          ),
          writeFile(
            join(repositoryRoot, '.git', 'info', 'exclude',),
            'from-info.txt\n',
          ),
          writeFile(
            join(fixture.path, 'global-excludes',),
            'from-global.txt\n',
          ),
          mkdir(
            join(repositoryRoot, 'cache',),
            { mode: 0o750, },
          ),
        ],);
        await commitPaths({
          repositoryRoot,
          message: 'add ignore contract',
          paths: ['.gitignore',],
        },);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'config',
            'core.excludesFile',
            join(fixture.path, 'global-excludes',),
          ],
        },);
        await Promise.all([
          writeFile(join(repositoryRoot, 'cache', 'nested.txt',), 'nested\n', { mode: 0o640, },),
          writeFile(join(repositoryRoot, 'from-tree.txt',), 'tree\n', { mode: 0o751, },),
          writeFile(join(repositoryRoot, 'from-info.txt',), 'info\n',),
          writeFile(join(repositoryRoot, 'from-global.txt',), 'global\n',),
          symlink('cache/nested.txt', join(repositoryRoot, 'link.txt',),),
        ],);
        await Promise.all([
          chmod(join(repositoryRoot, 'cache',), 0o750,),
          chmod(join(repositoryRoot, 'cache', 'nested.txt',), 0o640,),
          chmod(join(repositoryRoot, 'from-tree.txt',), 0o751,),
        ],);

        /**
         * Successful built-wrapper worktree creation.
         */
        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'topic',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'cache', 'nested.txt',), 'utf8',),).toBe('nested\n',);
        expect(await readFile(join(destinationRoot, 'from-tree.txt',), 'utf8',),).toBe('tree\n',);
        expect(await readFile(join(destinationRoot, 'from-info.txt',), 'utf8',),).toBe('info\n',);
        expect(await readFile(join(destinationRoot, 'from-global.txt',), 'utf8',),).toBe('global\n',);
        expect(await readlink(join(destinationRoot, 'link.txt',),),).toBe('cache/nested.txt',);
        expect(await permissionMode(join(destinationRoot, 'cache',),),).toBe(0o750,);
        expect(await permissionMode(join(destinationRoot, 'cache', 'nested.txt',),),).toBe(0o640,);
        expect(await permissionMode(join(destinationRoot, 'from-tree.txt',),),).toBe(0o751,);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'detects an ordinary Git alias and copies after no-checkout registration',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'alias-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({
          repositoryRoot,
          message: 'ignore state',
          paths: ['.gitignore',],
        },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'source state\n',);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'config',
            'alias.new-worktree',
            'worktree add --no-checkout -b alias-topic',
          ],
        },);

        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'new-worktree',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('source state\n',);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'copies source-ignored state even when destination reports it untracked',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'old-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'branch',
            'old',
          ],
        },);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({
          repositoryRoot,
          message: 'ignore local state',
          paths: ['.gitignore',],
        },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'source only\n',);

        requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            destinationRoot,
            'old',
          ],
        },),);
        const status = await runRealGit({
          cwd: destinationRoot,
          args: [
            'status',
            '--short',
            '--',
            'state.txt',
          ],
        },);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('source only\n',);
        expect(status.stdout,).toContain('?? state.txt',);
      },
    },),

    it({
      name: 'retains differing tracked destination collision and exits two',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'collision-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, 'state.txt',), 'tracked old\n',);
        await commitPaths({
          repositoryRoot,
          message: 'track old state',
          paths: ['state.txt',],
        },);
        await runRealGit({ cwd: repositoryRoot, args: ['branch', 'old',], },);
        await rm(join(repositoryRoot, 'state.txt',),);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await runRealGit({ cwd: repositoryRoot, args: ['add', '--', '.gitignore', 'state.txt',], },);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'commit',
            '--no-gpg-sign',
            '-m',
            'replace tracked state with ignore',
          ],
        },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'ignored new\n',);

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            destinationRoot,
            'old',
          ],
        },),);

        expect(error.exitCode,).toBe(2,);
        expect(error.stderr,).toContain('would overwrite differing destination entry',);
        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('tracked old\n',);
      },
    },),

    it({
      name: 'accepts an exact hook-created destination entry',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'hook-match',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore hook state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'same\n',);
        await writePostCheckoutHook({
          repositoryRoot,
          body: "require('node:fs').writeFileSync('state.txt', 'same\\n');",
        },);

        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'hook-match',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('same\n',);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'copies into a worktree retained after post-checkout failure and preserves Git status',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'hook-failure',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'copied after failure\n',);
        await writePostCheckoutHook({
          repositoryRoot,
          body: 'process.exitCode = 7;',
        },);

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'hook-failure',
            destinationRoot,
          ],
        },),);

        expect(error.exitCode,).toBe(7,);
        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('copied after failure\n',);
        expect(copySummaryLines(error.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'excludes a nested registered worktree while copying its ignored parent tree',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(repositoryRoot, 'sandboxes', 'nested-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'sandboxes/\n',);
        await commitPaths({ repositoryRoot, message: 'ignore sandboxes', paths: ['.gitignore',], },);
        await mkdir(join(repositoryRoot, 'sandboxes',),);
        await writeFile(join(repositoryRoot, 'sandboxes', 'local.txt',), 'local sibling\n',);

        requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'nested-topic',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'sandboxes', 'local.txt',), 'utf8',),).toBe('local sibling\n',);
        expect(await readFile(join(destinationRoot, '.git',), 'utf8',),).toContain('gitdir:',);
      },
    },),

    it({
      name: 'ignores a stale missing registered root while detecting the new worktree',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const staleRoot = join(fixture.path, 'stale',);
        const destinationRoot = join(fixture.path, 'healthy',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'state.txt\n',);
        await commitPaths({ repositoryRoot, message: 'ignore state', paths: ['.gitignore',], },);
        await writeFile(join(repositoryRoot, 'state.txt',), 'healthy\n',);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'stale',
            staleRoot,
          ],
        },);
        await rm(staleRoot, { recursive: true, force: true, },);

        requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'healthy',
            destinationRoot,
          ],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('healthy\n',);
      },
    },),

    it({
      name: 'uses an empty source set for bare repository creation',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const bareRoot = join(fixture.path, 'repository.git',);
        const destinationRoot = join(fixture.path, 'bare-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: fixture.path,
          args: [
            'clone',
            '--bare',
            repositoryRoot,
            bareRoot,
          ],
        },);

        const result = requireSuccess(await captureWrapper({
          cwd: fixture.path,
          args: [
            '--git-dir',
            bareRoot,
            'worktree',
            'add',
            '-b',
            'bare-topic',
            destinationRoot,
            'main',
          ],
        },),);

        expect(result.stderr,).toContain('bare repository with an empty source set',);
        expect(copySummaryLines(result.stderr,),).toHaveLength(1,);
      },
    },),

    it({
      name: 'rejects an ignored FIFO without opening it and retains new worktree',
      skip: process.platform === 'win32',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'fifo-topic',);
        await initializeRepository(repositoryRoot,);
        await writeFile(join(repositoryRoot, '.gitignore',), 'pipe\n',);
        await commitPaths({ repositoryRoot, message: 'ignore pipe', paths: ['.gitignore',], },);
        await nanoSpawn(
          'mkfifo',
          [join(repositoryRoot, 'pipe',),],
        );

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'fifo-topic',
            destinationRoot,
          ],
        },),);

        expect(error.exitCode,).toBe(2,);
        expect(error.stderr,).toContain('unsupported filesystem type',);
        expect((await lstat(join(repositoryRoot, 'pipe',),)).isFIFO(),).toBe(true,);
        expect(await readFile(join(destinationRoot, '.git',), 'utf8',),).toContain('gitdir:',);
      },
    },),

    it({
      name: 'recovers a staged journal before forwarding the next command',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'recovery-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'recovery-topic',
            destinationRoot,
          ],
        },);
        /**
         * Private staged payload sibling to destination.
         */
        const stageContainer = join(
          dirname(destinationRoot,),
          '.cli-git-worktree-copy-recovery',
        );
        const stageRoot = join(stageContainer, 'payload',);
        await mkdir(stageRoot, { recursive: true, mode: 0o700, },);
        await Promise.all([
          chmod(stageContainer, 0o700,),
          chmod(stageRoot, 0o700,),
          writeFile(join(stageRoot, 'state.txt',), 'recovered\n', { mode: 0o640, },),
        ],);
        await chmod(join(stageRoot, 'state.txt',), 0o640,);
        /**
         * Common Git directory returned by real Git.
         */
        const commonDir = (await runRealGit({
          cwd: repositoryRoot,
          args: [
            'rev-parse',
            '--path-format=absolute',
            '--git-common-dir',
          ],
        },)).stdout.trim();
        const journalRoot = join(commonDir, 'cli-git-worktree-copy', 'v1',);
        const journalPath = join(journalRoot, 'recovery.json',);
        await mkdir(journalRoot, { recursive: true, mode: 0o700, },);
        await chmod(journalRoot, 0o700,);
        await writeFile(
          journalPath,
          `${JSON.stringify({
            destinationRoot,
            intendedEntries: [],
            phase: 'staged',
            selectedRoots: ['state.txt',],
            sourceRoot: repositoryRoot,
            stageContainer,
            stageRoot,
            version: 1,
          },)}\n`,
          { mode: 0o600, },
        );

        const result = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: ['status', '--short',],
        },),);

        expect(await readFile(join(destinationRoot, 'state.txt',), 'utf8',),).toBe('recovered\n',);
        expect(result.stderr,).toContain('recovered ignored-state copies for 1 worktree transaction',);
        /**
         * Next invocation proving no completed journal remains to recover.
         */
        const nextResult = requireSuccess(await captureWrapper({
          cwd: repositoryRoot,
          args: ['status', '--short',],
        },),);
        expect(nextResult.stderr,).not.toContain('recovered ignored-state copies',);
      },
    },),

    it({
      name: 'rejects a journal whose stage path escapes destination sibling boundary',
      fn: async () => {
        await using fixture = await createTempDirectory();
        const repositoryRoot = join(fixture.path, 'repository',);
        const destinationRoot = join(fixture.path, 'malicious-topic',);
        await initializeRepository(repositoryRoot,);
        await runRealGit({
          cwd: repositoryRoot,
          args: [
            'worktree',
            'add',
            '-b',
            'malicious-topic',
            destinationRoot,
          ],
        },);
        const commonDir = (await runRealGit({
          cwd: repositoryRoot,
          args: [
            'rev-parse',
            '--path-format=absolute',
            '--git-common-dir',
          ],
        },)).stdout.trim();
        const journalRoot = join(commonDir, 'cli-git-worktree-copy', 'v1',);
        const journalPath = join(journalRoot, 'malicious.json',);
        await mkdir(journalRoot, { recursive: true, mode: 0o700, },);
        await chmod(journalRoot, 0o700,);
        await writeFile(
          journalPath,
          `${JSON.stringify({
            destinationRoot,
            intendedEntries: [],
            phase: 'complete',
            selectedRoots: [],
            sourceRoot: repositoryRoot,
            stageContainer: fixture.path,
            stageRoot: join(fixture.path, 'payload',),
            version: 1,
          },)}\n`,
          { mode: 0o600, },
        );

        const error = requireFailure(await captureWrapper({
          cwd: repositoryRoot,
          args: ['status', '--short',],
        },),);

        expect(error.exitCode,).toBe(2,);
        expect(error.stderr,).toContain('unsafe private stage relation',);
        expect(await readFile(journalPath, 'utf8',),).toContain('malicious-topic',);
      },
    },),
  ],
},);
