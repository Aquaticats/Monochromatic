/**
 The file system a run's unit gets: the checkout kept read-only around the folders the run must write, read from the checkout the
 launcher works in and from Git's own record of it. The test writes checks out of its own in a throwaway folder, so no real
 checkout is read.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  fileSystemOfARun,
  fileSystemPropertiesOf,
} from '../dist/final/node/index.mjs';

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

 @param folder - the throwaway folder it goes in

 @returns the checkout and the Git directory every worktree of it shares

 @example
 ```ts
 const made = await makeCheckout({ folder: '/w', });
 ```
 */
async function makeCheckout({ folder, }: { readonly folder: string },): Promise<{
  readonly checkoutFolder: string;
  readonly commonDirectory: string;
}> {
  /**
   The checkout the run was started from.
   */
  const checkoutFolder = join(
    folder,
    'checkout',
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

 @param request - the checkout and the name Git gave the worktree

 @param request.checkoutFolder - the checkout the worktree belongs to

 @param request.commonDirectory - the Git directory every worktree of it shares

 @param request.name - the worktree's name

 @returns where the run works

 @example
 ```ts
 const worktree = await makeWorktree({ checkoutFolder: '/repo', commonDirectory: '/repo/.git', name: 'pi-a' });
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

await describe({
  name: '',
  children: [
    describe({
      name: fileSystemOfARun.name,
      children: [
        it({
          name: 'keeps the checkout read-only and makes the run worktree, its Git directory, the checkout cache and the run folder writable',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout the run was started from.
             */
            const made = await makeCheckout({ folder: folder.path, },);
            /**
             Where the run works.
             */
            const worktree = await makeWorktree({
              ...made,
              name: 'pi-a',
            },);
            /**
             The folder this repository's tools cache in.
             */
            const cacheFolder = join(
              made.checkoutFolder,
              '.cache',
            );
            /**
             The run's own folder.
             */
            const runFolder = join(
              folder.path,
              'runs',
              'a-1',
            );
            await mkdir(
              runFolder,
              { recursive: true, },
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
            expect(fileSystem.foldersAUnitCannotWrite,).toEqual([made.checkoutFolder,],);
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              worktree,
              made.commonDirectory,
              `-${cacheFolder}`,
              runFolder,
            ],);
          },
        },),
        it({
          name: 'makes no worktree writable for a run that works in the checkout itself, whose files it must not change',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The checkout the run works in.
             */
            const made = await makeCheckout({ folder: folder.path, },);
            /**
             The folder this repository's tools cache in.
             */
            const cacheFolder = join(
              made.checkoutFolder,
              '.cache',
            );
            /**
             The run's own folder.
             */
            const runFolder = join(
              folder.path,
              'runs',
              'a-1',
            );
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
              `-${cacheFolder}`,
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
            const runFolder = join(
              folder.path,
              'runs',
              'a-1',
            );
            /**
             The folder this repository's tools cache in.
             */
            const cacheFolder = join(
              checkoutFolder,
              '.cache',
            );
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
              `-${cacheFolder}`,
              runFolder,
            ],);
          },
        },),
        it({
          name: 'names a marker a checkout does not have as one systemd may ignore, so a start is not refused for it',
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
            /**
             The run's own folder.
             */
            const runFolder = join(
              folder.path,
              'runs',
              'a-1',
            );
            /**
             What the run's unit may and may not write.
             */
            const fileSystem = await fileSystemOfARun({
              checkoutFolder,
              worktree,
              runFolder,
              worksInTheCheckoutItself: false,
            },);
            /**
             The marker the checkout does not have, and the folder this repository's tools cache in.
             */
            const marker = join(
              checkoutFolder,
              '.git',
            );
            /**
             The folder this repository's tools cache in.
             */
            const cacheFolder = join(
              checkoutFolder,
              '.cache',
            );
            expect(fileSystem.foldersAUnitCannotWrite,).toEqual([checkoutFolder,],);
            expect(fileSystem.foldersAUnitMustWrite,).toEqual([
              worktree,
              `-${marker}`,
              `-${cacheFolder}`,
              runFolder,
            ],);
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
              foldersAUnitMustWrite: ['/repo/.git', '-/repo/.cache',],
            },),).toEqual([
              '--property=ReadOnlyPaths=/repo',
              '--property=ReadWritePaths=/repo/.git',
              '--property=ReadWritePaths=-/repo/.cache',
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
  ],
},);
