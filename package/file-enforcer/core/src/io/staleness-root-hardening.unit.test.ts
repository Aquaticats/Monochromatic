import {
  mkdir,
  mkdtemp,
  rm,
  symlink,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { findNodeModulesRoot, } from '../../dist/final/node/index.mjs';

//region Fixture helpers

/**
 Creates isolated temp directory for staleness root hardening tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'file-enforcer-staleness-root-',
  ),);
}

/**
 Removes isolated temp directory.
 
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

//endregion Fixture helpers

//region Error capture helpers

/**
 Captures node_modules root discovery failure for assertion.
 
 @param startDirectory - Start directory passed to findNodeModulesRoot.
 
 @returns Caught error, or undefined when discovery resolves.
 
 @example
 ```ts
 const error = await findNodeModulesRootError('/tmp/project/packages/app');
 ```
 */
async function findNodeModulesRootError(startDirectory: string,): Promise<unknown> {
  try {
    await findNodeModulesRoot({ startDirectory, },);
  }
  catch (error: unknown) {
    return error;
  }

  return undefined;
}

//endregion Error capture helpers

await describe({
  name: findNodeModulesRoot.name,
  children: [
    it({
      name: 'propagates corrupt node_modules symlink-loop stat failures',
      fn: async function propagatesCorruptNodeModulesStat(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const workspaceDirectory = join(
          tempDir,
          'workspace',
        );
        const packageDirectory = join(
          workspaceDirectory,
          'packages',
          'app',
        );
        await mkdir(
          packageDirectory,
          { recursive: true, },
        );
        await symlink(
          'node_modules',
          join(
            workspaceDirectory,
            'node_modules',
          ),
        );

        const caught = await findNodeModulesRootError(packageDirectory,);
        expect(caught,).toMatchObject({
          code: 'ELOOP',
        },);
      },
    },),
  ],
},);
