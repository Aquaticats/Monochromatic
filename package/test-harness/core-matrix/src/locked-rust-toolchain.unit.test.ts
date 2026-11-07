/**
 Tests for reading the locked Rust toolchain from `toolchain.lock.json`.
 Lock files are written into disposable directories, so no test reads the repository's own lock.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  LockedRustToolchainError,
  lockedRustToolchainFromText,
  readLockedRustToolchain,
  TOOLCHAIN_LOCK_FILE_NAME,
} from '../dist/final/node/index.mjs';

/**
 Toolchain name the fixture locks carry.
 */
const FIXTURE_TOOLCHAIN = 'nightly-2000-01-02';

/**
 Lock text with a Rust entry.
 */
const FIXTURE_LOCK = JSON.stringify({ tools: { rust: { version: FIXTURE_TOOLCHAIN, }, }, },);

/**
 Creates a disposable directory that is removed when the `await using` binding ends.

 @returns the directory path and its disposer
 */
async function createScratchDirectory(): Promise<{ readonly path: string; [Symbol.asyncDispose]: () => Promise<void>; }> {
  /**
   Disposable directory under the repository's scratch root.
   */
  const path = await mkdtemp(join(scratchRoot(), 'locked-rust-toolchain-',),);
  return {
    path,
    [Symbol.asyncDispose]: async function cleanup() {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'locked rust toolchain',
  children: [
    describe({
      name: lockedRustToolchainFromText.name,
      children: [
        it({
          name: 'returns the version of the rust tool',
          fn: async () => {
            expect(lockedRustToolchainFromText(FIXTURE_LOCK,),).toBe(FIXTURE_TOOLCHAIN,);
          },
        },),
        it({
          name: 'rejects text that is not JSON',
          fn: async () => {
            expect(() => lockedRustToolchainFromText('{',),).toThrow(LockedRustToolchainError,);
          },
        },),
        it({
          name: 'rejects a lock without a rust entry, with an empty version, and with a null document',
          fn: async () => {
            expect(() => lockedRustToolchainFromText('{"tools":{}}',),).toThrow(LockedRustToolchainError,);
            expect(() => lockedRustToolchainFromText('{"tools":{"rust":{"version":""}}}',),).toThrow(
              LockedRustToolchainError,
            );
            expect(() => lockedRustToolchainFromText('null',),).toThrow(LockedRustToolchainError,);
          },
        },),
      ],
    },),
    describe({
      name: readLockedRustToolchain.name,
      children: [
        it({
          name: 'finds the lock in an ancestor directory',
          fn: async () => {
            await using directory = await createScratchDirectory();
            await writeFile(join(directory.path, TOOLCHAIN_LOCK_FILE_NAME,), FIXTURE_LOCK,);
            /**
             Directory two levels below the lock.
             */
            const nested = join(directory.path, 'one', 'two',);
            await mkdir(nested, { recursive: true, },);
            expect(await readLockedRustToolchain(nested,),).toBe(FIXTURE_TOOLCHAIN,);
          },
        },),
        it({
          name: 'reports a lock without a rust entry instead of looking further up',
          fn: async () => {
            await using directory = await createScratchDirectory();
            await writeFile(join(directory.path, TOOLCHAIN_LOCK_FILE_NAME,), '{"tools":{}}',);
            await expect(readLockedRustToolchain(directory.path,),).rejects.toThrow(LockedRustToolchainError,);
          },
        },),
      ],
    },),
  ],
},);
