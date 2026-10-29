/**
 Unit tests for pruning superseded versions of the repository's own crates.
 */

import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isOwnCrate,
  pruneStaleOwnCrates,
  versionOfEntry,
} from './prune.ts';
import { toolById, } from './plan.ts';

/**
 Registry index directory name the fixture uses.
 */
const INDEX = 'index.crates.io-fixture';

/**
 Scratch root for fixtures: the package's gitignored `.cache`, never the system temporary directory.
 */
const SCRATCH_ROOT = join(
  import.meta.dirname,
  '..',
  '..',
  '.cache',
  'prune-test',
);

/**
 Creates one disposable directory under the scratch root.

 @param prefix - Name prefix of the directory

 @returns Absolute path of the new directory
 */
function scratchDir(prefix: string,): string {
  mkdirSync(
    SCRATCH_ROOT,
    { recursive: true },
  );
  return mkdtempSync(join(
    SCRATCH_ROOT,
    prefix,
  ),);
}

/**
 Builds a disposable cargo home holding cached archives and extracted sources.

 @param versions - Entry names to create, as `<crate>-<version>`

 @returns The fixture's environment and the paths of its two registry index directories
 */
function fixtureCargoHome(
  { versions, }: { readonly versions: readonly string[]; },
): {
  readonly env: { readonly CARGO_HOME: string; };
  readonly cacheDir: string;
  readonly srcDir: string;
} {
  /**
   Disposable cargo home.
   */
  const cargoHome = scratchDir('toolchain-prune-',);
  /**
   Cached archive directory of the fixture index.
   */
  const cacheDir = join(
    cargoHome,
    'registry',
    'cache',
    INDEX,
  );
  /**
   Extracted source directory of the fixture index.
   */
  const srcDir = join(
    cargoHome,
    'registry',
    'src',
    INDEX,
  );
  mkdirSync(
    cacheDir,
    { recursive: true },
  );
  mkdirSync(
    srcDir,
    { recursive: true },
  );
  for (const name of versions) {
    writeFileSync(
      join(
        cacheDir,
        `${name}.crate`,
      ),
      '',
    );
    mkdirSync(
      join(
        srcDir,
        name,
      ),
    );
  }
  return {
    env: { CARGO_HOME: cargoHome },
    cacheDir,
    srcDir,
  };
}

await describe({
  name: 'prune',
  children: [
    describe({
      name: versionOfEntry.name,
      children: [
        it({
          name: 'reads the version after the crate name',
          fn: async () => {
            expect(versionOfEntry({
              name: 'monochromatic-lint-0.1.0',
              crate: 'monochromatic-lint',
            },),)
              .toBe('0.1.0',);
          },
        },),
        it({
          name: 'keeps a pre-release suffix in the version',
          fn: async () => {
            expect(versionOfEntry({
              name: 'monochromatic-lint-0.5.0-rc.1',
              crate: 'monochromatic-lint',
            },),)
              .toBe('0.5.0-rc.1',);
          },
        },),
        it({
          name: 'rejects a longer crate name sharing the prefix',
          fn: async () => {
            expect(versionOfEntry({
              name: 'monochromatic-lint-closure-0.1.0',
              crate: 'monochromatic-lint',
            },),)
              .toBeNull();
          },
        },),
        it({
          name: 'rejects another crate',
          fn: async () => {
            expect(versionOfEntry({
              name: 'serde-1.0.0',
              crate: 'monochromatic-lint',
            },),)
              .toBeNull();
          },
        },),
      ],
    },),
    describe({
      name: isOwnCrate.name,
      children: [
        it({
          name: 'admits a monochromatic crate installed through cargo-binstall',
          fn: async () => {
            expect(isOwnCrate(toolById('cargo:monochromatic-lint',),),)
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects a third-party crate',
          fn: async () => {
            expect(isOwnCrate(toolById('cargo:cargo-about',),),)
              .toBe(false,);
          },
        },),
        it({
          name: 'rejects a tool on another route',
          fn: async () => {
            expect(isOwnCrate(toolById('node',),),)
              .toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: pruneStaleOwnCrates.name,
      children: [
        it({
          name: 'deletes superseded archives and sources and keeps the locked version',
          fn: async () => {
            /**
             Plan entry whose locked version is kept.
             */
            const entry = toolById('cargo:monochromatic-lint',);
            /**
             Fixture holding two superseded versions and the locked one.
             */
            const fixture = fixtureCargoHome({
              versions: [
                'monochromatic-lint-0.1.0',
                'monochromatic-lint-0.3.0',
                `monochromatic-lint-${entry.version}`,
              ],
            },);
            /**
             Paths that were deleted.
             */
            const deleted = await pruneStaleOwnCrates({
              entry,
              env: fixture.env,
            },);
            expect(deleted,)
              .toHaveLength(4,);
            expect(existsSync(join(
              fixture.cacheDir,
              'monochromatic-lint-0.1.0.crate',
            ),),)
              .toBe(false,);
            expect(existsSync(join(
              fixture.srcDir,
              'monochromatic-lint-0.3.0',
            ),),)
              .toBe(false,);
            expect(existsSync(join(
              fixture.cacheDir,
              `monochromatic-lint-${entry.version}.crate`,
            ),),)
              .toBe(true,);
            expect(existsSync(join(
              fixture.srcDir,
              `monochromatic-lint-${entry.version}`,
            ),),)
              .toBe(true,);
          },
        },),
        it({
          name: 'leaves other crates, including one sharing the name prefix',
          fn: async () => {
            /**
             Fixture holding a stale own version beside unrelated crates.
             */
            const fixture = fixtureCargoHome({
              versions: [
                'monochromatic-lint-0.1.0',
                'monochromatic-lint-closure-0.1.0',
                'serde-1.0.0',
              ],
            },);
            await pruneStaleOwnCrates({
              entry: toolById('cargo:monochromatic-lint',),
              env: fixture.env,
            },);
            expect(existsSync(join(
              fixture.cacheDir,
              'monochromatic-lint-closure-0.1.0.crate',
            ),),)
              .toBe(true,);
            expect(existsSync(join(
              fixture.srcDir,
              'serde-1.0.0',
            ),),)
              .toBe(true,);
          },
        },),
        it({
          name: 'deletes nothing for a third-party crate',
          fn: async () => {
            /**
             Fixture holding an old version of a crate this repository does not publish.
             */
            const fixture = fixtureCargoHome({ versions: ['cargo-about-0.8.0'], },);
            /**
             Paths that were deleted.
             */
            const deleted = await pruneStaleOwnCrates({
              entry: toolById('cargo:cargo-about',),
              env: fixture.env,
            },);
            expect(deleted,)
              .toEqual([],);
            expect(existsSync(join(
              fixture.cacheDir,
              'cargo-about-0.8.0.crate',
            ),),)
              .toBe(true,);
          },
        },),
        it({
          name: 'deletes nothing when the registry does not exist',
          fn: async () => {
            /**
             Cargo home without a registry directory.
             */
            const cargoHome = scratchDir('toolchain-prune-empty-',);
            expect(await pruneStaleOwnCrates({
              entry: toolById('cargo:monochromatic-lint',),
              env: { CARGO_HOME: cargoHome },
            },),)
              .toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
