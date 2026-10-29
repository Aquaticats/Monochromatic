import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdir,
  mkdtemp,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';

import { freshStalenessManifest, } from '../../dist/final/node/index.mjs';

//region Temporary fixture helpers

/**
 Creates isolated temporary directory for destination hardening tests.
 
 @returns Absolute temporary directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return mkdtemp(join(
    scratchRoot(),
    'file-enforcer-staleness-destination-',
  ),);
}

/**
 Removes isolated temporary directory after destination hardening tests.
 
 @param tempDir - Directory returned by {@link setup}.
 
 @example
 ```ts
 await teardown({ tempDir });
 ```
 */
async function teardown({ tempDir, }: { readonly tempDir: string; },): Promise<void> {
  await rm(
    tempDir,
    {
      recursive: true,
      force: true,
    },
  );
}

//endregion Temporary fixture helpers

//region Manifest fixture helpers

/**
 Writes manifest whose destination stamp points at directory path.
 
 @param manifestPath - Manifest path to create.
 
 @param destinationPath - Directory path recorded as destination stamp.
 
 @example
 ```ts
 await writeManifestWithDirectoryDestination({ manifestPath, destinationPath });
 ```
 */
async function writeManifestWithDirectoryDestination(
  {
    manifestPath,
    destinationPath,
  }: {
    readonly destinationPath: string;
    readonly manifestPath: string;
  },
): Promise<void> {
  const [, testEntryPoint,] = process.argv;
  if (testEntryPoint === undefined)
    throw new Error('Missing test entry point for destination hardening fixture',);

  /**
   Test file path used as active config dependency by direct unit execution.
   */
  const activeConfigPath = resolve(testEntryPoint,);
  /**
   Active config metadata needed for manifest freshness filtering.
   */
  const activeConfigStat = await stat(activeConfigPath,);
  /**
   Directory metadata crafted to match recorded destination stamps.
   */
  const destinationStat = await stat(destinationPath,);
  await writeFile(
    manifestPath,
    `${JSON.stringify({
      version: 1,
      entries: {
        'single:manual-destination-directory-fixture': {
          kind: 'single',
          sourceFiles: [{
            path: activeConfigPath,
            size: activeConfigStat.size,
            mtimeMs: activeConfigStat.mtimeMs,
          },],
          sourceGlobs: [],
          destinationFiles: [{
            path: resolve(destinationPath,),
            size: destinationStat.size,
            mtimeMs: destinationStat.mtimeMs,
            hash: 'not-a-real-directory-content-hash',
          },],
          sourceSetHash: 'manual-fixture',
          updatedAt: new Date().toISOString(),
        },
      },
    }, null, 2,)}\n`,
  );
}

//endregion Manifest fixture helpers

await describe({
  name: freshStalenessManifest.name,
  children: [
    it({
      name: 'does not treat destination directory reads as stale cache misses',
      fn: async function rejectsDirectoryDestinationHashRead(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown({ tempDir, },);
          },
        };
        const manifestPath = join(
          tempDir,
          'manifest.json',
        );
        const destinationPath = join(
          tempDir,
          'destination-directory',
        );
        await mkdir(destinationPath,);
        await writeManifestWithDirectoryDestination({
          manifestPath,
          destinationPath,
        },);

        await expect(freshStalenessManifest({ manifestPath, },),)
          .rejects
          .toMatchObject({
            code: 'EISDIR',
          },);
      },
    },),
  ],
},);
