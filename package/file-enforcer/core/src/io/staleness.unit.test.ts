import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
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
  cat,
  freshStalenessManifest,
  invalidatePaths,
  overwrite,
  overwriteEach,
  reads,
  reset,
  resetWriteTimestamps,
  writes,
  type GlobResults,
} from '../../dist/final/node/index.mjs';

/**
 Creates an isolated temp directory for staleness tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(scratchRoot(), 'file-enforcer-staleness-',),);
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
  await rm(tempDir, { recursive: true, force: true, },);
}

/**
 Increments a named counter stored in a map.
 
 @param calls - Counter map.
 
 @example
 ```ts
 incrementCalls(calls);
 ```
 */
function incrementCalls(calls: Map<'count', number>,): void {
  calls.set(
    'count',
    (calls.get('count',) ?? 0) + 1,
  );
}

/**
 Reads a named counter stored in a map.
 
 @param calls - Counter map.
 
 @returns Current counter value.
 
 @example
 ```ts
 const count = callCount(calls);
 ```
 */
function callCount(calls: ReadonlyMap<'count', number>,): number {
  return calls.get('count',) ?? 0;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'eager staleness manifest',
      concurrency: 1,
      children: [
        it({
          name: 'reports fresh after eager overwrite sources and destinations are unchanged',
          fn: async function reportsFreshEagerOverwrite(): Promise<void> {
            const tempDir = await setup();
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return teardown(tempDir,);
              },
            };
            reset();
            resetWriteTimestamps();
            const source = join(tempDir, 'source.txt',);
            const dest = join(tempDir, 'dest.txt',);
            const manifestPath = join(tempDir, 'manifest.json',);
            await writeFile(source, 'alpha',);

            await overwrite({
              dest,
              content: await cat([source,],),
              manifestPath,
            },);
            reset();
            resetWriteTimestamps();

            const fresh = await freshStalenessManifest({ manifestPath, },);

            expect(fresh,).toBe(true,);
            const sourceTracked = reads.has(resolve(source,),);
            const destTracked = writes.has(resolve(dest,),);
            expect(sourceTracked,).toBe(true,);
            expect(destTracked,).toBe(true,);
          },
        },),
        it({
          name: 'reports stale after an eager overwrite source changes',
          fn: async function reportsStaleChangedEagerSource(): Promise<void> {
            const tempDir = await setup();
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return teardown(tempDir,);
              },
            };
            reset();
            resetWriteTimestamps();
            const source = join(tempDir, 'source.txt',);
            const dest = join(tempDir, 'dest.txt',);
            const manifestPath = join(tempDir, 'manifest.json',);
            await writeFile(source, 'alpha',);

            await overwrite({
              dest,
              content: await cat([source,],),
              manifestPath,
            },);
            await writeFile(source, 'beta',);
            reset();
            resetWriteTimestamps();

            expect(await freshStalenessManifest({ manifestPath, },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'lazy overwrite staleness cache',
      concurrency: 1,
      children: [
        it({
          name: 'skips content builder when sources and destination are unchanged',
          fn: async function skipsUnchangedLazyOverwrite(): Promise<void> {
            const tempDir = await setup();
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return teardown(tempDir,);
              },
            };
            reset();
            resetWriteTimestamps();
            const source = join(tempDir, 'source.txt',);
            const dest = join(tempDir, 'dest.txt',);
            const manifestPath = join(tempDir, 'manifest.json',);
            const calls = new Map<'count', number>();
            await writeFile(source, 'alpha',);

            async function buildContent(): Promise<string> {
              incrementCalls(calls,);
              return await cat([source,],);
            }

            await overwrite({ dest, content: buildContent, manifestPath, },);
            reset();
            resetWriteTimestamps();
            await overwrite({ dest, content: buildContent, manifestPath, },);

            expect(callCount(calls,),).toBe(1,);
            expect(await readFile(dest, 'utf8',),).toBe('alpha',);
            const sourceTracked = reads.has(resolve(source,),);
            const destTracked = writes.has(resolve(dest,),);
            expect(sourceTracked,).toBe(true,);
            expect(destTracked,).toBe(true,);
          },
        },),
        it({
          name: 'rebuilds when a source file changes',
          fn: async function rebuildsChangedSource(): Promise<void> {
            const tempDir = await setup();
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return teardown(tempDir,);
              },
            };
            reset();
            resetWriteTimestamps();
            const source = join(tempDir, 'source.txt',);
            const dest = join(tempDir, 'dest.txt',);
            const manifestPath = join(tempDir, 'manifest.json',);
            const calls = new Map<'count', number>();
            await writeFile(source, 'alpha',);

            async function buildContent(): Promise<string> {
              incrementCalls(calls,);
              return await cat([source,],);
            }

            await overwrite({ dest, content: buildContent, manifestPath, },);
            await writeFile(source, 'beta content',);
            invalidatePaths([source,],);
            reset();
            resetWriteTimestamps();
            await overwrite({ dest, content: buildContent, manifestPath, },);

            expect(callCount(calls,),).toBe(2,);
            expect(await readFile(dest, 'utf8',),).toBe('beta content',);
          },
        },),
        it({
          name: 'repairs an externally edited destination',
          fn: async function repairsChangedDestination(): Promise<void> {
            const tempDir = await setup();
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return teardown(tempDir,);
              },
            };
            reset();
            resetWriteTimestamps();
            const source = join(tempDir, 'source.txt',);
            const dest = join(tempDir, 'dest.txt',);
            const manifestPath = join(tempDir, 'manifest.json',);
            const calls = new Map<'count', number>();
            await writeFile(source, 'alpha',);

            async function buildContent(): Promise<string> {
              incrementCalls(calls,);
              return await cat([source,],);
            }

            await overwrite({ dest, content: buildContent, manifestPath, },);
            await writeFile(dest, 'external edit',);
            invalidatePaths([dest,],);
            reset();
            resetWriteTimestamps();
            await overwrite({ dest, content: buildContent, manifestPath, },);

            expect(callCount(calls,),).toBe(2,);
            expect(await readFile(dest, 'utf8',),).toBe('alpha',);
          },
        },),
      ],
    },),
    describe({
      name: 'lazy overwriteEach staleness cache',
      concurrency: 1,
      children: [
        it({
          name: 'skips glob builder until the glob path set changes',
          fn: async function skipsUntilGlobChanges(): Promise<void> {
            const tempDir = await setup();
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return teardown(tempDir,);
              },
            };
            reset();
            resetWriteTimestamps();
            const sourceDir = join(tempDir, 'src',);
            const destDir = join(tempDir, 'dest',);
            const manifestPath = join(tempDir, 'manifest.json',);
            const calls = new Map<'count', number>();
            await mkdir(sourceDir, { recursive: true, },);
            await writeFile(join(sourceDir, 'a.txt',), 'alpha',);
            await writeFile(join(sourceDir, 'b.txt',), 'beta',);

            async function readFiles(): Promise<GlobResults> {
              incrementCalls(calls,);
              return await cat(join(sourceDir, '*.txt',),);
            }

            await overwriteEach({
              destGlob: join(destDir, '*.txt',),
              files: readFiles,
              manifestPath,
            },);
            reset();
            resetWriteTimestamps();
            await overwriteEach({
              destGlob: join(destDir, '*.txt',),
              files: readFiles,
              manifestPath,
            },);
            await writeFile(join(sourceDir, 'c.txt',), 'gamma',);
            reset();
            resetWriteTimestamps();
            await overwriteEach({
              destGlob: join(destDir, '*.txt',),
              files: readFiles,
              manifestPath,
            },);

            expect(callCount(calls,),).toBe(2,);
            expect(await readFile(join(destDir, 'a.txt',), 'utf8',),).toBe('alpha',);
            expect(await readFile(join(destDir, 'c.txt',), 'utf8',),).toBe('gamma',);
          },
        },),
      ],
    },),
  ],
},);
