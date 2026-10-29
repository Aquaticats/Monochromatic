import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import {
  classifyEvent,
  reset,
  resetWriteTimestamps,
  setWriteTimestamp,
  shouldTrigger,
  trackDest,
  trackRead,
  trackWriteTime,
  watchDirs,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region watchDirs

    describe({
      name: watchDirs.name,
      children: [
        it({
          name: 'includes config file directory',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const dirs = await watchDirs('/project/file-enforcer.config.ts',);
            expect(dirs.has('/project',),).toBe(true,);
          },
        },),
        it({
          name: 'includes parent directories of all tracked reads',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('/repo/AGENTS.md',);
            trackRead('/repo/packages/config/oxlint.json',);

            const dirs = await watchDirs('/repo/config.ts',);
            expect(dirs.has('/repo',),).toBe(true,);
            expect(dirs.has('/repo/packages/config',),).toBe(true,);
          },
        },),
        it({
          name: 'includes parent directories of tracked writes for protection',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackDest('/repo/CLAUDE.md',);

            const dirs = await watchDirs('/repo/config.ts',);
            expect(dirs.has('/repo',),).toBe(true,);
          },
        },),
        it({
          name: 'deduplicates directories when multiple paths share a parent',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('/repo/a.md',);
            trackDest('/repo/b.md',);

            const dirs = await watchDirs('/repo/config.ts',);
            expect(dirs.size,).toBe(1,);
          },
        },),
        it({
          name: 'returns only config dir when no reads or writes are tracked',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const dirs = await watchDirs('/project/config.ts',);
            expect(dirs.size,).toBe(1,);
          },
        },),
      ],
    },),

    //endregion watchDirs

    //region classifyEvent

    describe({
      name: classifyEvent.name,
      children: [
        it({
          name: 'classifies tracked read file as source',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('/repo/AGENTS.md',);
            expect(await classifyEvent({
              filename: 'AGENTS.md',
              watchedDir: '/repo',
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'source',
              );
          },
        },),
        it({
          name: 'classifies config file itself as source',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            expect(await classifyEvent({
              filename: 'config.ts',
              watchedDir: '/repo',
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'source',
              );
          },
        },),
        it({
          name: 'classifies our own write echo as ignore (mtime <= writeTimestamp)',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-classify-',),);
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return rm(tempDir, { recursive: true, force: true, },);
              },
            };
            /** Write a real file so stat() works */
            const filePath = join(tempDir, 'managed.md',);
            await writeFile(filePath, 'enforced content',);
            trackDest(filePath,);
            trackWriteTime(filePath,);

            /** Immediately after our write, mtime should be <= our timestamp */
            expect(await classifyEvent({
              filename: 'managed.md',
              watchedDir: tempDir,
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'ignore',
              );
          },
        },),
        it({
          name: 'classifies external edit as protected (mtime > writeTimestamp)',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-classify-',),);
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return rm(tempDir, { recursive: true, force: true, },);
              },
            };
            /** Write the file and record a timestamp in the past */
            const filePath = join(tempDir, 'protected.md',);
            await writeFile(filePath, 'original',);
            trackDest(filePath,);
            // Record a timestamp well in the past to simulate stale write
            /** Offset to push our recorded time into the past */
            const pastOffset = 2_000;
            setWriteTimestamp({ filePath, timestamp: Date.now() - pastOffset, },);

            /** Now modify the file: its mtime will be "now", after our recorded timestamp */
            await writeFile(filePath, 'externally modified',);

            expect(await classifyEvent({
              filename: 'protected.md',
              watchedDir: tempDir,
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'protected',
              );
          },
        },),
        it({
          name:
            'classifies as protected when dest has no write timestamp (content-skip case)',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-classify-',),);
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return rm(tempDir, { recursive: true, force: true, },);
              },
            };
            /** File registered as managed but never actually written (content was unchanged) */
            const filePath = join(tempDir, 'skipcase.md',);
            await writeFile(filePath, 'same',);
            trackDest(filePath,);
            // No trackWriteTime: simulates content-based skip

            expect(await classifyEvent({
              filename: 'skipcase.md',
              watchedDir: tempDir,
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'protected',
              );
          },
        },),
        it({
          name: 'classifies as protected when file was deleted externally',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-classify-',),);
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return rm(tempDir, { recursive: true, force: true, },);
              },
            };
            /** Register a path that doesn't exist on disk */
            trackDest(join(tempDir, 'deleted.md',),);
            trackWriteTime(join(tempDir, 'deleted.md',),);

            /** stat() will fail since the file was never created */
            expect(await classifyEvent({
              filename: 'deleted.md',
              watchedDir: tempDir,
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'protected',
              );
          },
        },),
        it({
          name: 'classifies unrelated file as ignore',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('/repo/AGENTS.md',);
            expect(await classifyEvent({
              filename: 'README.md',
              watchedDir: '/repo',
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'ignore',
              );
          },
        },),
        it({
          name: 'write classification takes precedence over read for dual-tracked paths',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-classify-',),);
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return rm(tempDir, { recursive: true, force: true, },);
              },
            };
            /** A file that is both read and written */
            const filePath = join(tempDir, 'dual.md',);
            await writeFile(filePath, 'content',);
            trackRead(filePath,);
            trackDest(filePath,);
            trackWriteTime(filePath,);

            // Immediately after write, echo detection should classify as ignore
            expect(await classifyEvent({
              filename: 'dual.md',
              watchedDir: tempDir,
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              'ignore',
              );
          },
        },),
      ],
    },),

    //endregion classifyEvent

    //region shouldTrigger

    describe({
      name: shouldTrigger.name,
      children: [
        it({
          name: 'returns true for source events',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            trackRead('/repo/AGENTS.md',);
            expect(await shouldTrigger({
              filename: 'AGENTS.md',
              watchedDir: '/repo',
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              true,
              );
          },
        },),
        it({
          name: 'returns true for protected events',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-trigger-',),);
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return rm(tempDir, { recursive: true, force: true, },);
              },
            };
            /** File with a stale write timestamp */
            const filePath = join(tempDir, 'stale.md',);
            await writeFile(filePath, 'old',);
            trackDest(filePath,);
            /** Push timestamp into the past */
            const pastOffset = 2_000;
            setWriteTimestamp({ filePath, timestamp: Date.now() - pastOffset, },);
            await writeFile(filePath, 'modified externally',);

            expect(await shouldTrigger({
              filename: 'stale.md',
              watchedDir: tempDir,
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              true,
              );
          },
        },),
        it({
          name: 'returns false for ignore events',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            expect(await shouldTrigger({
              filename: 'random.txt',
              watchedDir: '/repo',
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              false,
              );
          },
        },),
        it({
          name: 'returns false for our own write echoes',
          fn: async () => {
            reset();
            resetWriteTimestamps();
            const tempDir = await mkdtemp(join(scratchRoot(), 'file-enforcer-trigger-',),);
            await using _cleanup = {
              [Symbol.asyncDispose](): Promise<void> {
                return rm(tempDir, { recursive: true, force: true, },);
              },
            };
            /** File just written by us */
            const filePath = join(tempDir, 'echo.md',);
            await writeFile(filePath, 'ours',);
            trackDest(filePath,);
            trackWriteTime(filePath,);

            expect(await shouldTrigger({
              filename: 'echo.md',
              watchedDir: tempDir,
              configPath: '/repo/config.ts',
            },),)
              .toBe(
              false,
              );
          },
        },),
      ],
    },),
    //endregion shouldTrigger
  ],
},);
