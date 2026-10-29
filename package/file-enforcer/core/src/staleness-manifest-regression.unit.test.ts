import { createHash, } from 'node:crypto';
import {
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  freshStalenessManifest,
  overwrite,
} from '../dist/final/node/index.mjs';

//region Manifest fixture helpers

/**
 Creates an isolated temp directory for manifest regression tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'file-enforcer-manifest-regression-',
  ),);
}

/**
 Removes an isolated temp directory.
 
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

/**
 Hashes content with the same algorithm the staleness manifest records.
 
 @param content - Content to hash.
 
 @returns SHA-256 hex digest.
 
 @example
 ```ts
 const hash = sha256('content');
 ```
 */
function sha256(content: string,): string {
  return createHash('sha256',)
    .update(content,)
    .digest('hex',);
}

/**
 Captures expected stale-manifest validation error.
 
 @param manifestPath - Manifest path that should fail validation.
 
 @returns Error thrown by freshness check.
 
 @throws When freshness check unexpectedly succeeds.
 
 @example
 ```ts
 const error = await captureFreshStalenessManifestError({ manifestPath });
 ```
 */
async function captureFreshStalenessManifestError(
  { manifestPath, }: { readonly manifestPath: string; },
): Promise<unknown> {
  try {
    await freshStalenessManifest({ manifestPath, },);
  }
  catch (error) {
    return error;
  }

  throw new Error('freshStalenessManifest unexpectedly accepted corrupt manifest',);
}

//endregion Manifest fixture helpers

await describe({
  name: 'file-enforcer manifest regressions',
  concurrency: 1,
  children: [
    it({
      name: 'corrupt staleness manifest fails closed instead of silently resetting',
      fn: async function corruptManifestFailsClosed(): Promise<void> {
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
        await writeFile(
          manifestPath,
          '{not-json',
        );

        /**
         Error produced by corrupt manifest validation.
         */
        const caught = await captureFreshStalenessManifestError({ manifestPath, },);

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('staleness manifest',);
      },
    },),

    it({
      name: 'destination hash mismatch makes same-metadata manifest stale',
      fn: async function destinationHashMismatchMakesManifestStale(): Promise<void> {
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
        const destPath = join(
          tempDir,
          'output.txt',
        );
        await writeFile(
          destPath,
          'bravo',
        );
        const destStat = await stat(destPath,);
        const [, testEntryPoint,] = process.argv;
        if (testEntryPoint === undefined)
          throw new Error('Missing test entry point for staleness fixture',);
        const activeConfigPath = resolve(testEntryPoint,);
        const configStat = await stat(activeConfigPath,);
        await writeFile(
          manifestPath,
          `${JSON.stringify({
            version: 1,
            entries: {
              [`single:${resolve(destPath,)}`]: {
                kind: 'single',
                sourceFiles: [{
                  path: activeConfigPath,
                  size: configStat.size,
                  mtimeMs: configStat.mtimeMs,
                },],
                sourceGlobs: [],
                destinationFiles: [{
                  path: resolve(destPath,),
                  size: destStat.size,
                  mtimeMs: destStat.mtimeMs,
                  hash: sha256('alpha',),
                },],
                sourceSetHash: 'manual-fixture',
                updatedAt: new Date().toISOString(),
              },
            },
          }, null, 2,)}\n`,
        );

        expect(await freshStalenessManifest({ manifestPath, },),).toBe(false,);
      },
    },),

    it({
      name: 'overwrite flushes manifest entry before returning',
      fn: async function overwriteFlushesManifestEntryBeforeReturning(): Promise<void> {
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
        const destPath = join(
          tempDir,
          'output.txt',
        );

        await overwrite({
          dest: destPath,
          content: 'immediate manifest content',
          manifestPath,
        },);

        const manifest = JSON.parse(await readFile(manifestPath, 'utf8',),) as {
          readonly entries?: Record<string, unknown>;
        };

        expect(Object.keys(manifest.entries ?? {},),).toContain(
          `single:${resolve(destPath,)}`,
        );
      },
    },),
  ],
},);
