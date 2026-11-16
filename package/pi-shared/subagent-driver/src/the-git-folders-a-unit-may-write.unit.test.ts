/**
 What a run's unit is told it may write, asked of real Git over the layouts a checkout comes in: a plain checkout, a linked worktree,
 a checkout whose Git directory is kept elsewhere, a bare repository and a worktree of one, a submodule, and a checkout whose Git
 is named by `GIT_DIR`.

 Each row compares the folders this package names against Git's own answers for that checkout, and fails on any named folder that
 holds the checkout: a writable folder above the checkout makes the whole checkout writable with it. A layout Git answers but this
 package cannot describe must be refused, never guessed.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  realpath,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  GitFolderAboveTheCheckoutError,
  fileSystemOfARun,
  fileSystemPropertiesOf,
  foldersAToolMakesIn,
  gitRunnerOf,
  makeTheFoldersAToolMakesIn,
  runCommandWithChildProcess,
} from '../dist/final/node/index.mjs';

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 How this test asks git a question, which is how the package asks it: this repository's wrapper, from the folder asked about.
 */
const git = gitRunnerOf(runCommandWithChildProcess,);

/**
 The variables git is asked with, so a fixture never waits for a prompt.
 */
const GIT_ENVIRONMENT: Readonly<Record<string, string>> = {
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};

/**
 A folder a test may write in, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 A folder a test may write in, removed when the test ends.

 @returns the folder, whose path is its real path

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path, with symbolic links resolved so that Git's answers and this package's compare equal.
   */
  const path = await realpath(
    await mkdtemp(join(scratchRoot(), 'pi-git-layout-',),),
  );
  return {
    path,
    async [Symbol.asyncDispose]() {
      // A notice the wrapper's commit service writes into a fixture's Git directory can land while the folder is removed, so
      // the removal is retried, as Node does for ENOTEMPTY when it is asked to.
      await rm(path, {
        recursive: true,
        force: true,
        maxRetries: 5,
        retryDelay: 200,
      },);
    },
  };
}

/**
 Says whether one folder holds another.

 @param folder - the folder that may hold the other, which must be compared as a path

 @param other - the folder that may be inside it

 @returns whether `folder` is a strict ancestor of `other`

 @example
 ```ts
 const above = isAncestor({ folder: '/repo', other: '/repo/.git', });
 ```
 */
function isAncestor({ folder, other, }: { readonly folder: string; readonly other: string },): boolean {
  return (folder !== other) && other.startsWith(`${folder}/`,);
}

/**
 Runs one git command in a folder and answers what it printed.

 @param folder - where git runs

 @param words - its arguments

 @param environment - variables it gets on top of this test's

 @returns its standard output, trimmed

 @throws Error when git fails, which no fixture step may

 @example
 ```ts
 await gitIn({ folder: '/repo', words: ['init', '--initial-branch=main',], environment: {}, });
 ```
 */
async function gitIn({
  folder,
  words,
  environment = {},
}: {
  readonly folder: string;
  readonly words: readonly string[];
  readonly environment?: Readonly<Record<string, string>>;
},): Promise<string> {
  /**
   What git answered.
   */
  const answered = await execFileAsync('git', [...words,], {
    cwd: folder,
    env: {
      ...process.env,
      ...GIT_ENVIRONMENT,
      ...environment,
    },
    encoding: 'utf8',
    timeout: 60_000,
    killSignal: 'SIGKILL',
  },);
  return answered.stdout.trim();
}

/**
 Makes a repository with one commit at a folder.

 @param folder - where the repository is made, and its root

 @example
 ```ts
 await makeRepository({ folder: '/w/checkout', });
 ```
 */
async function makeRepository({ folder, }: { readonly folder: string },): Promise<void> {
  await mkdir(folder, { recursive: true, },);
  await gitIn({ folder, words: ['init', '--initial-branch=main',], });
  await gitIn({ folder, words: ['-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', 'commit', '--allow-empty', '--message', 'base',], });
}

