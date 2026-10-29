/**
 Tests for the empty family through the built `./node` entry.

 Every case runs inside a disposable temporary directory; nothing touches
 the checkout.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  emptyDir,
  emptyFile,
  emptyPath,
  removeEmptyFilesInDir,
} from '@monochromatic-dev/fs-path-core/node';

//region Fixture helpers

/**
 Disposable temporary directory.
 */
type TempDir = {
  /**
   Absolute path of the directory.
   */
  readonly path: string;

  /**
   Removes the directory tree after the test.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable temporary directory.

 @returns directory that removes itself on dispose

 @example
 ```ts
 await using temp = await createTempDir();
 ```
 */
async function createTempDir(): Promise<TempDir> {
  /**
   Fresh directory under the system temporary root.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'fs-path-empty-',
  ),);
  return {
    path,
    [Symbol.asyncDispose](): Promise<void> {
      return rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Populates a directory with a file, a nested directory holding a file, and
 an empty nested directory.

 @param dir - directory to populate

 @example
 ```ts
 await populate(temp.path);
 ```
 */
async function populate(dir: string,): Promise<void> {
  await mkdir(
    join(
      dir,
      'nested',
    ),
    { recursive: true, },
  );
  await mkdir(
    join(
      dir,
      'hollow',
    ),
  );
  await writeFile(
    join(
      dir,
      'top.txt',
    ),
    'top',
  );
  await writeFile(
    join(
      dir,
      'nested',
      'inner.txt',
    ),
    'inner',
  );
}

//endregion Fixture helpers

await describe({
  name: '',
  children: [
    describe({
      name: emptyDir.name,
      children: [
        it({
          name: 'removes every entry, nested directories included, and keeps the directory',
          fn: async () => {
            await using temp = await createTempDir();
            await populate(temp.path,);
            expect(await emptyDir(temp.path,),).toBe(temp.path,);
            expect(await readdir(temp.path,),).toEqual([],);
            expect((await stat(temp.path,)).isDirectory(),).toBe(true,);
          },
        },),
        it({
          name: 'leaves an already empty directory in place',
          fn: async () => {
            await using temp = await createTempDir();
            expect(await emptyDir(temp.path,),).toBe(temp.path,);
            expect(await readdir(temp.path,),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: emptyFile.name,
      children: [
        it({
          name: 'truncates the file to zero bytes',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             File holding content before the call.
             */
            const target = join(
              temp.path,
              'bundle.js',
            );
            await writeFile(
              target,
              'console.log(1)',
            );
            expect(await emptyFile(target,),).toBe(target,);
            expect(await readFile(
              target,
              'utf8',
            ),).toBe('',);
          },
        },),
        it({
          name: 'strips a query suffix before writing and returns the original spelling',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             On-disk file the query-bearing spelling refers to.
             */
            const target = join(
              temp.path,
              'asset.css',
            );
            await writeFile(
              target,
              'body{}',
            );
            expect(await emptyFile(`${target}?raw`,),).toBe(`${target}?raw`,);
            expect(await readFile(
              target,
              'utf8',
            ),).toBe('',);
            expect(await readdir(temp.path,),).toEqual(['asset.css',],);
          },
        },),
      ],
    },),

    describe({
      name: emptyPath.name,
      children: [
        it({
          name: 'empties a path with an extension as a file',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Extension-bearing path treated as a file.
             */
            const target = join(
              temp.path,
              'out.txt',
            );
            await writeFile(
              target,
              'text',
            );
            expect(await emptyPath(target,),).toBe(target,);
            expect(await readFile(
              target,
              'utf8',
            ),).toBe('',);
          },
        },),
        it({
          name: 'empties a path without an extension as a directory',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Extension-less path treated as a directory.
             */
            const target = join(
              temp.path,
              'dist',
            );
            await mkdir(target,);
            await populate(target,);
            expect(await emptyPath(target,),).toBe(target,);
            expect(await readdir(target,),).toEqual([],);
          },
        },),
        it({
          name: 'ignores a query suffix when deciding between file and directory',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory whose query-bearing spelling must not read as a file.
             */
            const target = join(
              temp.path,
              'assets',
            );
            await mkdir(target,);
            await populate(target,);
            expect(await emptyPath(`${target}?raw`,),).toBe(target,);
            expect(await readdir(target,),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: removeEmptyFilesInDir.name,
      children: [
        it({
          name: 'removes zero-byte and whitespace-only files, keeping content and directories',
          fn: async () => {
            await using temp = await createTempDir();
            await mkdir(join(
              temp.path,
              'sub',
            ),);
            await writeFile(
              join(
                temp.path,
                'blank.txt',
              ),
              '',
            );
            await writeFile(
              join(
                temp.path,
                'spaces.txt',
              ),
              ' \n\t\n',
            );
            await writeFile(
              join(
                temp.path,
                'full.txt',
              ),
              'content',
            );
            expect(await removeEmptyFilesInDir(temp.path,),).toBe(temp.path,);
            expect((await readdir(temp.path,)).toSorted(),).toEqual([
              'full.txt',
              'sub',
            ],);
          },
        },),
      ],
    },),
  ],
},);
