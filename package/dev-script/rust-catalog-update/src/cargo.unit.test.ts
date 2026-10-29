/**
 Unit tests for the cutoff, the resolution fallback, and exempt-crate update specs.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  CargoResolveError,
  excludedUpdateSpecs,
  publishTimeCutoff,
  resolveCrate,
  type RunCargo,
  updateExempt,
} from './cargo.ts';

/**
 Fake cargo that answers from a list of outcomes and records each call.

 @param outcomes - `ok` per successive call

 @returns runner and its recorded calls

 @example
 ```ts
 fakeCargo([true,]);
 ```
 */
function fakeCargo(outcomes: readonly boolean[],): { readonly runCargo: RunCargo; readonly calls: string[][]; } {
  /**
   Recorded argument lists.
   */
  const calls: string[][] = [];
  return {
    calls,
    runCargo: async function run({ args, },) {
      calls.push([...args,],);
      return { ok: outcomes[calls.length - 1] ?? true, output: 'error: failed to select a version', };
    },
  };
}

/**
 Lockfile with two registry crates and a path root.
 */
const LOCK = [
  'version = 4',
  '[[package]]',
  'name = "app"',
  'version = "0.1.0"',
  '[[package]]',
  'name = "ring"',
  'version = "0.17.8"',
  'source = "registry+https://github.com/rust-lang/crates.io-index"',
  '[[package]]',
  'name = "tokio-util"',
  'version = "0.7.1"',
  'source = "registry+https://github.com/rust-lang/crates.io-index"',
  '',
].join('\n',);

await describe({
  name: publishTimeCutoff.name,
  children: [
    it({
      name: 'subtracts the release age and drops milliseconds',
      fn: async () => {
        expect(publishTimeCutoff({ now: Date.parse('2026-10-29T12:00:00.789Z',), minutes: 1_440, },),).toBe('2026-10-28T12:00:00Z',);
      },
    },),
  ],
},);

await describe({
  name: resolveCrate.name,
  children: [
    it({
      name: 'resolves with the cutoff when it succeeds',
      fn: async () => {
        /**
         Cargo that succeeds.
         */
        const cargo = fakeCargo([true,],);
        expect(await resolveCrate({ root: '/r', crateDir: 'c', cutoff: '2026-10-28T12:00:00Z', strict: true, runCargo: cargo.runCargo, },),).toEqual({ fellBack: false, },);
        expect(cargo.calls,).toEqual([['-Zunstable-options', 'generate-lockfile', '--publish-time', '2026-10-28T12:00:00Z',],],);
      },
    },),
    it({
      name: 'throws when strict and the constrained resolution fails',
      fn: async () => {
        /**
         Cargo that fails.
         */
        const cargo = fakeCargo([false,],);
        await expect(resolveCrate({ root: '/r', crateDir: 'c', cutoff: '2026-10-28T12:00:00Z', strict: true, runCargo: cargo.runCargo, },),).rejects.toThrow(CargoResolveError,);
        expect(cargo.calls.length,).toBe(1,);
      },
    },),
    it({
      name: 'falls back to an unconstrained resolution when not strict, and throws if that fails too',
      fn: async () => {
        /**
         Cargo that fails, then succeeds.
         */
        const recovering = fakeCargo([false, true,],);
        expect(await resolveCrate({ root: '/r', crateDir: 'c', cutoff: '2026-10-28T12:00:00Z', strict: false, runCargo: recovering.runCargo, },),).toEqual({ fellBack: true, },);
        expect(recovering.calls[1],).toEqual(['generate-lockfile',],);
        /**
         Cargo that fails twice.
         */
        const failing = fakeCargo([false, false,],);
        await expect(resolveCrate({ root: '/r', crateDir: 'c', cutoff: '2026-10-28T12:00:00Z', strict: false, runCargo: failing.runCargo, },),).rejects.toThrow(CargoResolveError,);
      },
    },),
    it({
      name: 'resolves without a cutoff when the release age is zero',
      fn: async () => {
        /**
         Cargo that succeeds.
         */
        const cargo = fakeCargo([true,],);
        await resolveCrate({ root: '/r', crateDir: 'c', cutoff: '', strict: true, runCargo: cargo.runCargo, },);
        expect(cargo.calls,).toEqual([['generate-lockfile',],],);
      },
    },),
  ],
},);

await describe({
  name: excludedUpdateSpecs.name,
  children: [
    it({
      name: 'moves crates matched by name or pattern and ignores version-pinned selectors',
      fn: async () => {
        expect(excludedUpdateSpecs({ lock: LOCK, selectors: ['ring', 'tokio*', 'app',], },),).toEqual(['ring@0.17.8', 'tokio-util@0.7.1',],);
        expect(excludedUpdateSpecs({ lock: LOCK, selectors: ['ring@0.17.8',], },),).toEqual([],);
        expect(excludedUpdateSpecs({ lock: LOCK, selectors: [], },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: updateExempt.name,
  children: [
    it({
      name: 'runs one cargo update for every spec, nothing for none, and throws on failure',
      fn: async () => {
        /**
         Cargo that succeeds, then fails.
         */
        const cargo = fakeCargo([true, false,],);
        await updateExempt({ root: '/r', crateDir: 'c', specs: [], runCargo: cargo.runCargo, },);
        expect(cargo.calls.length,).toBe(0,);
        await updateExempt({ root: '/r', crateDir: 'c', specs: ['ring@0.17.8', 'tokio-util@0.7.1',], runCargo: cargo.runCargo, },);
        expect(cargo.calls[0],).toEqual(['update', 'ring@0.17.8', 'tokio-util@0.7.1',],);
        await expect(updateExempt({ root: '/r', crateDir: 'c', specs: ['ring@0.17.8',], runCargo: cargo.runCargo, },),).rejects.toThrow(CargoResolveError,);
      },
    },),
  ],
},);
