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
  foldersAToolMakesIn,
  gitRunnerOf,
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
      await rm(path, { recursive: true, force: true, },);
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
  expect(fileSystem.foldersAUnitCannotWrite,).toEqual([checkoutFolder,],);
  for (const answered of [answers.gitDirectory, answers.commonDirectory,]) {
    if (answered === checkoutFolder) {
      continue;
    }
    expect(fileSystem.foldersAUnitMustWrite,).toContain(answered,);
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
      ],
    },),
  ],
},);
