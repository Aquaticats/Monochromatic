/**
 Container integration test for {@link ensurePackage}.
 Runs inside a container via the matrix runner; never on the host.
 
 Test matrix (managed by the matrix runner):
 - ubuntu:latest (apt) as root and user
 - cachyos:latest (pacman) as root and user
 
 Package shapes tested:
 - String shorthand (`p('tree')`): binary = effname = package name
 - Bin differs from effname (`p({ bin: 'rg', effname: 'ripgrep' })`)
 - Per-manager overrides via yes tuples (`yes: [['dnf', 'ImageMagick']]`)
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  binaryExists,
  detectManager,
  ensurePackage,
  NO_PACKAGE_MANAGER,
  p,
  registerPackages,
  resetManagerCache,
} from '../../dist/final/node/index.mjs';

//region Test packages

/**
 Varied package shapes for testing.
 Each shape exercises a different code path in {@link ensurePackage}.
 */
const TEST_PACKAGES = [
  /** Shape: string shorthand (binary = effname = package name everywhere) */
  p('tree',),
  p('jq',),
  /** Shape: bin differs from effname */
  p({
    bin: 'rg',
    effname: 'ripgrep',
  },),
  /** Shape: per-manager override where name varies (via yes tuples) */
  p({
    bin: 'convert',
    effname: 'imagemagick',
    yes: ['apt', ['dnf', 'ImageMagick',],],
  },),
  /** Shape: effname only (bin defaults to effname) */
  p({ effname: 'strace', },),
] as const;

//endregion Test packages

//region Setup

resetManagerCache();
registerPackages([...TEST_PACKAGES,],);

const manager = await detectManager();
console.log(
  `[container-test] detected manager: ${manager === NO_PACKAGE_MANAGER ? 'none' : manager}`,
);
console.log(`[container-test] uid: ${String(process.getuid?.() ?? 'unavailable',)}`,);

//endregion Setup

await describe({
  name:
    'ensurePackage (container) -- sequential: idempotent test depends on prior install, package manager locks prevent concurrent installs',
  concurrency: 1,
  children: [
    //region String shorthand shape

    it({
      name: 'tree (string shorthand)',
      fn: async () => {
        const before = await binaryExists({ binary: 'tree', },);
        if (before)
          return;
        await ensurePackage('tree',);
        const after = await binaryExists({ binary: 'tree', },);
        expect(after,).toBe(true,);
      },
    },),

    it({
      name: 'jq (string shorthand)',
      fn: async () => {
        const before = await binaryExists({ binary: 'jq', },);
        if (before)
          return;
        await ensurePackage('jq',);
        const after = await binaryExists({ binary: 'jq', },);
        expect(after,).toBe(true,);
      },
    },),

    //endregion String shorthand shape

    //region Bin differs from effname

    it({
      name: 'rg (bin != effname)',
      fn: async () => {
        const before = await binaryExists({ binary: 'rg', },);
        if (before)
          return;
        await ensurePackage('rg',);
        const after = await binaryExists({ binary: 'rg', },);
        expect(after,).toBe(true,);
      },
    },),

    //endregion Bin differs from effname

    //region Per-manager override

    it({
      name: 'convert (manager override)',
      fn: async () => {
        const before = await binaryExists({ binary: 'convert', },);
        if (before)
          return;
        await ensurePackage('convert',);
        const after = await binaryExists({ binary: 'convert', },);
        expect(after,).toBe(true,);
      },
    },),

    //endregion Per-manager override

    //region Effname only

    it({
      name: 'strace (effname only)',
      fn: async () => {
        const before = await binaryExists({ binary: 'strace', },);
        if (before)
          return;
        await ensurePackage('strace',);
        const after = await binaryExists({ binary: 'strace', },);
        expect(after,).toBe(true,);
      },
    },),

    //endregion Effname only

    //region Idempotent second call

    it({
      name: 'idempotent second call (tree)',
      fn: async () => {
        /** tree was installed by the earlier test case. */
        await expect(ensurePackage('tree',),).resolves.toBeUndefined();
      },
    },),

    //endregion Idempotent second call

    //region Unknown binary

    it({
      name: 'unknown binary throws',
      fn: async () => {
        let caught: unknown;
        try {
          await ensurePackage('nonexistent-binary-that-should-not-exist-42',);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
      },
    },),
    //endregion Unknown binary
  ],
},);
