import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import {
  delimiter,
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

import { resolveGit, } from './resolve-git.ts';

/** Absolute path to real git binary used for fixture setup and assertions. */
const realGitPath = await resolveGit();

/** Absolute path to cli-git entry point under test. */
const WRAPPER_PATH = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
  'node',
  'index.mjs',
);

/** Executable mode for disposable Node-based Git capture fixture. */
const TEST_EXECUTABLE_MODE = 0o755;

/** Git author email used in disposable repositories. */
const TEST_USER_EMAIL = 'cli-git@example.invalid';

/** Git author name used in disposable repositories. */
const TEST_USER_NAME = 'cli-git test';

/** Options for running git-like commands in tests. */
type RunGitOptions = {
  /** Working directory for subprocess. */
  readonly cwd: string;
  /** Arguments passed after executable name. */
  readonly args: readonly string[];
  /** Optional complete subprocess environment. */
  readonly env?: NodeJS.ProcessEnv;
};

/** Disposable temporary directory used by CLI integration tests. */
type TempDirectory = {
  /** Absolute path to temporary directory. */
  readonly path: string;
  /** Deletes temporary directory after test exits. */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 * Creates disposable temporary directory for git repositories.
 *
 * @returns Temporary directory that removes itself when disposed.
 *
 * @example
 * ```ts
 * await using tempDirectory = await createTempDirectory();
 * console.log(tempDirectory.path);
 * ```
 */
async function createTempDirectory(): Promise<TempDirectory> {
  /** Absolute temporary directory path for one test case. */
  const path = await mkdtemp(join(
    tmpdir(),
    'cli-git-index-',
  ),);

  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 * Runs real git binary, bypassing wrapper under test.
 *
 * @param options - Working directory and git argv.
 *
 * @returns Captured subprocess result.
 *
 * @example
 * ```ts
 * await runRealGit({ cwd: '/repo', args: ['status', '--short'] });
 * ```
 */
async function runRealGit(options: RunGitOptions,): Promise<Result> {
  return nanoSpawn(
    realGitPath,
    options.args,
    { cwd: options.cwd, },
  );
}

/**
 * Runs cli-git entry point through Node.
 *
 * @param options - Working directory and git argv.
 *
 * @returns Captured subprocess result.
 *
 * @example
 * ```ts
 * await runWrapper({ cwd: '/repo', args: ['status', '--short'] });
 * ```
 */
async function runWrapper(options: RunGitOptions,): Promise<Result> {
  return nanoSpawn(
    'node',
    [
      WRAPPER_PATH,
      ...options.args,
    ],
    {
      cwd: options.cwd,
      ...(options.env === undefined ? {} : { env: options.env, }),
    },
  );
}

/**
 * Sentinel returned by {@link catchWrapperError} when the wrapper unexpectedly
 * succeeded instead of failing. A real `Symbol` rather than `undefined` so the
 * "no error captured" case is a distinct value {@link requireSubprocessError}
 * rejects.
 */
const WRAPPER_SUCCEEDED = Symbol('git wrapper command unexpectedly succeeded',);

/**
 * Captures cli-git subprocess failure.
 *
 * @param options - Working directory and git argv.
 *
 * @returns Subprocess failure, or {@link WRAPPER_SUCCEEDED} when invocation succeeds.
 *
 * @example
 * ```ts
 * const error = await catchWrapperError({ cwd: '/repo', args: ['status'] });
 * expect(error).toBeInstanceOf(SubprocessError);
 * ```
 */
async function catchWrapperError(
  options: RunGitOptions,
): Promise<SubprocessError | typeof WRAPPER_SUCCEEDED> {
  try {
    await runWrapper(options,);
  }
  catch (error) {
    if (error instanceof SubprocessError)
      return error;
    throw error;
  }
  return WRAPPER_SUCCEEDED;
}

/**
 * Narrows captured subprocess error after expectation assertion.
 *
 * @param error - Subprocess error, or {@link WRAPPER_SUCCEEDED} from catch helper.
 *
 * @returns Subprocess error when present.
 *
 * @throws When subprocess unexpectedly succeeded.
 *
 * @example
 * ```ts
 * const error = requireSubprocessError(await catchWrapperError(options));
 * console.log(error.stderr);
 * ```
 */
function requireSubprocessError(error: SubprocessError | typeof WRAPPER_SUCCEEDED,): SubprocessError {
  expect(error,).toBeInstanceOf(SubprocessError,);

  if (error === WRAPPER_SUCCEEDED)
    throw new Error('Expected cli-git subprocess to fail.',);

  return error;
}

/**
 * Initializes disposable git repository and configures commit identity.
 *
 * @param options - Repository path to create and initialize.
 *
 * @returns Nothing after repository is initialized.
 *
 * @example
 * ```ts
 * await initializeRepository({ repoPath: '/tmp/repo' });
 * ```
 */
async function initializeRepository({
  repoPath,
}: {
  /** Absolute path to repository root. */
  readonly repoPath: string;
},): Promise<void> {
  await mkdir(
    repoPath,
    { recursive: true, },
  );
  await runRealGit({
    cwd: repoPath,
    args: [
      'init',
      '--quiet',
    ],
  },);
  await runRealGit({
    cwd: repoPath,
    args: [
      'config',
      'user.email',
      TEST_USER_EMAIL,
    ],
  },);
  await runRealGit({
    cwd: repoPath,
    args: [
      'config',
      'user.name',
      TEST_USER_NAME,
    ],
  },);
}

/**
 * Writes file and stages it with real git.
 *
 * @param options - Repository path, relative file name, and file content.
 *
 * @returns Nothing after file is staged.
 *
 * @example
 * ```ts
 * await writeAndStageFile({ repoPath: '/repo', fileName: 'file.txt', content: 'x\n' });
 * ```
 */
async function writeAndStageFile({
  repoPath,
  fileName,
  content,
}: {
  /** Absolute repository root. */
  readonly repoPath: string;
  /** Repository-relative file name. */
  readonly fileName: string;
  /** File content to write. */
  readonly content: string;
},): Promise<void> {
  await writeFile(
    join(
      repoPath,
      fileName,
    ),
    content,
  );
  await runRealGit({
    cwd: repoPath,
    args: [
      'add',
      fileName,
    ],
  },);
}

/**
 * Creates initial commit in repository.
 *
 * @param options - Repository path to seed.
 *
 * @returns Nothing after initial commit exists.
 *
 * @example
 * ```ts
 * await createInitialCommit({ repoPath: '/repo' });
 * ```
 */
async function createInitialCommit({
  repoPath,
}: {
  /** Absolute repository root. */
  readonly repoPath: string;
},): Promise<void> {
  await writeAndStageFile({
    repoPath,
    fileName: 'tracked.txt',
    content: 'initial\n',
  },);
  await runRealGit({
    cwd: repoPath,
    args: [
      'commit',
      '--quiet',
      '-m',
      'initial',
    ],
  },);
}

/**
 * Reads latest commit subject from repository.
 *
 * @param options - Repository path to inspect.
 *
 * @returns Latest commit subject.
 *
 * @example
 * ```ts
 * const subject = await readLatestSubject({ repoPath: '/repo' });
 * ```
 */
async function readLatestSubject({
  repoPath,
}: {
  /** Absolute repository root. */
  readonly repoPath: string;
},): Promise<string> {
  /** Git log result containing latest subject on stdout. */
  const result = await runRealGit({
    cwd: repoPath,
    args: [
      'log',
      '-1',
      '--format=%s',
    ],
  },);

  return result.stdout;
}

await describe({
  name: 'cli-git entry point',
  children: [
    it({
      name: 'emits stable wrapper JSONL and blocks real git below root',
      fn: async function testRequireRootJsonl(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        await initializeRepository({ repoPath: tempDirectory.path, },);
        /** Subdirectory rejected by require-root. */
        const subdirectoryPath = join(tempDirectory.path, 'subdirectory',);
        await mkdir(subdirectoryPath,);
        /** Built-wrapper policy failure. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: subdirectoryPath,
          args: ['status', '--short',],
        },),);
        /** Exact expected settled finding bytes. */
        const expectedEvent = JSON.stringify({
          schemaVersion: 1,
          sequence: 0,
          type: 'finding',
          trigger: 'pre-forward',
          policyId: 'require-root',
          severity: 'error',
          code: 'require-root/not-at-root',
          message: `cli-git: not at the root of the git repository. Repo root is ${tempDirectory.path} but effective cwd is ${subdirectoryPath}. Tip: cd to ${tempDirectory.path} or pass -C ${tempDirectory.path} before the subcommand.`,
          fix: 'none',
        },);
        expect(error.exitCode,).toBe(1,);
        expect(error.stderr,).toBe(expectedEvent,);
        expect(error.stdout,).toBe('',);
      },
    },),
    it({
      name: 'strips require-root escape before built git forwarding',
      fn: async function testBuiltEscapeForwarding(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        await initializeRepository({ repoPath: tempDirectory.path, },);
        /** Subdirectory allowed for one escaped invocation. */
        const subdirectoryPath = join(tempDirectory.path, 'subdirectory',);
        await mkdir(subdirectoryPath,);
        await runWrapper({
          cwd: subdirectoryPath,
          args: ['--no-enforce-require-root', 'status', '--short',],
        },);
      },
    },),
    it({
      name: 'preserves human and machine-readable status output',
      fn: async function testStatusOutputModes(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeFile(join(tempDirectory.path, 'untracked.txt',), 'content\n',);
        /** Human status with cli-git guidance. */
        const human = await runWrapper({ cwd: tempDirectory.path, args: ['status',], },);
        expect(human.stdout,).toContain('cli-git: bulk-add patterns',);
        expect(human.stdout,).not.toContain('use "git add <file>..."',);
        /** Porcelain output without wrapper prose. */
        const machine = await runWrapper({
          cwd: tempDirectory.path,
          args: ['status', '--porcelain=v1',],
        },);
        expect(machine.stdout,).toBe('?? untracked.txt',);
        /** Explicit user status-hints choice suppresses wrapper guidance. */
        const overridden = await runWrapper({
          cwd: tempDirectory.path,
          args: ['-c', 'advice.statusHints=true', 'status',],
        },);
        expect(overridden.stdout,).not.toContain('cli-git: bulk-add patterns',);
        expect(overridden.stdout,).toContain('use "git add <file>..."',);
      },
    },),
    it({
      name: 'runs recovery probe before exact transformed Git boundary',
      fn: async function testFinalTransformedForwarding(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /** Fake real-Git executable directory. */
        const fakeBin = join(tempDirectory.path, 'bin',);
        await mkdir(fakeBin,);
        /** Append-only capture path for forwarded argument vectors. */
        const capturePath = join(tempDirectory.path, 'captured.jsonl',);
        /** Node executable discovered by resolveGit after wrapper self path. */
        const fakeGitPath = join(fakeBin, 'git',);
        await writeFile(fakeGitPath, `#!/usr/bin/env node
import { appendFileSync } from 'node:fs';
const capturePath = process.env.CLI_GIT_CAPTURE_PATH;
if (capturePath === undefined) throw new Error('missing capture path');
appendFileSync(capturePath, JSON.stringify(process.argv.slice(2)) + '\\n');
`,);
        await chmod(fakeGitPath, TEST_EXECUTABLE_MODE,);
        /** Environment routing wrapper resolution to disposable Git boundary. */
        const env: NodeJS.ProcessEnv = {
          ...process.env,
          CLI_GIT_CAPTURE_PATH: capturePath,
          PATH: [
            fakeBin,
            dirname(process.execPath,),
          ].join(delimiter,),
        };
        await runWrapper({
          cwd: tempDirectory.path,
          args: ['push', 'origin', 'main',],
          env,
        },);
        await runWrapper({
          cwd: tempDirectory.path,
          args: ['status', '--porcelain=v1',],
          env,
        },);
        await runWrapper({
          cwd: tempDirectory.path,
          args: ['commit', '--dry-run', '-m', 'message', 'file.txt',],
          env,
        },);
        expect(await readFile(capturePath, 'utf8',),).toBe(
          '["rev-parse","--is-inside-work-tree"]\n'
          + '["rev-parse","--path-format=absolute","--git-common-dir"]\n'
          + '["push","--atomic","origin","main"]\n'
          + '["rev-parse","--is-inside-work-tree"]\n'
          + '["-c","advice.statusHints=false","rev-parse","--path-format=absolute","--git-common-dir"]\n'
          + '["-c","advice.statusHints=false","status","--porcelain=v1"]\n'
          + '["rev-parse","--is-inside-work-tree"]\n'
          + '["rev-parse","--path-format=absolute","--git-common-dir"]\n'
          + '["commit","-o","--dry-run","-m","message","file.txt"]\n',
        );
      },
    },),
    it({
      name: 'runs trust status and direct check management commands',
      fn: async function testManagementCommands(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        await initializeRepository({ repoPath: tempDirectory.path, },);
        /** Subdirectory checked by direct management operation. */
        const subdirectoryPath = join(tempDirectory.path, 'subdirectory',);
        await mkdir(subdirectoryPath,);
        /** Stable management status response. */
        const statusResult = await runWrapper({
          cwd: subdirectoryPath,
          args: ['cli-git', 'status',],
        },);
        expect(statusResult.stdout,).toBe(JSON.stringify({
          schemaVersion: 1,
          type: 'trust-status',
          configPresent: false,
          trusted: false,
          unchanged: false,
          reason: 'no-config',
        },),);
        /** Built-in direct-check failure. */
        const checkError = requireSubprocessError(await catchWrapperError({
          cwd: subdirectoryPath,
          args: ['cli-git', 'check', '--policy', 'require-root', '--all',],
        },),);
        expect(checkError.exitCode,).toBe(1,);
        expect(checkError.stdout,).toContain('"trigger":"direct-check"',);
        expect(checkError.stdout,).toContain('"code":"require-root/not-at-root"',);
        expect(checkError.stderr,).toBe('',);
        /** Direct check honoring Git global chdir before management namespace. */
        const globalChdirError = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: ['-C', subdirectoryPath, 'cli-git', 'check', '--all',],
        },),);
        expect(globalChdirError.exitCode,).toBe(1,);
        expect(globalChdirError.stdout,).toContain('"trigger":"direct-check"',);
      },
    },),
    it({
      name: 'reports direct-check usage failures with exit code two',
      fn: async function testManagementUsageFailure(): Promise<void> {
        await using tempDirectory = await createTempDirectory();
        /** Missing direct-check scope failure. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: ['cli-git', 'check',],
        },),);
        expect(error.exitCode,).toBe(2,);
        expect(error.stderr,).toContain('requires exactly one scope',);
        /** Mixed pre-separator and valid pathspec scope failure. */
        const mixedScopeError = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: ['cli-git', 'check', 'before', '--', 'after',],
        },),);
        expect(mixedScopeError.exitCode,).toBe(2,);
        expect(mixedScopeError.stderr,).toContain('pathspecs must follow --',);
      },
    },),
    it({
      name: 'prints wrapper diagnostic for pathless commit before git fatal',
      fn: async function testPathlessCommitDiagnostic(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'file.txt',
          content: 'content\n',
        },);

        /** cli-git failure for commit without pathspec. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '-m',
            'pathless',
          ],
        },),);

        expect(error.stderr,).toContain('"type":"core-finding"',);
        expect(error.stderr,).toContain('"coreId":"commit-only"',);
        expect(error.stderr,).toContain('"code":"commit-only/pathspec-required"',);
        expect(error.stderr,).toContain(
          'cli-git: git commit requires an explicit pathspec',
        );
        expect(error.stderr,).not.toContain('No paths with --include/--only',);
      },
    },),
    it({
      name: 'commits named path through wrapper-injected only mode',
      fn: async function testNamedPathCommit(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'file.txt',
          content: 'content\n',
        },);

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '-m',
            'named path',
            'file.txt',
          ],
        },);

        expect(await readLatestSubject({ repoPath: tempDirectory.path, },),).toBe(
          'named path',
        );
      },
    },),
    it({
      name: 'commits quiet named path before message-file option',
      fn: async function testQuietNamedPathBeforeMessageFile(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'file.txt',
          content: 'content\n',
        },);

        /** Path to message file consumed by `git commit -F`. */
        const messagePath = join(
          tempDirectory.path,
          'message.txt',
        );

        await writeFile(
          messagePath,
          'quiet named path\n',
        );

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '-q',
            'file.txt',
            '-F',
            'message.txt',
          ],
        },);

        expect(await readLatestSubject({ repoPath: tempDirectory.path, },),).toBe(
          'quiet named path',
        );
      },
    },),
    it({
      name: 'commits named path after separated author option',
      fn: async function testNamedPathAfterSeparatedAuthor(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'file.txt',
          content: 'content\n',
        },);

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '--author',
            'Alt Author <alt@example.invalid>',
            'file.txt',
            '-m',
            'author path',
          ],
        },);

        expect(await readLatestSubject({ repoPath: tempDirectory.path, },),).toBe(
          'author path',
        );
      },
    },),
    it({
      name: 'rejects pathless amend that would silently ignore staged changes',
      fn: async function testPathlessAmendWithStagedChanges(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'tracked.txt',
          content: 'staged update\n',
        },);

        /** Commit hash before the rejected amend. */
        const headBefore = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'rev-parse',
            'HEAD',
          ],
        },);

        /** cli-git failure for pathless amend over a dirty index. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '--amend',
            '--no-edit',
          ],
        },),);

        expect(error.stderr,).toContain(
          'git commit --amend without pathspecs would silently ignore your staged changes',
        );

        /** Commit hash after the rejected amend, proving no commit was rewritten. */
        const headAfter = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'rev-parse',
            'HEAD',
          ],
        },);

        expect(headAfter.stdout,).toBe(headBefore.stdout,);

        /** Repository status after rejected amend, proving change is still staged. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toContain('M  tracked.txt',);
      },
    },),
    it({
      name: 'amends message pathlessly when index matches HEAD',
      fn: async function testPathlessAmendWithCleanIndex(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '--amend',
            '-m',
            'reworded',
          ],
        },);

        expect(await readLatestSubject({ repoPath: tempDirectory.path, },),).toBe(
          'reworded',
        );
      },
    },),
    it({
      name: 'amends staged path into previous commit when pathspec names it',
      fn: async function testAmendWithPathspec(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'tracked.txt',
          content: 'staged update\n',
        },);

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '--amend',
            '--no-edit',
            'tracked.txt',
          ],
        },);

        /** Repository status after amend, proving change landed in the commit. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toBe('',);
        expect(await readLatestSubject({ repoPath: tempDirectory.path, },),).toBe(
          'initial',
        );
      },
    },),
    it({
      name: 'commits include-mode path without manufacturing an only conflict',
      fn: async function testIncludeModeCommit(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'tracked.txt',
          content: 'include mode\n',
        },);

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '-i',
            '-m',
            'include mode',
            'tracked.txt',
          ],
        },);

        expect(await readLatestSubject({ repoPath: tempDirectory.path, },),).toBe(
          'include mode',
        );
      },
    },),
    it({
      name: 'passes pathless commit through to conclude a merge',
      fn: async function testMergeConclusionCommit(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);

        /** Branch the repository started on; merged back into below. */
        const initialBranch = (await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'branch',
            '--show-current',
          ],
        },)).stdout;

        await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'checkout',
            '--quiet',
            '-b',
            'side',
          ],
        },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'tracked.txt',
          content: 'side version\n',
        },);
        await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '--quiet',
            '-m',
            'side',
            'tracked.txt',
          ],
        },);
        await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'checkout',
            '--quiet',
            initialBranch,
          ],
        },);
        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'tracked.txt',
          content: 'main version\n',
        },);
        await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '--quiet',
            '-m',
            'mainline',
            'tracked.txt',
          ],
        },);

        try {
          await runRealGit({
            cwd: tempDirectory.path,
            args: [
              'merge',
              'side',
            ],
          },);
        }
        catch (error) {
          // The conflicting merge is expected to exit non-zero; the conflict
          // is the fixture state the wrapper commit must conclude.
          if (!(error instanceof SubprocessError))
            throw error;
        }

        await writeAndStageFile({
          repoPath: tempDirectory.path,
          fileName: 'tracked.txt',
          content: 'resolved\n',
        },);

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'commit',
            '-m',
            'merge resolved',
          ],
        },);

        expect(await readLatestSubject({ repoPath: tempDirectory.path, },),).toBe(
          'merge resolved',
        );

        /** Second parent of HEAD, present only when the merge truly concluded. */
        const secondParent = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'rev-parse',
            '--verify',
            'HEAD^2',
          ],
        },);

        expect(secondParent.stdout
          .length,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'auto-pushes real commits but not implied dry runs',
      fn: async function testDryRunSkipsAutoPush(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        /** Working repository whose commits should be backed up. */
        const repoPath = join(
          tempDirectory.path,
          'repo',
        );
        /** Bare origin remote observing which pushes actually happen. */
        const remotePath = join(
          tempDirectory.path,
          'origin.git',
        );

        await initializeRepository({ repoPath, },);
        await createInitialCommit({ repoPath, },);
        await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'init',
            '--bare',
            '--quiet',
            remotePath,
          ],
        },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'remote',
            'add',
            'origin',
            remotePath,
          ],
        },);
        await writeAndStageFile({
          repoPath,
          fileName: 'tracked.txt',
          content: 'staged update\n',
        },);

        await runWrapper({
          cwd: repoPath,
          args: [
            'commit',
            '--short',
            '-m',
            'dry run',
            'tracked.txt',
          ],
        },);

        /** Remote refs after the implied dry run, which must not have pushed. */
        const remoteRefsAfterDryRun = await runRealGit({
          cwd: repoPath,
          args: [
            'ls-remote',
            'origin',
          ],
        },);

        expect(remoteRefsAfterDryRun.stdout,).toBe('',);

        await runWrapper({
          cwd: repoPath,
          args: [
            'commit',
            '-m',
            'real commit',
            'tracked.txt',
          ],
        },);

        /** Local HEAD that the auto-push must have backed up to origin. */
        const localHead = await runRealGit({
          cwd: repoPath,
          args: [
            'rev-parse',
            'HEAD',
          ],
        },);
        /** Remote refs after the real commit, which must include HEAD. */
        const remoteRefsAfterCommit = await runRealGit({
          cwd: repoPath,
          args: [
            'ls-remote',
            'origin',
          ],
        },);

        expect(remoteRefsAfterCommit.stdout,).toContain(localHead.stdout,);
      },
    },),
    it({
      name: 'skips auto-push with a note when HEAD is detached',
      fn: async function testDetachedHeadSkipsAutoPush(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        /** Working repository committed to while detached. */
        const repoPath = join(
          tempDirectory.path,
          'repo',
        );
        /** Bare origin remote that must stay empty. */
        const remotePath = join(
          tempDirectory.path,
          'origin.git',
        );

        await initializeRepository({ repoPath, },);
        await createInitialCommit({ repoPath, },);
        await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'init',
            '--bare',
            '--quiet',
            remotePath,
          ],
        },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'remote',
            'add',
            'origin',
            remotePath,
          ],
        },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'checkout',
            '--quiet',
            '--detach',
            'HEAD',
          ],
        },);
        await writeAndStageFile({
          repoPath,
          fileName: 'tracked.txt',
          content: 'detached update\n',
        },);

        /** Wrapper result whose stderr must carry the detached-HEAD note. */
        const result = await runWrapper({
          cwd: repoPath,
          args: [
            'commit',
            '-m',
            'detached commit',
            'tracked.txt',
          ],
        },);

        expect(result.stderr,).toContain('HEAD is detached',);
        expect(await readLatestSubject({ repoPath, },),).toBe('detached commit',);

        /** Remote refs after the detached commit, which must not have pushed. */
        const remoteRefs = await runRealGit({
          cwd: repoPath,
          args: [
            'ls-remote',
            'origin',
          ],
        },);

        expect(remoteRefs.stdout,).toBe('',);
      },
    },),
    it({
      name: 'rejects stash list at main worktree root',
      fn: async function testStashListAtMainWorktreeRoot(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);

        /** cli-git failure for read-only stash list inside main worktree. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'stash',
            'list',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: git stash is rejected in the main git worktree',
        );
      },
    },),
    it({
      name: 'allows stash list at main worktree root with worktree escape hatch',
      fn: async function testStashListAtMainWorktreeRootWithEscapeHatch(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);

        /** cli-git success proves wrapper stripped the unknown-to-git escape hatch. */
        const result = await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'stash',
            '--no-enforce-worktree',
            'list',
          ],
        },);

        expect(result.stdout,).toBe('',);
      },
    },),
    it({
      name: 'allows stash list at linked worktree root',
      fn: async function testStashListAtLinkedWorktreeRoot(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        /** Main repository path that owns linked worktree metadata. */
        const repoPath = join(
          tempDirectory.path,
          'repo',
        );
        /** Linked worktree path where stash remains allowed. */
        const linkedWorktreePath = join(
          tempDirectory.path,
          'linked',
        );

        await initializeRepository({ repoPath, },);
        await createInitialCommit({ repoPath, },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'worktree',
            'add',
            '--detach',
            '--quiet',
            linkedWorktreePath,
            'HEAD',
          ],
        },);

        /** cli-git success for read-only stash list inside linked worktree. */
        const result = await runWrapper({
          cwd: linkedWorktreePath,
          args: [
            'stash',
            'list',
          ],
        },);

        expect(result.stdout,).toBe('',);
      },
    },),
    it({
      name: 'rejects stash with explicit work-tree from unrelated cwd',
      fn: async function testStashWithExplicitWorkTreeOutsideWorktree(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        /** Repository whose worktree would be reverted if stash reached real git. */
        const repoPath = join(
          tempDirectory.path,
          'repo',
        );
        /** Launch directory outside any worktree. */
        const launchPath = join(
          tempDirectory.path,
          'launch',
        );

        await initializeRepository({ repoPath, },);
        await createInitialCommit({ repoPath, },);
        await mkdir(launchPath,);
        await writeFile(
          join(
            repoPath,
            'tracked.txt',
          ),
          'modified\n',
        );

        /** cli-git failure before explicit --work-tree stash can run. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: launchPath,
          args: [
            '--git-dir',
            join(
              repoPath,
              '.git',
            ),
            '--work-tree',
            repoPath,
            'stash',
            'push',
            '--',
            'tracked.txt',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: git stash requires the effective working directory to be inside a linked git worktree',
        );

        /** Repository status after rejected stash, proving file contents stayed modified. */
        const status = await runRealGit({
          cwd: repoPath,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toContain(' M tracked.txt',);
      },
    },),
    it({
      name: 'allows clean dry-run at main worktree root',
      fn: async function testCleanDryRunAtMainWorktreeRoot(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeFile(
          join(
            tempDirectory.path,
            'untracked.txt',
          ),
          'temporary\n',
        );

        /** cli-git success for dry-run clean inside main worktree. */
        const result = await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'clean',
            '-nd',
          ],
        },);

        expect(result.stdout,).toContain('Would remove untracked.txt',);
      },
    },),
    it({
      name: 'rejects add bulk pathspec after pathspec separator',
      fn: async function testAddBulkPathspecAfterSeparator(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeFile(
          join(
            tempDirectory.path,
            'file.txt',
          ),
          'content\n',
        );

        /** cli-git failure for `git add -- .` inside main worktree. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'add',
            '--',
            '.',
          ],
        },),);

        expect(error.stderr,).toContain('bulk-staging patterns (.)',);

        /** Repository status after rejected add, proving untracked file remains unstaged. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toContain('?? file.txt',);
      },
    },),
    it({
      name: 'rejects clean when no-dry-run follows dry-run',
      fn: async function testCleanNoDryRunAfterDryRun(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeFile(
          join(
            tempDirectory.path,
            'untracked.txt',
          ),
          'temporary\n',
        );

        /** cli-git failure because Git's later --no-dry-run makes clean destructive. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'clean',
            '--dry-run',
            '--no-dry-run',
            '-fd',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: state-changing git clean is rejected in the main git worktree',
        );

        /** Repository status after rejected clean, proving untracked file remains. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toContain('?? untracked.txt',);
      },
    },),
    it({
      name: 'rejects state-changing clean at main worktree root',
      fn: async function testStateChangingCleanAtMainWorktreeRoot(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeFile(
          join(
            tempDirectory.path,
            'untracked.txt',
          ),
          'temporary\n',
        );

        /** cli-git failure for non-dry-run clean inside main worktree. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'clean',
            '-fd',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: state-changing git clean is rejected in the main git worktree',
        );

        /** Repository status after rejected clean, proving untracked file remains. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toContain('?? untracked.txt',);
      },
    },),
    it({
      name:
        'allows state-changing clean at main worktree root with worktree escape hatch',
      fn: async function testCleanAtMainWorktreeRootWithEscapeHatch(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await writeFile(
          join(
            tempDirectory.path,
            'untracked.txt',
          ),
          'temporary\n',
        );

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'clean',
            '--no-enforce-worktree',
            '-fd',
          ],
        },);

        /** Repository status after escaped clean, proving real git received clean. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toBe('',);
      },
    },),
    it({
      name: 'rejects reset hard at main worktree root',
      fn: async function testResetHardAtMainWorktreeRoot(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);
        await writeFile(
          join(
            tempDirectory.path,
            'tracked.txt',
          ),
          'modified\n',
        );

        /** cli-git failure for reset hard inside main worktree. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'reset',
            '--hard',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: destructive git reset modes are rejected in the main git worktree',
        );

        /** Repository status after rejected reset, proving tracked modification remains. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toContain(' M tracked.txt',);
      },
    },),
    it({
      name: 'allows reset hard at main worktree root with worktree escape hatch',
      fn: async function testResetHardAtMainWorktreeRootWithEscapeHatch(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);
        await writeFile(
          join(
            tempDirectory.path,
            'tracked.txt',
          ),
          'modified\n',
        );

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'reset',
            '--no-enforce-worktree',
            '--hard',
          ],
        },);

        /** Repository status after escaped reset, proving real git received reset hard. */
        const status = await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'status',
            '--short',
          ],
        },);

        expect(status.stdout,).toBe('',);
      },
    },),
    it({
      name: 'rejects linked worktree subdirectory because .git is a file',
      fn: async function testLinkedWorktreeSubdirectory(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        /** Main repository path inside disposable parent directory. */
        const repoPath = join(
          tempDirectory.path,
          'repo',
        );
        /** Linked worktree path with `.git` file instead of directory. */
        const worktreePath = join(
          tempDirectory.path,
          'worktree',
        );
        /** Subdirectory below linked worktree root. */
        const subdirectory = join(
          worktreePath,
          'subdir',
        );

        await initializeRepository({ repoPath, },);
        await createInitialCommit({ repoPath, },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'worktree',
            'add',
            '--detach',
            '--quiet',
            worktreePath,
            'HEAD',
          ],
        },);
        await mkdir(subdirectory,);

        /** cli-git failure for worktree subdirectory. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: subdirectory,
          args: [
            'status',
            '--short',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: not at the root of the git repository',
        );
        expect(error.stderr,).toContain(`Repo root is ${worktreePath}`,);
      },
    },),
    it({
      name: 'rejects checkout branch creation in current worktree',
      fn: async function testCheckoutBranchCreationRejected(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);

        /** cli-git failure for checkout branch creation inside current worktree. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: tempDirectory.path,
          args: [
            'checkout',
            '-b',
            'side',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: git checkout branch creation is rejected in the current worktree',
        );
      },
    },),
    it({
      name: 'allows checkout branch creation with branch escape hatch',
      fn: async function testCheckoutBranchCreationEscapeHatch(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        await initializeRepository({ repoPath: tempDirectory.path, },);
        await createInitialCommit({ repoPath: tempDirectory.path, },);

        await runWrapper({
          cwd: tempDirectory.path,
          args: [
            'checkout',
            '--no-enforce-worktree-branch',
            '-b',
            'side',
          ],
        },);

        /** Branch checked out after escape-hatched branch creation. */
        const branchName = (await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'branch',
            '--show-current',
          ],
        },)).stdout;

        expect(branchName,).toBe('side',);
      },
    },),
    it({
      name: 'allows branch creation through worktree add',
      fn: async function testWorktreeAddBranchCreationAllowed(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        /** Main repository path that owns linked worktree metadata. */
        const repoPath = join(
          tempDirectory.path,
          'repo',
        );
        /** Linked worktree path to create through wrapper. */
        const worktreePath = join(
          tempDirectory.path,
          'side-worktree',
        );

        await initializeRepository({ repoPath, },);
        await createInitialCommit({ repoPath, },);

        await runWrapper({
          cwd: repoPath,
          args: [
            'worktree',
            'add',
            '--quiet',
            '-b',
            'side',
            worktreePath,
            'HEAD',
          ],
        },);

        /** Branch checked out in linked worktree. */
        const branchName = (await runRealGit({
          cwd: worktreePath,
          args: [
            'branch',
            '--show-current',
          ],
        },)).stdout;

        expect(branchName,).toBe('side',);
      },
    },),
    it({
      name: 'rejects implicit remote branch guess in current worktree',
      fn: async function testImplicitRemoteBranchGuessRejected(): Promise<void> {
        await using tempDirectory = await createTempDirectory();

        /** Working repository with one matching remote branch and no local branch. */
        const repoPath = join(
          tempDirectory.path,
          'repo',
        );
        /** Bare origin remote that owns the guessed branch. */
        const remotePath = join(
          tempDirectory.path,
          'origin.git',
        );

        await initializeRepository({ repoPath, },);
        await createInitialCommit({ repoPath, },);
        await runRealGit({
          cwd: tempDirectory.path,
          args: [
            'init',
            '--bare',
            '--quiet',
            remotePath,
          ],
        },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'remote',
            'add',
            'origin',
            remotePath,
          ],
        },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'push',
            '--quiet',
            'origin',
            'HEAD:refs/heads/remote-topic',
          ],
        },);
        await runRealGit({
          cwd: repoPath,
          args: [
            'fetch',
            '--quiet',
            'origin',
          ],
        },);

        /** cli-git failure for implicit remote-tracking branch creation. */
        const error = requireSubprocessError(await catchWrapperError({
          cwd: repoPath,
          args: [
            'switch',
            'remote-topic',
          ],
        },),);

        expect(error.stderr,).toContain(
          'cli-git: git switch for remote-topic branch creation is rejected in the current worktree',
        );
      },
    },),
  ],
},);