/**
 Asks Git, as this test does for a fixture, which Git directories a checkout has.

 @param request - the checkout and the variables git is asked with

 @param request.folder - the checkout

 @param request.environment - variables git is asked with

 @returns the Git directory of the checkout and the one every worktree of it shares

 @example
 ```ts
 const answers = await askedOfGit({ folder: '/repo', environment: {}, });
 ```
 */
async function askedOfGit({
  folder,
  environment = {},
}: {
  readonly folder: string;
  readonly environment?: Readonly<Record<string, string>>;
},): Promise<{ readonly gitDirectory: string; readonly commonDirectory: string }> {
  /**
   What Git printed for the two directories.
   */
  const printed = await gitIn({
    folder,
    words: ['rev-parse', '--path-format=absolute', '--absolute-git-dir', '--git-common-dir',],
    environment,
  },);
  /**
   Those two paths.
   */
  const [gitDirectory, commonDirectory,] = printed.split('\n',);
  if ((gitDirectory === undefined) || (commonDirectory === undefined)) {
    throw new Error(`Git answered ${JSON.stringify(printed)} for ${folder}'s two directories`,);
  }
  return {
    gitDirectory,
    commonDirectory,
  };
}

/**
 Checks what this package names for one checkout against Git's own answers and the rule that no writable folder holds the checkout.

 @param request - the checkout, where the run works, where its driver writes, and the variables git is asked with

 @param request.checkoutFolder - the checkout a start names

 @param request.worktree - where the run works

 @param request.runFolder - the run's own folder

 @param request.environment - the variables git is asked with

 @example
 ```ts
 await expectWhatTheUnitMayWrite({ checkoutFolder: '/repo', worktree: '/repo/.claude/worktrees/pi-a', runFolder: '/runs/a-1', environment: {}, });
 ```
 */
async function expectWhatTheUnitMayWrite({
  checkoutFolder,
  worktree,
  runFolder,
  environment = {},
}: {
  readonly checkoutFolder: string;
  readonly worktree: string;
  readonly runFolder: string;
  readonly environment?: Readonly<Record<string, string>>;
},): Promise<void> {
  /**
   What Git itself says the checkout's Git directories are.
   */
  const answers = await askedOfGit({
    folder: checkoutFolder,
    environment,
  },);
  /**
   What this package names for the same checkout.
   */
  const fileSystem = await fileSystemOfARun({
    git: async function answeredByThisTest(request,) {
      return await runCommandWithChildProcess({
        command: 'git',
        words: request.words,
        folder: request.folder,
        environment: {
          ...GIT_ENVIRONMENT,
          ...environment,
        },
      },);
    },
    checkoutFolder,
    worktree,
    runFolder,
    worksInTheCheckoutItself: false,
  },);
  // The Git folders Git names are writable wherever they lie, the bare repository's own directory included: a run's commits go
  // there, and a read-only view of it would only block them.
  for (const answered of [answers.gitDirectory, answers.commonDirectory,]) {
    expect(fileSystem.foldersAUnitMustWrite,).toContain(answered,);
  }
  // The checkout's files are what is read-only, and a checkout that is itself a Git directory of the run cannot be: its
  // read-only view would stop the commit that keeps the run's work.
  for (const readOnly of fileSystem.foldersAUnitCannotWrite) {
    expect(readOnly,).toBe(checkoutFolder,);
    expect([answers.gitDirectory, answers.commonDirectory,],).not.toContain(readOnly,);
  }
  for (const named of fileSystem.foldersAUnitMustWrite) {
    expect(isAncestor({
      folder: named,
      other: checkoutFolder,
    },),).toBe(false,);
  }
  for (const named of foldersAToolMakesIn({ checkoutFolder, },)) {
    expect(fileSystem.foldersAUnitMustWrite,).toContain(named,);
  }
  expect(fileSystem.foldersAUnitMustWrite,).toContain(runFolder,);
}

/**
 Asks this machine's manager whether it answers at all.

 @returns what it said when it does not answer, or nothing when it does

 @example
 ```ts
 const { reason, } = await noManagerReason();
 ```
 */
