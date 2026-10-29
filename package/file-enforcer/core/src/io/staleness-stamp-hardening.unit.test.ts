import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
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
 Creates isolated temporary directory for staleness stamp hardening tests.
 
 @returns Absolute temporary directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return mkdtemp(join(
    scratchRoot(),
    'file-enforcer-staleness-stamp-',
  ),);
}

/**
 Removes isolated temporary directory after staleness stamp hardening tests.
 
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

//region Manifest fixture helpers

/**
 Writes manifest whose active-config entry records an invalid nested source path.
 
 @param manifestPath - Manifest path to create.
 
 @param invalidSourcePath - Source path that should fail with `ENOTDIR` when statted.
 
 @example
 ```ts
 await writeManifestWithInvalidSource({ manifestPath, invalidSourcePath });
 ```
 */
async function writeManifestWithInvalidSource(
  {
    manifestPath,
    invalidSourcePath,
  }: {
    readonly invalidSourcePath: string;
    readonly manifestPath: string;
  },
): Promise<void> {
  const [, testEntryPoint,] = process.argv;
  if (testEntryPoint === undefined)
    throw new Error('Missing test entry point for staleness stamp fixture',);

  /**
   Test file path used as active config dependency by direct unit execution.
   */
  const activeConfigPath = resolve(testEntryPoint,);
  /**
   Active config metadata needed for manifest freshness filtering.
   */
  const activeConfigStat = await stat(activeConfigPath,);
  await writeFile(
    manifestPath,
    `${JSON.stringify({
      version: 1,
      entries: {
        'single:manual-staleness-stamp-fixture': {
          kind: 'single',
          sourceFiles: [
            {
              path: activeConfigPath,
              size: activeConfigStat.size,
              mtimeMs: activeConfigStat.mtimeMs,
            },
            {
              path: resolve(invalidSourcePath,),
              size: 0,
              mtimeMs: 0,
            },
          ],
          sourceGlobs: [],
          destinationFiles: [],
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
      name: 'does not treat non-directory source stamp failures as missing files',
      fn: async function rejectsNonDirectorySourceStampFailure(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const manifestPath = join(
          tempDir,
          'manifest.json',
        );
        const regularFilePath = join(
          tempDir,
          'not-a-directory',
        );
        const invalidSourcePath = join(
          regularFilePath,
          'source.txt',
        );
        await writeFile(
          regularFilePath,
          'regular file',
        );
        await writeManifestWithInvalidSource({
          manifestPath,
          invalidSourcePath,
        },);

        await expect(freshStalenessManifest({ manifestPath, },),)
          .rejects
          .toMatchObject({
            code: 'ENOTDIR',
          },);
      },
    },),
  ],
},);
