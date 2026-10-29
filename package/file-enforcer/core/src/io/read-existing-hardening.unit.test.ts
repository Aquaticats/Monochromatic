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
import { readExisting, } from '../../dist/final/node/index.mjs';

/**
 Creates an isolated temporary directory for read-existing hardening tests.
 
 @param prefix - Temp directory prefix identifying the test family.
 
 @returns Absolute temporary directory path.
 
 @example
 ```ts
 const tempDir = await setup('file-enforcer-read-existing-');
 ```
 */
async function setup(prefix: string,): Promise<string> {
  return mkdtemp(join(scratchRoot(), prefix,),);
}

/**
 Removes a temporary directory after a read-existing hardening test.
 
 @param tempDir - Absolute temporary directory path to remove.
 
 @example
 ```ts
 await teardown(tempDir);
 ```
 */
async function teardown(tempDir: string,): Promise<void> {
  await rm(tempDir, { recursive: true, force: true, },);
}

await describe({
  name: readExisting.name,
  children: [
    it({
      name: 'does not treat directory read failures as absent files',
      fn: async () => {
        const tempDir = await setup('file-enforcer-read-existing-',);
        await expect(readExisting(tempDir,),).rejects.toMatchObject({
          code: 'EISDIR',
        },);
        await teardown(tempDir,);
      },
    },),
  ],
},);
