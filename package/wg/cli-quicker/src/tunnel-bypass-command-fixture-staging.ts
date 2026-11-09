/**
 Staging of the built CLI copy that the bypass integration tests run.

 @module
 */

import {
  cp,
  mkdir,
  mkdtemp,
  rm,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import type { StagedCli, } from './tunnel-bypass-command-fixture-types.ts';

/**
 Package directory, whose `.cache` holds staged copies of the built CLI.
 Staging inside the package keeps the bundle's bare imports resolvable through the package's `node_modules`.
 */
const PACKAGE_DIRECTORY = fileURLToPath(new URL(
  '..',
  import.meta.url,
),);

/**
 Copies the built CLI into a disposable tree whose paired Rust watcher is `/usr/bin/true`.

 The CLI finds its watcher only next to its own package, so the integration test cannot name a watcher
 through the environment, which would be a switch for the process policy in production too.
 A staged tree puts the stand-in where the CLI looks, and leaves the real watcher untouched.

 @returns Entry point and a disposer that removes the tree.

 @example
 ```ts
 await using staged = await stageWgQuickerCli();
 ```
 */
export async function stageWgQuickerCli(): Promise<StagedCli> {
  /**
   Parent directory of the staged tree.
   */
  const cacheDirectory = join(
    PACKAGE_DIRECTORY,
    '.cache',
    'integration-stage',
  );
  await mkdir(
    cacheDirectory,
    { recursive: true, },
  );
  /**
   Root of this staged tree, laid out as `package/wg/<package>`.
   */
  const root = await mkdtemp(join(
    cacheDirectory,
    'cli-',
  ),);
  /**
   Staged copy of the TypeScript package's built output.
   */
  const bundleDirectory = join(
    root,
    'package',
    'wg',
    'cli-quicker',
    'dist',
    'final',
    'node',
  );
  await mkdir(
    bundleDirectory,
    { recursive: true, },
  );
  await cp(
    join(
      PACKAGE_DIRECTORY,
      'dist',
      'final',
      'node',
    ),
    bundleDirectory,
    { recursive: true, },
  );
  /**
   Where the CLI looks for the paired release watcher.
   */
  const watcherDirectory = join(
    root,
    'package',
    'wg',
    'cli-quicker-exempt',
    'target',
    'release',
  );
  await mkdir(
    watcherDirectory,
    { recursive: true, },
  );
  await symlink(
    '/usr/bin/true',
    join(
      watcherDirectory,
      'wg-quicker-exempt',
    ),
  );
  return {
    cliPath: join(
      bundleDirectory,
      'index.mjs',
    ),
    [Symbol.asyncDispose]: async function removeStage(): Promise<void> {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}
