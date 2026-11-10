/**
 Tests that the ensure and empty families reach one filesystem target for one
 path spelling.

 A bundler-style `?query` suffix names no filesystem entry, so a spelling that
 carries one has to mean the same file or directory to both families: a caller
 that ensures `dist/x.js?raw` and then empties `dist/x.js?raw` must not clear a
 different file.

 Every case runs inside a disposable temporary directory; nothing touches
 the checkout.

 @module
 */

import {
  mkdtemp,
  readdir,
  rm,
  stat,
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
  emptyDir,
  emptyFile,
  emptyPath,
  ensureDir,
  ensureFile,
  ensurePath,
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
    'fs-path-query-',
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
 Path name a query suffix is appended to, below the fixture.
 */
const PLAIN_NAME = 'x.js';

/**
 Query suffix a bundler adds to the same path.
 */
const QUERY_SUFFIX = '?raw';

//endregion Fixture helpers

await describe({
  name: '',
  children: [
    describe({
      name: 'query suffix',
      children: [
        it({
          name: 'ensurePath and emptyPath reach the same file',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Parent directory both families resolve below.
             */
            const parent = join(
              temp.path,
              'dist',
            );
            /**
             File the spelling names once the suffix is stripped.
             */
            const target = join(
              parent,
              PLAIN_NAME,
            );
            /**
             Spelling a caller passes, suffix included.
             */
            const spelling = `${target}${QUERY_SUFFIX}`;
            expect(await ensurePath(spelling,),).toBe(spelling,);
            expect((await readdir(parent,)).toSorted(),).toEqual([ PLAIN_NAME, ],);
            await writeFile(
              target,
              'content a bundler asked for',
            );
            expect(await emptyPath(spelling,),).toBe(spelling,);
            expect((await readdir(parent,)).toSorted(),).toEqual([ PLAIN_NAME, ],);
            expect((await stat(target,)).size,).toBe(0,);
          },
        },),
        it({
          name: 'ensurePath and emptyPath reach the same directory',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Parent directory both families resolve below.
             */
            const parent = join(
              temp.path,
              'dist',
            );
            /**
             Directory the spelling names once the suffix is stripped.
             */
            const target = join(
              parent,
              'logs',
            );
            /**
             Spelling a caller passes, suffix included.
             */
            const spelling = `${target}${QUERY_SUFFIX}`;
            expect(await ensurePath(spelling,),).toBe(spelling,);
            expect((await stat(target,)).isDirectory(),).toBe(true,);
            await writeFile(
              join(
                target,
                'entry.txt',
              ),
              'content a bundler asked for',
            );
            expect(await emptyPath(spelling,),).toBe(spelling,);
            expect(await readdir(target,),).toEqual([],);
          },
        },),
        it({
          name: 'ensureFile and emptyFile reach the same file',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             File the spelling names once the suffix is stripped.
             */
            const target = join(
              temp.path,
              PLAIN_NAME,
            );
            /**
             Spelling a caller passes, suffix included.
             */
            const spelling = `${target}${QUERY_SUFFIX}`;
            expect(await ensureFile(spelling,),).toBe(spelling,);
            await writeFile(
              target,
              'content a bundler asked for',
            );
            expect(await emptyFile(spelling,),).toBe(spelling,);
            expect((await readdir(temp.path,)).toSorted(),).toEqual([ PLAIN_NAME, ],);
            expect((await stat(target,)).size,).toBe(0,);
          },
        },),
        it({
          name: 'ensureDir and emptyDir reach the same directory',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             Directory the spelling names once the suffix is stripped.
             */
            const target = join(
              temp.path,
              'cache',
            );
            /**
             Spelling a caller passes, suffix included.
             */
            const spelling = `${target}${QUERY_SUFFIX}`;
            expect(await ensureDir(spelling,),).toBe(spelling,);
            expect((await stat(target,)).isDirectory(),).toBe(true,);
            await writeFile(
              join(
                target,
                'entry.txt',
              ),
              'content a bundler asked for',
            );
            expect(await emptyDir(spelling,),).toBe(spelling,);
            expect(await readdir(target,),).toEqual([],);
          },
        },),
        it({
          name: 'leaves a spelling without a suffix naming the file it always named',
          fn: async () => {
            await using temp = await createTempDir();
            /**
             File a spelling without a suffix names.
             */
            const target = join(
              temp.path,
              PLAIN_NAME,
            );
            expect(await ensurePath(target,),).toBe(target,);
            await writeFile(
              target,
              'content a bundler asked for',
            );
            expect(await emptyPath(target,),).toBe(target,);
            expect((await stat(target,)).size,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