async function noManagerReason(): Promise<{ readonly reason?: string }> {
  try {
    await execFileAsync('systemd-run', [
      '--user',
      '--wait',
      '--collect',
      '--quiet',
      `--unit=probe-git-folders-manager-${String(process.pid,)}`,
      '--property=RuntimeMaxSec=30',
      '--property=CPUQuota=100%',
      '--property=MemoryMax=256M',
      '--property=TasksMax=64',
      '/bin/true',
    ], {
      encoding: 'utf8',
      timeout: 60_000,
      killSignal: 'SIGKILL',
    },);
    return {};
  }
  catch (error) {
    return { reason: `no user manager answered ${String(error,)}`, };
  }
}

/**
 Runs one command in a real capped unit with the folders this package names for a run, and answers its status.

 @param request - the unit's name, the process's folders, and the command

 @param request.unit - the unit's name

 @param request.fileSystem - what the run's unit may and may not write

 @param request.words - the command

 @returns the unit's status and what it printed

 @example
 ```ts
 const ended = await inAUnitOfTheRun({ unit: 'probe-commit', fileSystem, words: ['git', 'commit', '--allow-empty', '--message', 'x'], });
 ```
 */
async function inAUnitOfTheRun({
  unit,
  fileSystem,
  words,
}: {
  readonly unit: string;
  readonly fileSystem: Awaited<ReturnType<typeof fileSystemOfARun>>;
  readonly words: readonly string[];
},): Promise<{ readonly status: number; readonly output: string }> {
  try {
    /**
     What the unit printed.
     */
    const answered = await execFileAsync('systemd-run', [
      '--user',
      '--wait',
      '--collect',
      '--quiet',
      '--pipe',
      `--unit=${unit}`,
      '--property=RuntimeMaxSec=30',
      '--property=CPUQuota=100%',
      '--property=MemoryMax=512M',
      '--property=TasksMax=64',
      ...fileSystemPropertiesOf(fileSystem,),
      '--setenv=PATH=/usr/bin:/bin',
      '--setenv=GIT_ASKPASS=',
      '--setenv=GIT_TERMINAL_PROMPT=0',
      ...words,
    ], {
      encoding: 'utf8',
      timeout: 60_000,
      killSignal: 'SIGKILL',
    },);
    return {
      status: 0,
      output: `${answered.stdout}${answered.stderr}`,
    };
  }
  catch (error) {
    /**
     What a unit that ended with a status reported, read field by field because the status is the answer.
     */
    const failure = error as {
      readonly code?: unknown;
      readonly stdout?: unknown;
      readonly stderr?: unknown;
    };
    return {
      status: ((typeof failure.code) === 'number') ? failure.code : (-1),
      output: `${((typeof failure.stdout) === 'string') ? failure.stdout : ''}${((typeof failure.stderr) === 'string') ? failure.stderr : ''}`,
    };
  }
}

/**
 Why every manager test is skipped when this machine's manager does not answer, asked once for the whole file.
 */
const { reason: NO_MANAGER_REASON, } = await noManagerReason();

