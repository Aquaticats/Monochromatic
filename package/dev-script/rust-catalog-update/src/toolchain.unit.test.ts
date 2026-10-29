/**
 Unit tests for choosing the rustup toolchain cargo runs under.

 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

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
  const root = await mkdtemp(join(tmpdir(), 'rust-catalog-update-toolchain-',),);
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
      name: 'prefers RUSTUP_TOOLCHAIN, then the lock pin, then rustup\'s default',
      fn: async () => {
        await using repo = await tempRoot();
        /**
         Repository root.
         */
        const { root, } = repo;
        expect(await cargoToolchain({ root, explicit: '', },),).toBe('',);
        await writeFile(join(root, 'toolchain.lock.json',), '{"tools":{"rust":{"version":"nightly-2026-10-29"}}}',);
        expect(await cargoToolchain({ root, explicit: '', },),).toBe('nightly-2026-10-29',);
        expect(await cargoToolchain({ root, explicit: 'nightly-2026-11-01', },),).toBe('nightly-2026-11-01',);
      },
    },),
  ],
},);
