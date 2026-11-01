/**
 Tests for complete-output spooling in the built bash-poke artifact.

 @module
 */

import { chmod, mkdir, mkdtemp, readFile, rm, stat, writeFile, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  constants,
  discardingSpoolWriter,
  openSpoolWriter,
  spoolDirectory,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Disposable temp root, removed when async disposal completes.
 */
type DisposableTempRoot = {
  /**
   Absolute directory spooling happens below.
   */
  readonly path: string;

  /**
   Removal hook.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable temp root for one case.
 
 @returns root directory removed on disposal
 
 @example
 ```ts
 await using root = await createDisposableTempRoot();
 ```
 */
async function createDisposableTempRoot(): Promise<DisposableTempRoot> {
  /**
   Absolute directory unique to the current case.
   */
  const path = await mkdtemp(join(scratchRoot(), 'bash-poke-spool-', ), );
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { recursive: true, force: true, }, );
    },
  };
}

/**
 Reads the permission bits of a path as an octal string.
 
 @param path - file or directory to inspect
 
 @returns three octal permission digits
 
 @example
 ```ts
 await permissionDigits('/tmp/x');
 ```
 */
async function permissionDigits(path: string, ): Promise<string> {
  /**
   Full mode word, whose last three octal digits are the permission bits.
   */
  const info = await stat(path, );
  return info.mode.toString(8, ).slice(-3, );
}

/**
 Requires a spool path, failing the case when spooling was unavailable.
 
 @param writer - opened spool writer
 
 @returns absolute spool file path
 
 @example
 ```ts
 requirePath(writer);
 ```
 */
function requirePath(writer: { readonly path?: string; }, ): string {
  /**
   Path reported by the writer, absent only on the discarding fallback.
   */
  const {path} = writer;
  if (path === undefined)
    throw new Error('expected a spool path');
  return path;
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region spoolDirectory

    describe({
      name: spoolDirectory.name,
      children: [
        it({
          name: 'creates an owner-only directory below the temp root',
          fn: async () => {
            await using root = await createDisposableTempRoot();
            const dir = await spoolDirectory({ tmp: root.path, }, );
            expect(dir, ).toBe(join(root.path, constants.SPOOL_DIR_NAME, ), );
            expect((await stat(dir, )).isDirectory(), ).toBe(true);
            expect(await permissionDigits(dir, ), ).toBe('700');
          },
        }, ),
        it({
          name: 'tightens a directory left with loose permissions',
          fn: async () => {
            await using root = await createDisposableTempRoot();
            const dir = join(root.path, constants.SPOOL_DIR_NAME, );
            await mkdir(dir, { recursive: true, }, );
            await chmod(dir, 0o755, );
            expect(await permissionDigits(dir, ), ).toBe('755');
            await spoolDirectory({ tmp: root.path, }, );
            expect(await permissionDigits(dir, ), ).toBe('700');
          },
        }, ),
      ],
    }, ),

    //endregion spoolDirectory

    //region openSpoolWriter

    describe({
      name: openSpoolWriter.name,
      children: [
        it({
          name: 'writes everything appended and reports the path',
          fn: async () => {
            await using root = await createDisposableTempRoot();
            const writer = await openSpoolWriter({ tmp: root.path, jobId: 'job-1', }, );
            const path = requirePath(writer, );
            expect(path, ).toBe(join(root.path, constants.SPOOL_DIR_NAME, 'job-1.log', ), );
            expect(writer.write('first\n', ), ).toBe(true);
            writer.write('second\n', );
            await writer.close();
            expect(await readFile(path, 'utf8', ), ).toBe('first\nsecond\n');
          },
        }, ),
        it({
          name: 'creates an owner-only file',
          fn: async () => {
            await using root = await createDisposableTempRoot();
            const writer = await openSpoolWriter({ tmp: root.path, jobId: 'job-2', }, );
            writer.write('x', );
            await writer.close();
            expect(
              await permissionDigits(requirePath(writer, ), ),
            ).toBe('600');
          },
        }, ),
        it({
          name: 'falls back to discarding when the temp root is not a directory',
          fn: async () => {
            await using root = await createDisposableTempRoot();
            // A file where the spool directory must go makes mkdir fail, which
            // is the fault path a job must survive.
            const blocker = join(root.path, 'blocker', );
            await writeFile(blocker, 'not a directory', 'utf8', );
            const writer = await openSpoolWriter({ tmp: blocker, jobId: 'job-3', }, );
            expect(writer.path, ).toBeUndefined();
            expect(writer.write('dropped', ), ).toBe(true);
            await writer.close();
          },
        }, ),
      ],
    }, ),

    //endregion openSpoolWriter

    //region discardingSpoolWriter

    describe({
      name: 'discarding spool writer',
      children: [
        it({
          name: 'accepts writes without a path and closes cleanly',
          fn: async () => {
            expect(discardingSpoolWriter.path, ).toBeUndefined();
            expect(discardingSpoolWriter.write('anything', ), ).toBe(true);

            let resumed = false;
            discardingSpoolWriter.onceDrain(function onResumed(): void {
              resumed = true;
            }, );
            expect(resumed, ).toBe(false);
            await discardingSpoolWriter.close();
          },
        }, ),
      ],
    }, ),

    //endregion discardingSpoolWriter
  ],
}, );
