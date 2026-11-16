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
import { basename, join, } from 'node:path';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  ENTRIES_A_RUN_NEVER_WRITES,
  GitFolderAboveTheCheckoutError,
  fileSystemOfARun,
  fileSystemPropertiesOf,
  foldersAToolMakesIn,
  gitFoldersOf,
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
  // The Git directory the fixture's worktree would have been given, made here because these fixtures name a worktree path
  // without making Git make one, and this package reads that directory as the file system names it.
  await mkdir(join(answers.commonDirectory, 'worktrees', basename(worktree,),), { recursive: true, },);
  /**
   What this package names for the same checkout.
   */
  const fileSystem = await fileSystemOfARun({
    git: async function answeredByThisTest(request,) {
      // The two questions this package asks: the checkout's own Git directories, and the ones the run's worktree was given, which
      // is the worktree Git directory inside the shared one that a fixture of this shape always has.
      /**
       What this question is about.
       */
      const name = request.words.at(-1,);
      /**
       The folder the answer is about, which is the worktree for its own question and the checkout for the other.
       */
      const answered = (request.folder === worktree)
        ? join(answers.commonDirectory, 'worktrees', basename(worktree,),)
        : ((name === '--git-common-dir') ? answers.commonDirectory : answers.gitDirectory);
      return { status: 0, stdout: `${answered}\n`, stderr: '', };
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
  // read-only view would stop the commit that keeps the run's work. Beside it sit the entries of the shared Git directory a run
  // never writes, each joined onto that directory and marked optional with a leading dash.
  /**
   The read-only folders that name the checkout itself.
   */
  const theCheckoutItself = fileSystem.foldersAUnitCannotWrite.filter(function isTheCheckout(folder,) {
    return folder === checkoutFolder;
  },);
  expect(theCheckoutItself.length,).toBe(([answers.gitDirectory, answers.commonDirectory,].includes(checkoutFolder)) ? 0 : 1,);
  for (const readOnly of fileSystem.foldersAUnitCannotWrite) {
    expect([answers.gitDirectory, answers.commonDirectory,],).not.toContain(readOnly,);
    expect(ENTRIES_A_RUN_NEVER_WRITES.map(function joined(entry,) {
      return `-${join(answers.commonDirectory, entry,)}`;
    },).includes(readOnly,) || (readOnly === checkoutFolder),).toBe(true,);
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
        it({
          name: 'refuses every write to an entry of the shared Git directory a run never writes, in a real unit, while its own worktree Git directory takes one',
          fn: async () => {
            if (NO_MANAGER_REASON !== undefined) {
              console.log(`[fileSystemOfARun] the shared Git directory entries at a real unit: skipped: ${NO_MANAGER_REASON}`,);
              return;
            }
            await using folder = await makeFolder();
            /**
             A checkout with a commit, the run's worktree, and another worktree playing a second run's.
             */
            const checkout = join(folder.path, 'checkout',);
            await makeRepository({ folder: checkout, },);
            // The checkout's own index, which a real checkout always has and which this test needs to exist for its read-only mount
            // to be there at all: a read-only path that does not exist is skipped by systemd, and the entry could then be created.
            await writeFile(join(checkout, 'tracked.txt',), 'tracked\n',);
            await gitIn({ folder: checkout, words: ['add', '--', 'tracked.txt',], });
            const worktree = join(checkout, '.claude', 'worktrees', 'pi-a',);
            const otherWorktree = join(checkout, '.claude', 'worktrees', 'pi-b',);
            await mkdir(join(checkout, '.claude', 'worktrees',), { recursive: true, },);
            await gitIn({ folder: checkout, words: ['worktree', 'add', '-q', '-b', 'fix/run-a', worktree,], });
            await gitIn({ folder: checkout, words: ['worktree', 'add', '-q', '-b', 'fix/run-b', otherWorktree,], });
            /**
             The per-worktree configuration, which a repository with `extensions.worktreeConfig` has and which this test needs to exist
             for its read-only mount to be there at all.
             */
            await writeFile(join(checkout, '.git', 'config.worktree',), '[probe]\n\tfrom = the fixture\n',);
            /**
             The shared Git directory, and the run's own Git directory inside it.
             */
            const shared = join(checkout, '.git',);
            const ownGitDirectory = join(shared, 'worktrees', 'pi-a',);
            /**
             The run's own folder.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            await makeTheFoldersAToolMakesIn({ checkoutFolder: checkout, },);
            /**
             What the run's unit may and may not write, in author mode, which is the mode that commits.
             */
            const fileSystem = await fileSystemOfARun({
              git,
              checkoutFolder: checkout,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            /**
             One write per entry a run never writes, at the path the entry is, and one at the same name inside the run's own worktree Git
             directory, which a commit does write.
             */
            const entries: readonly (readonly [string, string, boolean])[] = [
              ['HEAD', join(shared, 'HEAD',)],
              ['index', join(shared, 'index',)],
              ['config.worktree', join(shared, 'config.worktree',)],
              ['hooks/pre-commit', join(shared, 'hooks', 'pre-commit',)],
              ['info/exclude', join(shared, 'info', 'exclude',)],
              ['another worktree HEAD', join(shared, 'worktrees', 'pi-b', 'HEAD',)],
              ['the run own worktree HEAD', join(ownGitDirectory, 'HEAD',)],
            ].map(function withWhetherItIsTheRunsOwn(entry,) {
              const [name, path,] = entry as unknown as readonly [string, string];
              return [name, path, path === join(ownGitDirectory, 'HEAD',),] as const;
            },);
            /**
             What each write answered, keyed by the entry's name: one unit of the run's own per entry, appending to that one path.
             */
            const answers = Object.fromEntries(
              await Promise.all(entries.map(async function writeOne(entry,) {
                const [name, path,] = entry;
                const ended = await inAUnitOfTheRun({
                  unit: `probe-git-entry-${String(process.pid,)}-${name.replaceAll('/', '-',).replaceAll(' ', '-',)}`,
                  fileSystem,
                  words: ['sh', '-c', `printf 'written by the run\\n' >> '${path}'`,],
                },);
                return [name, (ended.status === 0) ? 'written' : `refused: ${ended.output.slice(0, 60)}`,] as const;
              },),),
            );
            // Every entry a run never writes refused the write, and the run's own worktree Git directory took one: the positive
            // control that says the refusals are the grant and not a unit that refuses everything.
            expect(
              Object.fromEntries(Object.entries(answers,).map(function whetherItWasWritten(entry,) {
                const [name, answer,] = entry;
                return [name, answer === 'written',];
              },),),
            ).toEqual({
              HEAD: false,
              index: false,
              'config.worktree': false,
              'hooks/pre-commit': false,
              'info/exclude': false,
              'another worktree HEAD': false,
              'the run own worktree HEAD': true,
            },);
            // The refusals are the operating system's, and each names the entry it refused.
            expect(Object.entries(answers,).filter(function isNotRefused(entry,) {
              const [name, answer,] = entry;
              return (answer !== 'written') && (!answer.startsWith('refused:'));
            },),).toEqual([],);
            expect(await gitIn({ folder: checkout, words: ['log', '--format=%s', '-1',], },),).toBe('base',);
          },
        },),
        it({
          name: 'asks Git for each Git directory on its own, so a checkout whose path holds a newline or ends in a space is still named',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             Checkouts whose own names carry what a reader of two answers in one stream cannot tell apart: a newline, and a trailing
             space that trimming would eat.
             */
            const awkwardNames = [
              'checkout\nline-break',
              'checkout-with-a-trailing-space ',
            ];
            /**
             What this package names for each of those checkouts, keyed by its name.
             */
            const named = await Promise.all(awkwardNames.map(async function nameOne(name,) {
              /**
               The checkout itself, which is a real repository named that way.
               */
              const checkout = join(folder.path, name,);
              await mkdir(checkout, { recursive: true, },);
              // Nine answers cannot be told from four in a repository whose path holds a line feed, and this repository's own git
              // wrapper refuses to classify one: a commit there is another tool's limit, and none of this package needs one to name
              // the folders it writes.
              await gitIn({ folder: checkout, words: ['init', '--initial-branch=main',], });
              /**
               What this package names for its two Git directories, compared with the paths the fixture made rather than with a
               reader that splits two answers by newline, which is the reading this test exists to replace.
               */
              return [name, await gitFoldersOf({ git, folder: checkout, },),];
            },),);
            expect(Object.fromEntries(named,),).toEqual({
              'checkout\nline-break': {
                gitDirectory: join(folder.path, 'checkout\nline-break', '.git',),
                commonDirectory: join(folder.path, 'checkout\nline-break', '.git',),
              },
              'checkout-with-a-trailing-space ': {
                gitDirectory: join(folder.path, 'checkout-with-a-trailing-space ', '.git',),
                commonDirectory: join(folder.path, 'checkout-with-a-trailing-space ', '.git',),
              },
            },);
          },
        },),
        it({
          name: 'refuses a run that works in the checkout itself every write to its Git data, and still lets an author commit in a worktree of it',
          fn: async () => {
            if (NO_MANAGER_REASON !== undefined) {
              console.log(`[fileSystemOfARun] the two modes at a real unit: skipped: ${NO_MANAGER_REASON}`,);
              return;
            }
            await using folder = await makeFolder();
            /**
             A checkout with two commits on `main`, so a run that moves `main` can be seen doing it.
             */
            const checkout = join(folder.path, 'checkout',);
            await makeRepository({ folder: checkout, },);
            await gitIn({
              folder: checkout,
              words: ['-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', 'commit', '--allow-empty', '--message', 'main-only',],
            },);
            /**
             What `main` names before any unit runs, which nothing may move.
             */
            const mainBefore = await gitIn({ folder: checkout, words: ['rev-parse', 'main',], },);
            /**
             The run's own folder.
             */
            const runFolder = join(folder.path, 'runs', 'a-1',);
            await mkdir(runFolder, { recursive: true, },);
            await makeTheFoldersAToolMakesIn({ checkoutFolder: checkout, },);
            /**
             What a run's unit may and may not write when the run works in the checkout itself: the checkout is read-only, and
             nothing of its Git data is writable either.
             */
            const readOnlyInMain = await fileSystemOfARun({
              git,
              checkoutFolder: checkout,
              worktree: checkout,
              runFolder,
              worksInTheCheckoutItself: true,
            },);
            /**
             What the unit answered for each attempt, keyed by what was attempted.
             */
            const attempts: Record<string, { readonly status: number; readonly output: string }> = {};
            /**
             Each attempt, in the unit this package's folders build: a ref moved, a commit made, and a file written into the Git
             directory, all of which a read-only-in-main run must be refused.
             */
            const attemptsInTheUnit: readonly (readonly [string, readonly string[]])[] = [
              ['update-ref', ['git', '-C', checkout, 'update-ref', 'refs/heads/main', mainBefore,],],
              ['commit', ['git', '-C', checkout, '-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', '-c', 'commit.gpgSign=false', 'commit', '--allow-empty', '--message', 'a commit from a read-only run',],],
              ['write-into-the-git-directory', ['sh', '-c', `printf 'written\\n' > '${checkout}/.git/written-by-the-run'`,],],
            ];
            const attemptEntries = await Promise.all(attemptsInTheUnit.map(async function attemptInTheUnit(entry,) {
              const [what, words,] = entry;
              const ended = await inAUnitOfTheRun({
                unit: `probe-read-only-in-main-${what.replaceAll('/', '-',)}-${String(process.pid,)}`,
                fileSystem: readOnlyInMain,
                words,
              },);
              return [what, ended,] as const;
            },),);
            for (const [what, ended,] of attemptEntries) {
              attempts[what] = ended;
            }
            // The operating system refused each write, which is what a run that cannot change the checkout needs.
            expect(
              Object.fromEntries(Object.entries(attempts,).map(function whatHappened(entry,) {
                const [what, ended,] = entry;
                return [what, (ended.status === 0) ? 'allowed' : 'refused',];
              },),),
            ).toEqual({
              'update-ref': 'refused',
              commit: 'refused',
              'write-into-the-git-directory': 'refused',
            },);
            expect(attempts['write-into-the-git-directory']?.output,).toContain('Read-only file system',);
            // Reading the checkout's own Git data still works, because it is readable through the read-only checkout.
            const read = await inAUnitOfTheRun({
              unit: `probe-read-only-in-main-log-${String(process.pid,)}`,
              fileSystem: readOnlyInMain,
              words: ['git', '-C', checkout, 'log', '--format=%s', '-1',],
            },);
            expect(read.status,).toBe(0,);
            expect(read.output.trim(),).toBe('main-only',);
            // `main` is where it was: the ref the unit tried to move is the ref it could not write.
            expect(await gitIn({ folder: checkout, words: ['rev-parse', 'main',], },),).toBe(mainBefore,);

            /**
             The worktree an author's run works in, which Git made.
             */
            const worktree = join(checkout, '.claude', 'worktrees', 'pi-author',);
            await mkdir(join(checkout, '.claude', 'worktrees',), { recursive: true, },);
            await gitIn({ folder: checkout, words: ['worktree', 'add', '-b', 'agent-branch', worktree,], });
            /**
             What an author's unit may and may not write: the checkout is still read-only, and the Git folders a worktree commit
             writes are writable, so the run's work is kept.
             */
            const withAWorktree = await fileSystemOfARun({
              git,
              checkoutFolder: checkout,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            expect(withAWorktree.foldersAUnitMustWrite,).toContain(join(checkout, '.git',),);
            /**
             What an ordinary commit in the author's worktree answered inside the unit.
             */
            const committed = await inAUnitOfTheRun({
              unit: `probe-author-commit-${String(process.pid,)}`,
              fileSystem: withAWorktree,
              words: ['git', '-C', worktree, '-c', 'user.name=Disposable', '-c', 'user.email=disposable@example.invalid', '-c', 'commit.gpgSign=false', 'commit', '--allow-empty', '--message', 'an author commit the run keeps',],
            },);
            expect(committed.status,).toBe(0,);
            // What `git commit` prints when it worked: the branch, the new commit and the message it was given.
            expect(committed.output,).toContain('an author commit the run keeps',);
            expect(await gitIn({ folder: checkout, words: ['log', '--format=%s', '-1', 'agent-branch',], },),).toBe('an author commit the run keeps',);
            // The worktree's own Git directory is a folder inside the checkout's, and the same writable folder covers both: what
            // tells the two modes apart is the mode, not a path. An author's unit writes its own branch's ref by renaming a lock
            // file in `refs/heads`, so the folder holding that ref is writable in this mode and the read-only mode is where a
            // checkout's refs are out of a run's reach.
            expect(await gitIn({ folder: checkout, words: ['rev-parse', '--show-toplevel',], },),).toBe(checkout,);
          },
        },),
      ],
    },),
  ],
},);
