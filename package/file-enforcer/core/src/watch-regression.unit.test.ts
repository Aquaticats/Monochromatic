import {
  mkdir,
  mkdtemp,
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
  classifyEvent,
  l,
  reset,
  resetWriteTimestamps,
  watchDirectory,
  watchDirs,
} from '../dist/final/node/index.mjs';

/**
 Creates an isolated temp directory for watch regression tests.
 
 @returns Temp directory path.
 
 @example
 ```ts
 const tempDir = await setup();
 ```
 */
async function setup(): Promise<string> {
  return await mkdtemp(join(
    scratchRoot(),
    'file-enforcer-watch-regression-',
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

await describe({
  name: 'file-enforcer watch regressions',
  concurrency: 1,
  children: [
    it({
      name: 'empty glob dependencies watch future matching files',
      fn: async function emptyGlobWatchesFutureMatches(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        reset();
        resetWriteTimestamps();
        const sourceDirectory = join(
          tempDir,
          'src',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          configPath,
          '',
        );
        await mkdir(
          sourceDirectory,
          { recursive: true, },
        );
        await cat(join(
          sourceDirectory,
          '*.txt',
        ),);

        /**
         Directories watched after recording an empty glob expansion.
         */
        const watchedDirectories = await watchDirs(configPath,);
        /**
         Absolute source directory expected to be monitored for future files.
         */
        const resolvedSourceDirectory = resolve(sourceDirectory,);
        expect(watchedDirectories.has(resolvedSourceDirectory,),).toBe(true,);
        await writeFile(
          join(
            sourceDirectory,
            'new.txt',
          ),
          'new content',
        );
        expect(await classifyEvent({
          filename: 'new.txt',
          watchedDir: sourceDirectory,
          configPath,
        },),).toBe('source',);
      },
    },),

    it({
      name: 'missing glob dependency roots watch nearest existing ancestor',
      fn: async function missingGlobRootWatchesAncestor(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        reset();
        resetWriteTimestamps();
        const sourceDirectory = join(
          tempDir,
          'future',
          'src',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          configPath,
          '',
        );
        await cat(join(
          sourceDirectory,
          '*.txt',
        ),);

        /**
         Directories watched after recording an empty glob with missing static root.
         */
        const watchedDirectories = await watchDirs(configPath,);
        expect(watchedDirectories.has(tempDir,),).toBe(true,);
        /**
         Missing glob root that should not be watched directly until it exists.
         */
        const resolvedSourceDirectory = resolve(sourceDirectory,);
        expect(watchedDirectories.has(resolvedSourceDirectory,),).toBe(false,);
        await mkdir(
          sourceDirectory,
          { recursive: true, },
        );
        expect(await classifyEvent({
          filename: 'future',
          watchedDir: tempDir,
          configPath,
        },),).toBe('source',);
      },
    },),

    it({
      name: 'missing glob dependency roots advance watched ancestor after each rerun',
      fn: async function missingGlobRootAdvancesWatchedAncestor(): Promise<void> {
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        reset();
        resetWriteTimestamps();
        const futureDirectory = join(
          tempDir,
          'future',
        );
        const sourceDirectory = join(
          futureDirectory,
          'src',
        );
        const sourceGlob = join(
          sourceDirectory,
          '*.txt',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          configPath,
          '',
        );
        await cat(sourceGlob,);
        /**
         Directories watched before any missing glob-root ancestors exist.
         */
        const initiallyWatchedDirectories = await watchDirs(configPath,);
        expect(initiallyWatchedDirectories.has(tempDir,),).toBe(true,);

        await mkdir(futureDirectory,);
        expect(await classifyEvent({
          filename: 'future',
          watchedDir: tempDir,
          configPath,
        },),).toBe('source',);

        reset();
        await cat(sourceGlob,);
        /**
         Directories watched after first missing ancestor exists.
         */
        const futureWatchedDirectories = await watchDirs(configPath,);
        expect(futureWatchedDirectories.has(futureDirectory,),).toBe(true,);
        await mkdir(sourceDirectory,);
        expect(await classifyEvent({
          filename: 'src',
          watchedDir: futureDirectory,
          configPath,
        },),).toBe('source',);

        reset();
        await cat(sourceGlob,);
        /**
         Directories watched after static glob root exists.
         */
        const sourceWatchedDirectories = await watchDirs(configPath,);
        expect(sourceWatchedDirectories.has(sourceDirectory,),).toBe(true,);
        await writeFile(
          join(
            sourceDirectory,
            'new.txt',
          ),
          'new content',
        );
        expect(await classifyEvent({
          filename: 'new.txt',
          watchedDir: sourceDirectory,
          configPath,
        },),).toBe('source',);
      },
    },),

    it({
      name: 'watch directory setup failures reject instead of resolving',
      fn: async function watchDirectorySetupFailureRejects({ sinon, },): Promise<void> {
        const errorStub = sinon.stub(l, 'error',);
        const tempDir = await setup();
        await using _cleanup = {
          [Symbol.asyncDispose](): Promise<void> {
            return teardown(tempDir,);
          },
        };
        const missingDirectory = join(
          tempDir,
          'missing-watch-root',
        );
        const configPath = join(
          tempDir,
          'file-enforcer.config.ts',
        );
        await writeFile(
          configPath,
          '',
        );

        await expect(watchDirectory({
          dir: missingDirectory,
          signal: new AbortController().signal,
          configPath,
          onEvent: function unexpectedWatchEvent(): void {
            throw new Error('Missing-directory watcher unexpectedly emitted an event',);
          },
        },),)
          .rejects
          .toMatchObject({ code: 'ENOENT', },);
        expect(errorStub,).toHaveBeenCalledWith(
          expect.stringContaining('watcher error in',),
        );
      },
    },),

  ],
},);
