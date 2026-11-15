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
  WorktreeGoneError,
  fileSystemOfARun,
  fileSystemPropertiesOf,
  foldersAToolMakesIn,
  makeTheFoldersAToolMakesIn,
  refuseUnlessTheWorktreeIsThere,
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
              checkoutFolder: made.checkoutFolder,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            expect(fileSystem.foldersAUnitCannotWrite,).toEqual([made.checkoutFolder,],);
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              worktree,
              made.commonDirectory,
              ...foldersAToolMakesIn({ checkoutFolder: made.checkoutFolder, },),
              runFolder,
            ],);
            expect(fileSystem.foldersAUnitMustWriteWhenTheyAreThere,).toEqual([],);
          },
        },),
        it({
          name: 'makes no worktree writable for a run that works in the checkout itself, whose files it must not change',
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
              checkoutFolder: made.checkoutFolder,
              worktree: made.checkoutFolder,
              runFolder,
              worksInTheCheckoutItself: true,
            },);
            expect(fileSystem.foldersAUnitCannotWrite,).toEqual([made.checkoutFolder,],);
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              made.commonDirectory,
              ...foldersAToolMakesIn({ checkoutFolder: made.checkoutFolder, },),
              runFolder,
            ],);
            expect(fileSystem.foldersAUnitMustWrite,).not.toContain(made.checkoutFolder,);
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
              checkoutFolder,
              worktree: checkoutFolder,
              runFolder,
              worksInTheCheckoutItself: true,
            },);
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              gitDirectory,
              commonDirectory,
              ...foldersAToolMakesIn({ checkoutFolder, },),
              runFolder,
            ],);
            expect(fileSystem.foldersAUnitMustWriteWhenTheyAreThere,).toEqual([],);
          },
        },),
        it({
          name: 'names a marker a checkout does not have among the folders that may be absent, so a start is not refused for it',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A checkout that holds no marker at all.
             */
            const checkoutFolder = join(
              folder.path,
              'checkout',
            );
            await mkdir(
              checkoutFolder,
              { recursive: true, },
            );
            /**
             Where the run works.
             */
            const worktree = join(
              checkoutFolder,
              '.claude',
              'worktrees',
              'pi-a',
            );
            await mkdir(
              worktree,
              { recursive: true, },
            );
            /**
             The run's own folder.
             */
            const runFolder = await makeRunFolder({ folder: folder.path, },);
            /**
             What the run's unit may and may not write.
             */
            const fileSystem = await fileSystemOfARun({
              checkoutFolder,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            expect(fileSystem.foldersAUnitCannotWrite,).toEqual([checkoutFolder,],);
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              worktree,
              ...foldersAToolMakesIn({ checkoutFolder, },),
              runFolder,
            ],);
            expect(fileSystem.foldersAUnitMustWriteWhenTheyAreThere,).toEqual([join(
              checkoutFolder,
              '.git',
            ),],);
          },
        },),
      ],
    },),
    describe({
      name: fileSystemPropertiesOf.name,
      children: [
        it({
          name: 'writes each folder as the systemd property it belongs to, the read-only ones first and the ones that may be absent last',
          fn: async () => {
            expect(fileSystemPropertiesOf({
              foldersAUnitCannotWrite: ['/repo',],
              foldersAUnitMustWrite: ['/repo/.git',],
              foldersAUnitMustWriteWhenTheyAreThere: ['/repo/.cache',],
            },),).toEqual([
              '--property=ReadOnlyPaths="/repo"',
              '--property=ReadWritePaths="/repo/.git"',
              '--property=ReadWritePaths=-"/repo/.cache"',
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
              checkoutFolder: made.checkoutFolder,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            expect(fileSystemPropertiesOf(fileSystem,),).toEqual([
              `--property=ReadOnlyPaths="${made.checkoutFolder}"`,
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
              foldersAUnitMustWriteWhenTheyAreThere: [],
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
          name: 'is refused at once, naming the marker, instead of waiting for a writer that may never come before any unit exists',
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
             Where the run works, which this read never reaches.
             */
            const worktree = join(
              checkout,
              '.claude',
              'worktrees',
              'pi-a',
            );
            await mkdir(worktree, { recursive: true, },);
            /**
             What the read answered, or the fact that it was still waiting when the test stopped waiting.
             */
            const answer = await Promise.race([
              settledBy({
                call: async function read() {
                  return await fileSystemOfARun({
                    checkoutFolder: checkout,
                    worktree,
                    runFolder: join(folder.path, 'runs', 'a-1',),
                    worksInTheCheckoutItself: false,
                  },);
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
