/**
 The file system a run's unit gets: the checkout kept read-only around the folders the run must write, read from the checkout the
 launcher works in and from Git's own record of it. The test writes checkouts of its own in a throwaway folder, so no real checkout
 is read.

 The paths are checked as the `--property=` words they become, because that is where a checkout path with awkward characters would
 otherwise be read by systemd as several paths, or as a broken one, and refuse the whole unit.

 @module
 */

import {
  access,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { execFile, } from 'node:child_process';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import { promisify, } from 'node:util';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  FolderAToolMakesCannotBeMadeError,
  type GitRunner,
  GitFoldersUnreadableError,
  WorktreeGoneError,
  ACanonicalPathThatDidNotAnswerError,
  fileSystemOfARun,
  fileSystemPropertiesOf,
  foldersAToolMakesIn,
  gitFoldersTheFilesName,
  makeTheFoldersAToolMakesIn,
  refuseUnlessTheWorktreeIsThere,
} from '../dist/final/node/index.mjs';

/**
 A git that answers a checkout's two Git directories as Git would, for a fixture whose layout a test knows.

 @param request - the Git directory of the checkout and the one every worktree of it shares

 @param request.gitDirectory - what `--absolute-git-dir` answers

 @param request.commonDirectory - what `--git-common-dir` answers

 @returns a runner that answers that, and refuses anything else

 @example
 ```ts
 const git = gitAnswering({ gitDirectory: '/repo/.git', commonDirectory: '/repo/.git', });
 ```
 */
function gitAnswering({
  gitDirectory,
  commonDirectory,
}: {
  readonly gitDirectory: string;
  readonly commonDirectory: string;
},): GitRunner {
  return async function answered(request,) {
    if (request.words.slice(0, 2,).join(' ',) !== 'rev-parse --path-format=absolute') {
      return {
        status: 1,
        stdout: '',
        stderr: `a test's git answers no ${request.words.join(' ',)}`,
      };
    }
    /**
     Which of the two paths was asked for, which Git answers one at a time: two answers in one stream cannot be told apart from a
     pathname that holds a newline.
     */
    const asked = request.words.slice(2,).join(' ',);
    if (asked === '--absolute-git-dir') {
      return { status: 0, stdout: `${gitDirectory}\n`, stderr: '', };
    }
    if (asked === '--git-common-dir') {
      return { status: 0, stdout: `${commonDirectory}\n`, stderr: '', };
    }
    return {
      status: 1,
      stdout: '',
      stderr: `a test's git answers no ${asked}`,
    };
  };
}

/**
 `execFile` with a promise result, typed through Node's own adapter contract.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFile,);

/**
 What a call answered, or what it threw, so a call that may refuse is read without a promise method.

 @param call - the call

 @returns its answer, or the error it threw

 @example
 ```ts
 const answer = await settledBy({ call: async () => fileSystemOfARun({ ... }), });
 ```
 */
async function settledBy<Answer>({ call, }: { readonly call: () => Promise<Answer> },): Promise<{
  readonly value?: Answer;
  readonly error?: unknown;
}> {
  try {
    return { value: await call(), };
  }
  catch (error) {
    return { error, };
  }
}

/**
 Waits longer than a read of a marker should ever take, and answers what it waited for.

 @param request - how long to wait

 @param request.milliseconds - the bound

 @returns why the caller stopped waiting

 @example
 ```ts
 const answer = await waitForTheTest({ milliseconds: 2_000, });
 ```
 */
