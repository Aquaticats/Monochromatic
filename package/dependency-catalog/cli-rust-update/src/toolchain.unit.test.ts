/**
 Unit tests for choosing the rustup toolchain cargo runs under.

 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cargoToolchain,
  rustToolchainFromLock,
} from './toolchain.ts';

/**
 Creates a disposable repository root.

 @returns root that `await using` removes

 @example
 ```ts
 await using repo = await tempRoot();
 ```
 */
async function tempRoot(): Promise<AsyncDisposable & { readonly root: string; }> {
  /**
   Root directory.
   */
  const root = await mkdtemp(join(scratchRoot(), 'rust-catalog-update-toolchain-',),);
  return {
    root,
    [Symbol.asyncDispose]: async function removeRoot(): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: rustToolchainFromLock.name,
  children: [
    it({
      name: 'reads .tools.rust.version and returns empty without one',
      fn: async () => {
        expect(rustToolchainFromLock('{"tools":{"rust":{"version":"nightly-2026-10-29"}}}',),).toBe('nightly-2026-10-29',);
        expect(rustToolchainFromLock('{"tools":{}}',),).toBe('',);
      },
    },),
  ],
},);

await describe({
  name: cargoToolchain.name,
  children: [
    it({
      name: 'prefers the lock pin over an inherited RUSTUP_TOOLCHAIN, which applies only without a pin',
      fn: async () => {
        await using repo = await tempRoot();
        /**
         Repository root.
         */
        const { root, } = repo;
        expect(await cargoToolchain({ root, explicit: '', },),).toBe('',);
        expect(await cargoToolchain({ root, explicit: '1.98.1', },),).toBe('1.98.1',);
        await writeFile(join(root, 'toolchain.lock.json',), '{"tools":{"rust":{"version":"nightly-2026-10-29"}}}',);
        expect(await cargoToolchain({ root, explicit: '', },),).toBe('nightly-2026-10-29',);
        expect(await cargoToolchain({ root, explicit: '1.98.1', },),).toBe('nightly-2026-10-29',);
      },
    },),
  ],
},);