await describe({
  name: '',
  children: [
    describe({
      name: fileSystemOfARun.name,
      children: [
        it({
          name: 'names the checkout own Git directory for a plain checkout, and nothing above it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The plain checkout.
             */
            const checkout = join(folder.path, 'checkout',);
            await makeRepository({ folder: checkout, },);
            await expectWhatTheUnitMayWrite({
              checkoutFolder: checkout,
              worktree: join(checkout, '.claude', 'worktrees', 'pi-a',),
              runFolder: join(folder.path, 'runs', 'a-1',),
            },);
          },
        },),
        it({
          name: 'names a linked worktree own Git directory and the shared one for a linked worktree named as the checkout',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The repository the linked worktree belongs to.
             */
            const checkout = join(folder.path, 'checkout',);
            await makeRepository({ folder: checkout, },);
            /**
             The linked worktree, which a caller can name with --checkout-folder.
             */
            const linked = join(folder.path, 'linked',);
            await gitIn({ folder: checkout, words: ['worktree', 'add', '-b', 'linked', linked,], });
            await expectWhatTheUnitMayWrite({
              checkoutFolder: linked,
              worktree: join(linked, '.claude', 'worktrees', 'pi-a',),
              runFolder: join(folder.path, 'runs', 'a-1',),
            },);
          },
        },),
        it({
          name: 'names the real Git directory of a checkout that keeps its Git elsewhere, and never the folder above it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout whose Git directory is kept apart.
             */
            const checkout = join(folder.path, 'checkout',);
            await mkdir(join(folder.path, 'storage', 'actual.git',), { recursive: true, },);
            await gitIn({
              folder: folder.path,
              words: ['init', '--initial-branch=main', '--separate-git-dir', join(folder.path, 'storage', 'actual.git',), checkout,],
            },);
            await gitIn({ folder: checkout, words: ['-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', 'commit', '--allow-empty', '--message', 'base',], });
            await expectWhatTheUnitMayWrite({
              checkoutFolder: checkout,
              worktree: join(checkout, '.claude', 'worktrees', 'pi-a',),
              runFolder: join(folder.path, 'runs', 'a-1',),
            },);
          },
        },),
        it({
          name: 'names the Git directory of a bare repository, which is the checkout itself',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A repository with one commit, cloned bare, so a worktree of it can be made.
             */
            const source = join(folder.path, 'source',);
            await makeRepository({ folder: source, },);
            /**
             The bare repository, named as the checkout.
             */
            const bare = join(folder.path, 'bare.git',);
            await gitIn({ folder: folder.path, words: ['clone', '--bare', source, bare,], });
            await expectWhatTheUnitMayWrite({
              checkoutFolder: bare,
              worktree: join(folder.path, 'bare-worktree',),
              runFolder: join(folder.path, 'runs', 'a-1',),
            },);
          },
        },),
        it({
          name: 'names a worktree of a bare repository own Git directory and the shared one',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A repository with one commit, cloned bare.
             */
            const source = join(folder.path, 'source',);
            await makeRepository({ folder: source, },);
            const bare = join(folder.path, 'bare.git',);
            await gitIn({ folder: folder.path, words: ['clone', '--bare', source, bare,], });
            /**
             The worktree of the bare repository, made detached.
             */
            const worktree = join(folder.path, 'bare-worktree',);
            await gitIn({
              folder: folder.path,
              words: ['--git-dir', bare, 'worktree', 'add', '--detach', worktree,],
            },);
            await expectWhatTheUnitMayWrite({
              checkoutFolder: worktree,
              worktree: join(worktree, '.claude', 'worktrees', 'pi-a',),
              runFolder: join(folder.path, 'runs', 'a-1',),
            },);
          },
        },),
        it({
          name: 'names the Git directory of a submodule, which no commondir file describes',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The repository the submodule belongs to.
             */
            const host = join(folder.path, 'host',);
            await makeRepository({ folder: host, },);
            /**
             The repository a submodule is made from.
             */
            const source = join(folder.path, 'sub-source',);
            await makeRepository({ folder: source, },);
            await gitIn({
              folder: host,
              words: ['-c', 'protocol.file.allow=always', 'submodule', 'add', source, 'modules/s1',],
            },);
            /**
             The submodule's own checkout, which a caller can name with --checkout-folder.
             */
            const submodule = join(host, 'modules', 's1',);
            await expectWhatTheUnitMayWrite({
              checkoutFolder: submodule,
              worktree: join(submodule, '.claude', 'worktrees', 'pi-a',),
              runFolder: join(folder.path, 'runs', 'a-1',),
            },);
          },
        },),
        it({
          name: 'names the Git directory GIT_DIR points at, when that folder does not hold the checkout',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout the start names.
             */
            const checkout = join(folder.path, 'checkout',);
            await makeRepository({ folder: checkout, },);
            /**
             Another repository, whose Git directory GIT_DIR names.
             */
            const elsewhere = join(folder.path, 'elsewhere',);
            await makeRepository({ folder: elsewhere, },);
            await expectWhatTheUnitMayWrite({
              checkoutFolder: checkout,
              worktree: join(checkout, '.claude', 'worktrees', 'pi-a',),
              runFolder: join(folder.path, 'runs', 'a-1',),
              environment: { GIT_DIR: join(elsewhere, '.git',), },
            },);
          },
        },),
        it({
          name: 'refuses a GIT_DIR that names a folder holding the checkout, rather than granting the checkout parent',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The folder that holds the checkout, made a Git directory of its own by GIT_DIR.
             */
            await gitIn({ folder: folder.path, words: ['init', '--bare', '--initial-branch=main', folder.path,], });
            /**
             The checkout below it.
             */
            const checkout = join(folder.path, 'checkout',);
            await makeRepository({ folder: checkout, },);
            try {
              await fileSystemOfARun({
                git: async function answeredByThisTest(request,) {
                  return await runCommandWithChildProcess({
                    command: 'git',
                    words: request.words,
                    folder: request.folder,
                    environment: {
                      ...GIT_ENVIRONMENT,
                      GIT_DIR: folder.path,
                    },
                  },);
                },
                checkoutFolder: checkout,
                worktree: join(checkout, '.claude', 'worktrees', 'pi-a',),
                runFolder: join(folder.path, 'runs', 'a-1',),
                worksInTheCheckoutItself: false,
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(GitFolderAboveTheCheckoutError,);
              expect(String(error,),).toContain(folder.path,);
              expect(String(error,),).toContain('--checkout-folder',);
              return;
            }
            throw new Error('a Git directory holding the checkout was granted writable',);
          },
        },),
        it({
          name: 'lets a run commit in a worktree of a bare repository, whose Git directory is the checkout and so cannot be read-only',
          fn: async () => {
            if (NO_MANAGER_REASON !== undefined) {
              console.log(`[fileSystemOfARun] a commit in a bare repository: skipped: ${NO_MANAGER_REASON}`,);
              return;
            }
            await using folder = await makeFolder();
            /**
             A repository with one commit, cloned bare, so a worktree of it can be made.
             */
            const source = join(folder.path, 'source',);
            await makeRepository({ folder: source, },);
            const bare = join(folder.path, 'bare.git',);
            await gitIn({ folder: folder.path, words: ['clone', '--bare', source, bare,], });
            /**
             The worktree the run works in.
             */
            const worktree = join(bare, '.claude', 'worktrees', 'pi-a',);
            await mkdir(join(bare, '.claude', 'worktrees',), { recursive: true, },);
            await gitIn({ folder: folder.path, words: ['--git-dir', bare, 'worktree', 'add', '-b', 'agent-branch', worktree, 'main',], });
            /**
             The run's own folder.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            await makeTheFoldersAToolMakesIn({ checkoutFolder: bare, },);
            /**
             What the run's unit may and may not write, whose Git folders Git names as the bare repository itself.
             */
            const fileSystem = await fileSystemOfARun({
              git,
              checkoutFolder: bare,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            // A positive control first: the same commit outside the unit lands, so the unit's answer is the namespace's.
            await gitIn({
              folder: worktree,
              words: ['-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', '-c', 'commit.gpgSign=false', 'commit', '--allow-empty', '--message', 'control',],
            },);
            /**
             What the same commit answered inside the unit this package's folders build.
             */
            const ended = await inAUnitOfTheRun({
              unit: `probe-bare-commit-${String(process.pid,)}`,
              fileSystem,
              words: [
                'git',
                '-C',
                worktree,
                '-c',
                'user.name=Disposable',
                '-c',
                'user.email=disposable@example.invalid',
                '-c',
                'commit.gpgSign=false',
                'commit',
                '--allow-empty',
                '--message',
                'a commit the run keeps',
              ],
            },);
            expect(ended.status,).toBe(0,);
            // The commit is on the branch, which is what says the run's work is kept.
            expect(await gitIn({
              folder: bare,
              words: ['log', '--format=%s', '-1', 'agent-branch',],
            },),).toBe('a commit the run keeps',);
          },
        },),
      ],
    },),
  ],
},);