async function waitForTheTest({ milliseconds, }: { readonly milliseconds: number },): Promise<{ readonly stillWaiting: true }> {
  await wait(milliseconds,);
  return { stillWaiting: true, };
}

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

 @returns the folder

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<AThrowawayFolder> {
  /**
   The folder's path.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-run-file-system-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Makes the checkout a run was started from, with the Git directory Git keeps in it.

 @param request - where the checkout goes and what it is called

 @param request.folder - the throwaway folder it goes in

 @param request.name - the checkout's folder name, which may carry awkward characters

 @returns the checkout and the Git directory every worktree of it shares

 @example
 ```ts
 const made = await makeCheckout({ folder: '/w', name: 'checkout', });
 ```
 */
async function makeCheckout({
  folder,
  name,
}: {
  readonly folder: string;
  readonly name: string;
},): Promise<{
  readonly checkoutFolder: string;
  readonly commonDirectory: string;
}> {
  /**
   The checkout the run was started from.
   */
  const checkoutFolder = join(
    folder,
    name,
  );
  /**
   The Git directory every worktree of that checkout shares.
   */
  const commonDirectory = join(
    checkoutFolder,
    '.git',
  );
  await mkdir(
    commonDirectory,
    { recursive: true, },
  );
  return {
    checkoutFolder,
    commonDirectory,
  };
}

/**
 Makes the worktree a run works in, as Git writes it: a folder inside the checkout whose `.git` is a file naming the Git directory
 Git gave it, which names the Git directory every worktree shares.

 @param request - the checkout, its Git directory and the name Git gave the worktree

 @param request.checkoutFolder - the checkout the worktree belongs to

 @param request.commonDirectory - the Git directory every worktree of it shares

 @param request.name - the worktree's name

 @returns where the run works

 @example
 ```ts
 const worktree = await makeWorktree({ checkoutFolder: '/repo', commonDirectory: '/repo/.git', name: 'pi-a', });
 ```
 */
async function makeWorktree({
  checkoutFolder,
  commonDirectory,
  name,
}: {
  readonly checkoutFolder: string;
  readonly commonDirectory: string;
  readonly name: string;
},): Promise<string> {
  /**
   Where the run works.
   */
  const worktree = join(
    checkoutFolder,
    '.claude',
    'worktrees',
    name,
  );
  /**
   The Git directory Git gave the worktree.
   */
  const gitDirectory = join(
    commonDirectory,
    'worktrees',
    name,
  );
  await mkdir(
    gitDirectory,
    { recursive: true, },
  );
  await mkdir(
    worktree,
    { recursive: true, },
  );
  await writeFile(
    join(gitDirectory, 'commondir',),
    '../..\n',
  );
  await writeFile(
    join(worktree, '.git',),
    `gitdir: ${gitDirectory}\n`,
  );
  return worktree;
}

/**
 Makes the run's own folder.

 @param folder - the throwaway folder it goes in

 @returns the run's folder

 @example
 ```ts
 const runFolder = await makeRunFolder({ folder: '/w', });
 ```
 */
async function makeRunFolder({ folder, }: { readonly folder: string },): Promise<string> {
  /**
   The run's own folder.
   */
  const runFolder = join(
    folder,
    'runs',
    'a-1',
  );
  await mkdir(
    runFolder,
    { recursive: true, },
  );
  return runFolder;
}

/**
 The folders a run with a worktree may not write: the checkout, and the entries of its shared Git directory a run never writes,
 each as systemd reads it.

 @param request - the checkout and the Git directory every worktree of it shares

 @param request.checkoutFolder - the checkout a start names

 @param request.commonDirectory - the Git directory every worktree of the checkout shares

 @returns the folders, in the order this package names them

 @example
 ```ts
 const folders = readOnlyFoldersOf({ checkoutFolder: '/repo', commonDirectory: '/repo/.git', });
 ```
 */
function readOnlyFoldersOf({
  checkoutFolder,
  commonDirectory,
}: {
  readonly checkoutFolder: string;
  readonly commonDirectory: string;
},): readonly string[] {
  return [
    checkoutFolder,
    ...['HEAD', 'index', 'config', 'config.worktree', 'hooks', 'info', 'worktrees',].map(function inside(entry,) {
      return `-${join(commonDirectory, entry,)}`;
    },),
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: fileSystemOfARun.name,
      children: [
        it({
          name: 'keeps the checkout read-only and makes the run worktree, its Git directory and the run folder writable',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout the run was started from.
             */
            const made = await makeCheckout({ folder: folder.path, name: 'checkout', },);
            /**
             Where the run works.
             */
            const worktree = await makeWorktree({
              ...made,
              name: 'pi-a',
            },);
            /**
             The run's own folder.
             */
            const runFolder = await makeRunFolder({ folder: folder.path, },);
            /**
             What the run's unit may and may not write.
             */
            const fileSystem = await fileSystemOfARun({
              git: gitAnswering({
                gitDirectory: made.commonDirectory,
                commonDirectory: made.commonDirectory,
              },),
              checkoutFolder: made.checkoutFolder,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            expect(fileSystem.foldersAUnitCannotWrite,).toEqual(readOnlyFoldersOf({
              checkoutFolder: made.checkoutFolder,
              commonDirectory: made.commonDirectory,
            },),);
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              worktree,
              made.commonDirectory,
              ...foldersAToolMakesIn({ checkoutFolder: made.checkoutFolder, },),
              runFolder,
            ],);
          },
        },),
        it({
          name: 'makes neither a worktree nor a Git folder writable for a run that works in the checkout itself, whose files and refs it must not change',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout the run works in.
             */
            const made = await makeCheckout({ folder: folder.path, name: 'checkout', },);
            /**
             The run's own folder.
             */
            const runFolder = await makeRunFolder({ folder: folder.path, },);
            /**
             What the run's unit may and may not write.
             */
            const fileSystem = await fileSystemOfARun({
              git: gitAnswering({
                gitDirectory: made.commonDirectory,
                commonDirectory: made.commonDirectory,
              },),
              checkoutFolder: made.checkoutFolder,
              worktree: made.checkoutFolder,
              runFolder,
              worksInTheCheckoutItself: true,
            },);
            expect(fileSystem.foldersAUnitCannotWrite,).toEqual([made.checkoutFolder,],);
            // The checkout itself is read-only in this mode, so nothing of its Git data is writable either: a writable `.git`
            // would let a run that cannot change one file move `main`, which is what a review reproduced.
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              ...foldersAToolMakesIn({ checkoutFolder: made.checkoutFolder, },),
              runFolder,
            ],);
            expect(fileSystem.foldersAUnitMustWrite,).not.toContain(made.checkoutFolder,);
            expect(fileSystem.foldersAUnitMustWrite,).not.toContain(made.commonDirectory,);
          },
        },),
        it({
          name: 'names both Git directories of a checkout whose own Git directory lives elsewhere, such as a linked worktree named with --checkout-folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The repository the Git directories belong to.
             */
            const repository = join(
              folder.path,
              'repository',
            );
            /**
             The Git directory every worktree of that repository shares.
             */
            const commonDirectory = join(
              repository,
              '.git',
            );
            await mkdir(
              commonDirectory,
              { recursive: true, },
            );
            /**
             The checkout the run was started from, whose marker is a file naming a Git directory elsewhere.
             */
            const checkoutFolder = join(
              folder.path,
              'checkout',
            );
            /**
             The Git directory Git gave that checkout.
             */
            const gitDirectory = join(
              commonDirectory,
              'worktrees',
              'checkout',
            );
            await mkdir(
              gitDirectory,
              { recursive: true, },
            );
            await mkdir(
              checkoutFolder,
              { recursive: true, },
            );
            await writeFile(
              join(gitDirectory, 'commondir',),
              '../..\n',
            );
            await writeFile(
              join(checkoutFolder, '.git',),
              `gitdir: ${gitDirectory}\n`,
            );
            /**
             The run's own folder.
             */
            const runFolder = await makeRunFolder({ folder: folder.path, },);
            /**
             What the run's unit may and may not write.
             */
            const fileSystem = await fileSystemOfARun({
              git: gitAnswering({
                gitDirectory,
                commonDirectory,
              },),
              checkoutFolder,
              worktree: checkoutFolder,
              runFolder,
              worksInTheCheckoutItself: true,
            },);
            // Asked about a checkout with its own Git directory elsewhere, and with no worktree of its own to commit in: the two
            // Git directories stay out, which is the same rule as above with folders other than `.git`.
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              ...foldersAToolMakesIn({ checkoutFolder, },),
              runFolder,
            ],);
          },
        },),
        it({
          name: 'stops waiting for a folder the file system never names inside its bound, instead of holding the start before any unit exists',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout and the run's own folders, which exist so that nothing but the path reading can fail.
             */
            const made = await makeCheckout({ folder: folder.path, name: 'checkout', },);
            const worktree = await makeWorktree({
              ...made,
              name: 'pi-a',
            },);
            const runFolder = await makeRunFolder({ folder: folder.path, },);
            /**
             How long a folder's own reading may take here, short enough that a test can wait it out.
             */
            const patience = 50;
            /**
             A file system that never names a folder, which is what a mount that has stopped answering looks like.
             */
            async function neverAnswers(): Promise<string> {
              return await new Promise<string>(function waitForEver(resolve,) {
                // Nothing resolves it, which is what a file system that has stopped answering looks like.
                void resolve;
              },);
            }
            /**
             What asking for the run's folders answered.
             */
            const asked = await settledBy({
              call: async function readTheFolders() {
                return await fileSystemOfARun({
                  git: gitAnswering({
                    gitDirectory: made.commonDirectory,
                    commonDirectory: made.commonDirectory,
                  },),
                  checkoutFolder: made.checkoutFolder,
                  worktree,
                  runFolder,
                  worksInTheCheckoutItself: false,
                  readRealPath: neverAnswers,
                  canonicalPathMilliseconds: patience,
                },);
              },
            },);
            // The start refused in its own words rather than waiting for a file system that may never answer: no unit exists at
            // this point, so nothing else bounds this wait.
            expect(asked.error,).toBeInstanceOf(ACanonicalPathThatDidNotAnswerError,);
            /**
             What the refusal says, which names the folder and the bound.
             */
            const refusal = (asked.error as Error).message;
            expect(refusal,).toContain(made.checkoutFolder,);
            expect(refusal,).toContain(`longer than ${String(patience,)} ms`,);
          },
        },),
        it({
          name: 'refuses a checkout Git cannot describe, naming the folder, rather than guessing writable folders for it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A checkout whose Git cannot answer about its directories.
             */
            const checkoutFolder = join(
              folder.path,
              'checkout',
            );
            await mkdir(checkoutFolder, { recursive: true, },);
            /**
             Where the run works and where its driver writes.
             */
            const worktree = join(
              checkoutFolder,
              '.claude',
              'worktrees',
              'pi-a',
            );
            await mkdir(worktree, { recursive: true, },);
            const runFolder = await makeRunFolder({ folder: folder.path, },);
            try {
              await fileSystemOfARun({
                git: async function refuses() {
                  return {
                    status: 128,
                    stdout: '',
                    stderr: 'fatal: not a git repository\n',
                  };
                },
                checkoutFolder,
                worktree,
                runFolder,
                worksInTheCheckoutItself: false,
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(GitFoldersUnreadableError,);
              expect(String(error,),).toContain(checkoutFolder,);
              expect(String(error,),).toContain('fatal: not a git repository',);
              expect(String(error,),).toContain('--checkout-folder',);
              return;
            }
            throw new Error('a checkout Git cannot describe answered with folders a unit may write',);
          },
        },),
      ],
    },),
    describe({
      name: fileSystemPropertiesOf.name,
      children: [
        it({
          name: 'writes each folder as the systemd property it belongs to, the read-only ones first',
          fn: async () => {
            expect(fileSystemPropertiesOf({
              foldersAUnitCannotWrite: ['/repo',],
              foldersAUnitMustWrite: ['/repo/.git', '/repo/.cache',],
            },),).toEqual([
              '--property=ReadOnlyPaths="/repo"',
              '--property=ReadWritePaths="/repo/.git"',
              '--property=ReadWritePaths="/repo/.cache"',
            ],);
          },
        },),
        it({
          name: 'quotes a checkout folder, a worktree and a run folder that carry a space or a single quote, so systemd reads each as the one path it is',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout the run was started from, whose own name carries a space systemd would otherwise split on.
             */
            const made = await makeCheckout({
              folder: folder.path,
              name: 'my checkout',
            },);
            /**
             Where the run works, whose name carries a single quote systemd would otherwise read as the start of a quoted item.
             */
            const worktree = await makeWorktree({
              ...made,
              name: "pi-a's",
            },);
            /**
             The run's own folder.
             */
            const runFolder = await makeRunFolder({ folder: folder.path, },);
            /**
             The folder this repository's tools install themselves into, which the launcher made before this and which the unit
             must write.
             */
            const cacheFolder = join(
              made.checkoutFolder,
              '.cache',
            );
            /**
             What the run's unit may and may not write.
             */
            const fileSystem = await fileSystemOfARun({
              git: gitAnswering({
                gitDirectory: made.commonDirectory,
                commonDirectory: made.commonDirectory,
              },),
              checkoutFolder: made.checkoutFolder,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            expect(fileSystemPropertiesOf(fileSystem,),).toEqual([
              `--property=ReadOnlyPaths="${made.checkoutFolder}"`,
              ...['HEAD', 'index', 'config', 'config.worktree', 'hooks', 'info', 'worktrees',].map(function inside(entry,) {
                return `--property=ReadOnlyPaths=-"${join(made.commonDirectory, entry,)}"`;
              },),
              `--property=ReadWritePaths="${worktree}"`,
              `--property=ReadWritePaths="${made.commonDirectory}"`,
              `--property=ReadWritePaths="${cacheFolder}"`,
              `--property=ReadWritePaths="${runFolder}"`,
            ],);
          },
        },),
        it({
          name: 'writes no word at all when it is given no folder',
          fn: async () => {
            expect(fileSystemPropertiesOf({
              foldersAUnitCannotWrite: [],
              foldersAUnitMustWrite: [],
            },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: makeTheFoldersAToolMakesIn.name,
      children: [
        it({
          name: 'makes the checkout cache a run tools install themselves into, which systemd cannot make writable by itself when it is absent',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout the run was started from, which has no cache yet.
             */
            const made = await makeCheckout({ folder: folder.path, name: 'checkout', },);
            /**
             The folders a tool makes inside it.
             */
            const folders = foldersAToolMakesIn({ checkoutFolder: made.checkoutFolder, },);
            expect(folders,).toEqual([join(
              made.checkoutFolder,
              '.cache',
            ),],);
            await expect(access(folders[0] ?? '',),).rejects.toThrow('ENOENT',);
            expect(await makeTheFoldersAToolMakesIn({ checkoutFolder: made.checkoutFolder, },),).toEqual(folders,);
            await expect(access(folders[0] ?? '',),).resolves.toBeUndefined();
            // Asked again, which a start after a refused one does, it answers the same and makes nothing twice.
            expect(await makeTheFoldersAToolMakesIn({ checkoutFolder: made.checkoutFolder, },),).toEqual(folders,);
          },
        },),
        it({
          name: 'refuses when the folder cannot be made, naming it and what a run would lose',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A checkout whose own folder is a file, so nothing can be made inside it.
             */
            const checkout = join(
              folder.path,
              'checkout',
            );
            await writeFile(checkout, 'not a folder\n',);
            try {
              await makeTheFoldersAToolMakesIn({ checkoutFolder: checkout, },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(FolderAToolMakesCannotBeMadeError,);
              expect(String(error,),).toContain(join(
                checkout,
                '.cache',
              ),);
              expect(String(error,),).toContain('read-only',);
              return;
            }
            throw new Error("a checkout whose cache cannot be made answered with folders",);
          },
        },),
      ],
    },),
    describe({
      name: refuseUnlessTheWorktreeIsThere.name,
      children: [
        it({
          name: 'says the worktree is there and answers nothing, so a start that can go ahead is not refused',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A worktree that is there.
             */
            const worktree = join(
              folder.path,
              'worktree',
            );
            await mkdir(
              worktree,
              { recursive: true, },
            );
            await expect(refuseUnlessTheWorktreeIsThere({
              worktree,
              runName: 'a',
              runFolder: join(folder.path, 'runs', 'a-1',),
            },),).resolves.toBeUndefined();
          },
        },),
        it({
          name: 'refuses a start whose worktree is gone, naming the worktree, the launcher that most likely removed it and the run folder',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A worktree a launcher of another start of this name would have removed.
             */
            const worktree = join(
              folder.path,
              'worktree',
            );
            /**
             The run's own folder, which the launcher made before it asked.
             */
            const runFolder = join(
              folder.path,
              'runs',
              'a-1',
            );
            try {
              await refuseUnlessTheWorktreeIsThere({
                worktree,
                runName: 'a',
                runFolder,
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(WorktreeGoneError,);
              expect(String(error,),).toContain(worktree,);
              expect(String(error,),).toContain('another start of the name a',);
              expect(String(error,),).toContain(runFolder,);
              expect(String(error,),).toContain('Start the run again',);
              // What is kept is the branch and its commits; nothing claims the worktree's uncommitted files survived.
              expect(String(error,),).toContain('what the worktree held that was never committed, including the files Git ignores, is not there any more',);
              expect(String(error,),).not.toContain('Nothing of that work is lost',);
              return;
            }
            throw new Error('a start whose worktree is gone was not refused',);
          },
        },),
      ],
    },),
    describe({
      name: 'a checkout whose Git marker is a named pipe',
      children: [
        it({
          name: 'is refused at once by the walk that reads Git files, naming the marker, instead of waiting for a writer that may never come',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A checkout whose marker a concurrent command replaced with a named pipe.
             */
            const checkout = join(
              folder.path,
              'checkout',
            );
            await mkdir(checkout,);
            await execFileAsync('mkfifo', [join(checkout, '.git',),], { encoding: 'utf8', timeout: 5_000, },);
            /**
             What the walk answered, or the fact that it was still waiting when the test stopped waiting.
             */
            const answer = await Promise.race([
              settledBy({
                call: async function read() {
                  return await gitFoldersTheFilesName({ folder: checkout, },);
                },
              },),
              waitForTheTest({ milliseconds: 2_000, },),
            ],);
            expect(answer,).toHaveProperty('error',);
            expect(String((answer as { readonly error?: unknown; }).error,),).toContain(join(
              checkout,
              '.git',
            ),);
            expect(String((answer as { readonly error?: unknown; }).error,),).toContain('not an ordinary file',);
          },
        },),
      ],
    },),
  ],
},);
