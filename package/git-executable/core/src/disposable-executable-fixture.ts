/**
 Disposable executable candidates shared by this package's unit tests.

 Both unit test files build the same fixtures, so the file mode a candidate
 carries, the disposable directory a fixture lives in, and the rejection
 capture used to assert failures have one definition here.

 @module
 */

import {
  chmod,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  resolveRealGit,
  type ResolveRealGitOptions,
} from '../dist/final/node/index.mjs';

//region Constants

/**
 File mode making fixture scripts executable by owner.
 */
export const EXECUTABLE_MODE = 0o755;

//endregion Constants

//region Directories

/**
 Disposable temporary directory used by resolver tests.
 */
export type TempDirectory = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates disposable temporary directory for executable fixtures.
 
 @param prefix - Fixture directory prefix naming the test file that owns it.
 
 @returns Directory removed when async disposal completes.
 
 @example
 ```ts
 await using tempDirectory = await createTempDirectory({ prefix: 'git-executable-', });
 ```
 */
export async function createTempDirectory({
  prefix,
}: {
  readonly prefix: string;
},): Promise<TempDirectory> {
  /**
   Absolute fixture root unique to current test.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    prefix,
  ),);

  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Directories

//region Candidates

/**
 Writes executable fixture bytes.
 
 @param path - Absolute fixture path.
 
 @param content - Script or native-format bytes.
 
 @example
 ```ts
 await writeExecutable({ path: '/fixture/git', content: '#!/bin/sh\n' });
 ```
 */
export async function writeExecutable({
  path,
  content,
}: {
  readonly path: string;
  readonly content: string | Uint8Array;
},): Promise<void> {
  await writeFile(
    path,
    content,
    { mode: EXECUTABLE_MODE, },
  );
  await chmod(
    path,
    EXECUTABLE_MODE,
  );
}

/**
 Captures resolver rejection without promise matcher indirection.
 
 @param options - Disposable resolver inputs expected to reject.
 
 @returns Caught value or undefined when lookup unexpectedly resolves.
 
 @example
 ```ts
 const caught = await captureResolutionError({ pathEnv: '/missing' });
 ```
 */
export async function captureResolutionError(
  options: ResolveRealGitOptions,
): Promise<unknown> {
  try {
    await resolveRealGit(options,);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

//endregion Candidates
