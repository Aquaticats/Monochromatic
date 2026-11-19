/**
 The temporary folder a run's Pi writes its files in, and the variables that name it. Every folder this file makes is disposable, so
 no test reads or changes the one a real run uses.

 @module
 */

import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  makeTemporaryFolderOfARun,
  TEMPORARY_FOLDER_MODE,
  TEMPORARY_FOLDER_NAME,
  TEMPORARY_FOLDER_VARIABLES,
  variablesNamingTemporaryFolder,
} from '../dist/final/node/index.mjs';

/**
 A folder this test may write in, removed however the test ends.
 */
type DisposableFolder = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a folder of the test's own.

 @returns the folder and its disposer

 @example
 ```ts
 await using folder = await makeFolder();
 ```
 */
async function makeFolder(): Promise<DisposableFolder> {
  /**
   The folder, named after what the test holds in it.
   */
  const path = await mkdtemp(join(scratchRoot(), 'pi-driver-temporary-folder-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(
        path,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 What the operating system says the permission bits of a path are.

 @param path - the path to read

 @returns its mode, with the file-kind bits left out

 @example
 ```ts
 const mode = permissionBitsOf('/runs/a-1/temporary-files-pi-wrote',);
 ```
 */
async function permissionBitsOf(path: string,): Promise<number> {
  return (await lstat(path,)).mode & 0o777;
}

await describe({
  name: '',
  children: [
    describe({
      name: makeTemporaryFolderOfARun.name,
      children: [
        it({
          name: 'makes one folder inside the run folder, named for what it holds, and gives it the mode the rule states',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run folder this test's run was given, which does not exist yet, as a launcher leaves it.
             */
            const runFolder = join(
              folder.path,
              'run',
            );
            /**
             What making the folder answered.
             */
            const made = await makeTemporaryFolderOfARun({ runFolder, },);
            expect(made.folder,).toBe(join(
              runFolder,
              TEMPORARY_FOLDER_NAME,
            ),);
            expect(await permissionBitsOf(made.folder,),).toBe(TEMPORARY_FOLDER_MODE,);
            expect((await lstat(made.folder,)).isDirectory(),).toBe(true,);
          },
        },),
        it({
          name: 'sets the mode the rule states on a folder the umask left more open, rather than trusting the mode mkdir was given',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The run folder this test's run was given.
             */
            const runFolder = join(
              folder.path,
              'run',
            );
            /**
             The folder already there, holding the wider mode a shell with a lax umask would leave.
             */
            const temporaryFolder = join(
              runFolder,
              TEMPORARY_FOLDER_NAME,
            );
            await mkdir(
              temporaryFolder,
              { recursive: true, },
            );
            await chmod(
              temporaryFolder,
              TEMPORARY_FOLDER_MODE + 0o077,
            );
            await makeTemporaryFolderOfARun({ runFolder, },);
            expect(await permissionBitsOf(temporaryFolder,),).toBe(TEMPORARY_FOLDER_MODE,);
          },
        },),
      ],
    },),
    describe({
      name: variablesNamingTemporaryFolder.name,
      children: [
        it({
          name: 'names the folder in every variable a program reads its temporary folder from',
          fn: async () => {
            /**
             The folder this test names in the variables.
             */
            const folder = join(
              '/runs',
              'copy-route-4',
              TEMPORARY_FOLDER_NAME,
            );
            /**
             The value each variable is given.
             */
            const variables = variablesNamingTemporaryFolder({ folder, },);
            expect(Object.keys(variables,)
              .toSorted(),).toEqual([...TEMPORARY_FOLDER_VARIABLES,].toSorted(),);
            for (const name of TEMPORARY_FOLDER_VARIABLES) {
              expect(variables[name],).toBe(folder,);
            }
          },
        },),
      ],
    },),
  ],
},);
