import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  globResults,
  overwriteEach,
} from '../../dist/final/node/index.mjs';

//region Temporary fixture helpers

/**
 Creates isolated temporary directory for overwrite-each hardening tests.
 
 @returns Absolute temporary directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return mkdtemp(join(
    scratchRoot(),
    'file-enforcer-overwrite-each-',
  ),);
}

/**
 Removes isolated temporary directory after overwrite-each hardening tests.
 
 @param tempDir - Directory returned by {@link setup}.
 
 @example
 ```ts
 await teardown(tempDir);
 ```
 */
async function teardown(tempDir: string,): Promise<void> {
  await rm(
    tempDir,
    {
      recursive: true,
      force: true,
    },
  );
}

//endregion Temporary fixture helpers

await describe({
  name: overwriteEach.name,
  children: [
    it({
      name: 'rejects source patterns whose captures collapse to one destination',
      fn: async function rejectsCollapsedDestinationCaptures(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const sourceGlob = join(
          tempDir,
          'src',
          '*',
          '*.txt',
        );
        const destGlob = join(
          tempDir,
          'out',
          '**.txt',
        );
        const files = globResults({
          sourceGlob,
          results: [
            {
              path: join(
                tempDir,
                'src',
                'ab',
                'c.txt',
              ),
              content: 'first source',
            },
            {
              path: join(
                tempDir,
                'src',
                'a',
                'bc.txt',
              ),
              content: 'second source',
            },
          ],
        },);

        await expect(overwriteEach({
          destGlob,
          files,
        },),)
          .rejects
          .toThrow('Duplicate overwriteEach destination');
      },
    },),
  ],
},);
