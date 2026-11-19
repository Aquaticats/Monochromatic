/**
 The temporary folder a run's Pi writes its files in, the variables that name it, and what the maker answers when it cannot make it.
 Every folder this file makes is disposable, so no test reads or changes the one a real run uses.

 @module
 */

import {
  chmod,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
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
  makeTemporaryFolderOfARun,
  TEMPORARY_FOLDER_MODE,
  TEMPORARY_FOLDER_NAME,
  TEMPORARY_FOLDER_VARIABLES,
  TemporaryFolderNotMadeError,
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

/**
 The path the maker is asked about, inside a run folder the test names.

 @param runFolder - the run folder

 @returns the folder the maker makes inside it

 @example
 ```ts
 const temporaryFolder = temporaryFolderIn('/runs/a-1');
 ```
 */
function temporaryFolderIn(runFolder: string,): string {
  return join(
    runFolder,
    TEMPORARY_FOLDER_NAME,
  );
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
            expect(made.folder,).toBe(temporaryFolderIn(runFolder,),);
            expect(await permissionBitsOf(made.folder,),).toBe(TEMPORARY_FOLDER_MODE,);
            expect((await lstat(made.folder,)).isDirectory(),).toBe(true,);
          },
        },),
        it({
          name: 'sets the mode the rule states on a folder that was already there with wider bits, rather than trusting the mode mkdir was given',
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
             The folder already there, holding the wider mode somebody else's maker left.
             */
            const temporaryFolder = temporaryFolderIn(runFolder,);
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
        it({
          name: 'refuses a link planted under the folder name instead of following it, and leaves the folder it names at the mode it had',
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
             A folder outside the run folder that the planted link names, at a mode no run may change.
             */
            const outside = join(
              folder.path,
              'outside-the-run',
            );
            await mkdir(
              outside,
              { recursive: true, mode: 0o755, },
            );
            await chmod(outside, 0o755,);
            /**
             The mode the outside folder had before the maker was asked, which must be the one it keeps.
             */
            const outsideModeBefore = await permissionBitsOf(outside,);
            await mkdir(runFolder, { recursive: true, },);
            await symlink(
              outside,
              temporaryFolderIn(runFolder,),
            );
            /**
             What the maker threw.
             */
            let caught: unknown;
            try {
              await makeTemporaryFolderOfARun({ runFolder, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(TemporaryFolderNotMadeError,);
            expect((caught as TemporaryFolderNotMadeError).folder,).toBe(temporaryFolderIn(runFolder,),);
            expect((caught as Error).message,).toContain(temporaryFolderIn(runFolder,),);
            expect(await permissionBitsOf(outside,),).toBe(outsideModeBefore,);
            expect((await lstat(temporaryFolderIn(runFolder,))).isSymbolicLink(),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a plain file under the folder name, naming the folder and what the operating system answered',
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
             The file standing where the folder belongs, which holds what an earlier writer put there.
             */
            const fileInTheWay = temporaryFolderIn(runFolder,);
            await mkdir(runFolder, { recursive: true, },);
            await writeFile(fileInTheWay, 'not a folder\n',);
            /**
             What the maker threw.
             */
            let caught: unknown;
            try {
              await makeTemporaryFolderOfARun({ runFolder, },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(TemporaryFolderNotMadeError,);
            expect((caught as TemporaryFolderNotMadeError).folder,).toBe(fileInTheWay,);
            expect((caught as Error).message,).toContain(fileInTheWay,);
            expect((caught as Error).message,).toContain('EEXIST',);
            // The file the maker refused is left as its writer made it.
            expect(await readFile(fileInTheWay, 'utf8',),).toBe('not a folder\n',);
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
