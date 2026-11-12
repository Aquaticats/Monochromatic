/**
 Keeping the whole of what a failed start printed beside the runs, and saying so when a file cannot be written. The test writes in
 a throwaway folder, so no real runs folder is read or written.

 @module
 */

import {
  mkdtemp,
  readFile,
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
import { keepFailedOutput, } from '../dist/final/node/index.mjs';

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
  const path = await mkdtemp(join(scratchRoot(), 'pi-start-failure-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: keepFailedOutput.name,
      children: [
        it({
          name: 'keeps the whole of what a failed start printed beside the runs, and appends a second failure of the same name',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             The name the failed starts are of.
             */
            const name = 'copy-route';
            /**
             The file the first failure was kept in.
             */
            const first = await keepFailedOutput({
              runsFolder: folder.path,
              name,
              output: 'why it failed\n',
              report: function report() {
                throw new Error('a file that was written was reported as unwritable',);
              },
            },);
            expect(first.keptAt,).toBe(join(folder.path, `${name}-start-failure.log`,),);
            await keepFailedOutput({
              runsFolder: folder.path,
              name,
              output: 'why it failed again',
              report: function report() {
                throw new Error('a file that was written was reported as unwritable',);
              },
            },);
            /**
             What the file holds.
             */
            const kept = await readFile(first.keptAt ?? '', 'utf8',);
            expect(kept,).toContain('why it failed',);
            expect(kept,).toContain('why it failed again',);
          },
        },),
        it({
          name: 'reports a file it could not write, and answers no path, so the line still says why the start failed',
          fn: async () => {
            await using folder = await makeFolder();
            /**
             A file where the runs folder should be, so nothing can be kept under it.
             */
            const notAFolder = join(folder.path, 'a-file',);
            await writeFile(notAFolder, '',);
            /**
             What was reported.
             */
            const reported: string[] = [];
            /**
             Where the output would have been kept.
             */
            const kept = await keepFailedOutput({
              runsFolder: notAFolder,
              name: 'copy-route',
              output: 'why it failed',
              report: function report(line,) {
                reported.push(line,);
              },
            },);
            expect(kept.keptAt,).toBeUndefined();
            expect(reported,).toHaveLength(1,);
            expect(reported[0],).toContain('could not be kept',);
          },
        },),
      ],
    },),
  ],
},);
