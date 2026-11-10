/**
 File access at the boundary a build depends on: the in-memory registry a
 browser populates, the disk fallback a server uses, and the write the build
 pipeline ends with. The registry is consulted before the disk, so a seeded
 path must win over a file of the same name on disk.
 */
import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  existsSync,
  fsRegistry,
  readTextFile,
  readTextFileSync,
  writeTextFileEnsuringParentDirectory,
} from '@monochromatic-dev/css-cli';

//region Test helpers

/**
 One disposable directory per suite run, holding the files these cases read
 and write, so nothing lands in the package and nothing outlives the run.
 */
type DisposableDirectory = {
  /**
   Absolute path of the directory.
   */
  readonly path: string;
  /**
   Removes the directory and everything written into it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates the disposable directory this suite's files live in.
 
 @returns Directory path plus the disposal hook that removes it.
 */
async function disposableDirectory(): Promise<DisposableDirectory> {
  /**
   Directory owning every file this suite writes.
   */
  const directoryPath = await mkdtemp(join(
    scratchRoot(),
    'css-cli-fs-',
  ),);
  return {
    path: directoryPath,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directoryPath,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Test helpers

await describe({
  name: 'file access',
  children: [
    describe({
      name: readTextFileSync.name,
      children: [
        it({
          name: 'reads a file from disk',
          fn: async () => {
            await using directory = await disposableDirectory();
            await writeFile(
              join(directory.path, 'disk.css',),
              '.disk { top: 0; }',
            );

            /**
             Path of the file the case reads.
             */
            const diskPath = join(directory.path, 'disk.css',);
            expect(readTextFileSync(diskPath,),).toBe('.disk { top: 0; }',);
          },
        },),
        it({
          name: 'prefers a registry entry over the file on disk',
          fn: async () => {
            await using directory = await disposableDirectory();
            /**
             Path registered in memory and written to disk with other text.
             */
            const registeredPath = join(directory.path, 'registered.css',);
            await writeFile(registeredPath, '.disk { top: 0; }',);
            fsRegistry.set(registeredPath, '.registry { top: 0; }',);

            expect(readTextFileSync(registeredPath,),)
              .toBe('.registry { top: 0; }',);
          },
        },),
        it({
          name: 'throws when neither the registry nor the disk holds the file',
          fn: async () => {
            await using directory = await disposableDirectory();
            let caught: unknown;
            try {
              readTextFileSync(join(directory.path, 'absent.css',),);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('absent.css',);
          },
        },),
      ],
    },),
    describe({
      name: readTextFile.name,
      children: [
        it({
          name: 'reads a file from disk',
          fn: async () => {
            await using directory = await disposableDirectory();
            await writeFile(
              join(directory.path, 'async-disk.css',),
              '.async-disk { top: 0; }',
            );

            /**
             Path of the file the case reads.
             */
            const diskPath = join(directory.path, 'async-disk.css',);
            expect(await readTextFile(diskPath,),).toBe('.async-disk { top: 0; }',);
          },
        },),
        it({
          name: 'prefers a registry entry over the file on disk',
          fn: async () => {
            await using directory = await disposableDirectory();
            /**
             Path registered in memory and written to disk with other text.
             */
            const registeredPath = join(directory.path, 'async-registered.css',);
            await writeFile(registeredPath, '.disk { top: 0; }',);
            fsRegistry.set(registeredPath, '.registry { top: 0; }',);

            expect(await readTextFile(registeredPath,),)
              .toBe('.registry { top: 0; }',);
          },
        },),
      ],
    },),
    describe({
      name: existsSync.name,
      children: [
        it({
          name: 'reports a registry entry that no disk holds',
          fn: async () => {
            /**
             Path only the registry holds.
             */
            const registeredPath = '/registered-only/entry.css';
            fsRegistry.set(registeredPath, '.entry { top: 0; }',);
            expect(existsSync(registeredPath,),).toBe(true,);
          },
        },),
        it({
          name: 'reports a file that exists on disk',
          fn: async () => {
            await using directory = await disposableDirectory();
            await writeFile(
              join(directory.path, 'exists.css',),
              '.exists { top: 0; }',
            );

            /**
             Path of the file the case probes.
             */
            const existingPath = join(directory.path, 'exists.css',);
            expect(existsSync(existingPath,),).toBe(true,);
          },
        },),
        it({
          name: 'reports a path that neither the registry nor the disk holds',
          fn: async () => {
            await using directory = await disposableDirectory();
            /**
             Path neither the registry nor the disk holds.
             */
            const absentPath = join(directory.path, 'absent.css',);
            expect(existsSync(absentPath,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: writeTextFileEnsuringParentDirectory.name,
      children: [
        it({
          name: 'writes a file inside directories it creates',
          fn: async () => {
            await using directory = await disposableDirectory();
            /**
             Path two missing directories below the disposable root.
             */
            const outputPath = join(
              directory.path,
              'nested',
              'deeper',
              'out.css',
            );
            await writeTextFileEnsuringParentDirectory({
              absolutePath: outputPath,
              content: '.written { top: 0; }',
            },);

            expect(await readFile(outputPath, 'utf8',),)
              .toBe('.written { top: 0; }',);
          },
        },),
      ],
    },),
  ],
},);
