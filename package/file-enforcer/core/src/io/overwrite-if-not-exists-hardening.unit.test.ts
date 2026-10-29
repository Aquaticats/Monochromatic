import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  lstat,
  mkdtemp,
  readlink,
  rm,
  symlink,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import { overwriteIfNotExists, } from '../../dist/final/node/index.mjs';

//region Temporary fixture helpers

/**
 Creates an isolated temporary directory for overwrite-if-absent hardening tests.
 
 @returns Absolute temporary directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return mkdtemp(join(
    scratchRoot(),
    'file-enforcer-overwrite-if-absent-',
  ),);
}

/**
 Removes an isolated temporary directory after a hardening test.
 
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
  name: overwriteIfNotExists.name,
  children: [
    it({
      name: 'preserves dangling symlink destinations as existing paths',
      fn: async function preservesDanglingSymlinkDestinations(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const dest = join(
          tempDir,
          'managed.txt',
        );
        const target = join(
          tempDir,
          'missing-target.txt',
        );
        await symlink(
          target,
          dest,
        );

        await overwriteIfNotExists({
          dest,
          content: 'generated content',
        },);

        expect((await lstat(dest,)).isSymbolicLink(),).toBe(true,);
        expect(await readlink(dest,),).toBe(target,);
      },
    },),
  ],
},);
