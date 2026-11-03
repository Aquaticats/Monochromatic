/**
 Tests for the lockfile install (the counterpart of `pnpm install`), on disposable repositories.

 The real-cargo test builds a miniature of this repository's layout under a temporary directory:
 standalone crate roots, a `.fuzz` sidecar beside its subject, a virtual workspace, a path crate whose version moves,
 and a directory source that stands in for crates.io, so cargo needs neither the network nor a registry cache.

 @module
 */

import { createHash, } from 'node:crypto';
import {
  mkdir,
  mkdtemp,
  readFile,
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
  createSpawnCargo,
  type RunCargo,
  spawnCargo,
} from './cargo.ts';
import {
  isContinuousIntegration,
  parseMode,
} from './cli.ts';
import type { VersionRecord, } from './policy.ts';
import {
  installRustLockfiles,
  StaleLockfilesError,
} from './install.ts';
import type { Registry, } from './registry.ts';

//region Fixtures

/**
 Fixed clock, well after every fixture release.
 */
const NOW = Date.parse('2026-10-29T12:00:00Z',);

/**
 Arguments that ask cargo whether a lockfile satisfies its manifests, as the owner checks it.
 */
const METADATA_LOCKED_OFFLINE = ['metadata', '--locked', '--offline', '--format-version', '1',] as const;

/**
 Registry dependency whose floor the fixture raises.
 */
const RAISED_CRATE = 'leaf';

/**
 Registry dependency that the fixture leaves at its locked version, so a bump shows as unrelated movement.
 */
const BYSTANDER_CRATE = 'bystander';

/**
 Versions of the fixture crates before and after the manifest change.
 */
const VERSIONS = {
  leafOld: '1.0.0',
  leafNew: '1.1.0',
  bystanderOld: '1.0.0',
  bystanderNew: '1.0.1',
  pathOld: '0.1.0',
  pathNew: '0.2.0',
} as const;

/**
 Repo-relative directory of each fixture crate root, in this repository's layout.
 */
const DIRS = {
  pathCrate: 'package/dep/core',
  standalone: 'package/dep/cli',
  sidecar: 'package/dep/cli.fuzz',
  virtualRoot: 'package/dep/virtual',
  steady: 'package/dep/steady',
} as const;

/**
 Member of the virtual workspace.
 */
const VIRTUAL_MEMBER_DIR = `${DIRS.virtualRoot}/member`;

/**
 Crate roots whose manifests the fixture changes.
 */
const CHANGED_DIRS = [DIRS.standalone, DIRS.sidecar, DIRS.pathCrate, DIRS.virtualRoot,].toSorted();

/**
 Lockfile name.
 */
const LOCKFILE = 'Cargo.lock';

/**
 Catalog with the repository's policy.
 */
const CATALOG_TEXT = [
  'minimumReleaseAge = 1440',
  'minimumReleaseAgeStrict = true',
  'trustPolicy = "no-downgrade"',
  '',
  '[catalog]',
  `${RAISED_CRATE} = ">=${VERSIONS.leafOld}"`,
  '',
].join('\n',);

/**
 Releases by one trusted account, long before the fixture clock.
 */
const TRUSTED_HISTORY: readonly VersionRecord[] = [
  { num: VERSIONS.leafOld, createdAt: '2026-01-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/leaf', },
  { num: VERSIONS.leafNew, createdAt: '2026-02-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/leaf', },
];

/**
 Releases where the newest comes from a new account, after an earlier release made the usual publisher a known one.
 */
const DOWNGRADED_HISTORY: readonly VersionRecord[] = [
  { num: '0.9.0', createdAt: '2025-12-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/leaf', },
  { num: VERSIONS.leafOld, createdAt: '2026-01-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/leaf', },
  { num: VERSIONS.leafNew, createdAt: '2026-02-01T00:00:00Z', publisherId: '9', trustpubRepository: '', repository: 'https://github.com/o/leaf', },
];

/**
 Fake registry with fixed publish times and histories.

 @param history - every release of the crate under test

 @returns registry

 @example
 ```ts
 fakeRegistry(TRUSTED_HISTORY);
 ```
 */
function fakeRegistry(history: readonly VersionRecord[],): Registry {
  return {
    pubtimes: async function pubtimes(): Promise<ReadonlyMap<string, string>> {
      return new Map(history.map(function toTime(entry,): readonly [string, string,] {
        return [entry.num, entry.createdAt,];
      },),);
    },
    history: async function lookup(): Promise<readonly VersionRecord[]> {
      return history;
    },
    prefetch: async function prefetch(): Promise<void> {},
    owners: async function owners(): Promise<ReadonlySet<string>> {
      return new Set();
    },
    yanked: async function yanked(): Promise<ReadonlySet<string>> {
      return new Set();
    },
  };
}

/**
 Writes a file, creating its directory.

 @param path - absolute file path

 @param text - file text

 @example
 ```ts
 await writeFileIn('/r/package/a/Cargo.toml', '[package]\n');
 ```
 */
async function writeFileIn(path: string, text: string,): Promise<void> {
  await mkdir(
    join(
      path,
      '..',
    ),
    { recursive: true, },
  );
  await writeFile(
    path,
    text,
  );
}

/**
 Removes a repository when the test ends.

 @param root - repository root

 @returns disposable that removes it

 @example
 ```ts
 await using repo = disposableRoot(root);
 ```
 */
function disposableRoot(root: string,): AsyncDisposable & { readonly root: string; } {
  return {
    root,
    [Symbol.asyncDispose]: async function removeRoot(): Promise<void> {
      await rm(
        root,
        { recursive: true, force: true, },
      );
    },
  };
}

/**
 Cargo home private to this test run.

 Cargo takes one lock on the package cache under its home for every command, and the shared home's lock is
 held for minutes by other sessions' builds, so a test that used it would queue behind them. The directory
 source of each fixture needs nothing from a cargo home.
 */
const PRIVATE_CARGO_HOME = await mkdtemp(join(
  scratchRoot(),
  'rust-catalog-install-cargo-home-',
),);
process.env.CARGO_HOME = PRIVATE_CARGO_HOME;

/**
 Why the real-cargo tests are skipped, empty when cargo can run.

 The other tests use a fake cargo, so a machine without cargo still runs them.
 */
const CARGO_SKIP_REASON = await (async function probeCargo(): Promise<string> {
  try {
    /**
     Result of asking cargo for its version.
     */
    const probe = await spawnCargo({ cwd: PRIVATE_CARGO_HOME, args: ['--version',], toolchain: '', },);
    return probe.ok ? '' : `cargo --version failed: ${probe.output.trim()}`;
  } catch (error) {
    return `cargo cannot be started: ${String(error,)}`;
  }
})();

//endregion Fixtures

//region Fake cargo

/**
 Lockfile of a one-dependency crate.

 @param version - locked version of `dep`

 @returns lockfile text

 @example
 ```ts
 lock('1.0.0');
 ```
 */
function lock(version: string,): string {
  return [
    'version = 4',
    '',
    '[[package]]',
    'name = "a"',
    'version = "0.1.0"',
    'dependencies = ["dep"]',
    '',
    '[[package]]',
    'name = "dep"',
    `version = "${version}"`,
    'source = "registry+https://github.com/rust-lang/crates.io-index"',
    '',
  ].join('\n',);
}

/**
 Crate roots of the fake-cargo repository.
 */
const FAKE_DIRS = ['package/cli/a', 'package/cli/b',] as const;

/**
 Writes a disposable repository whose crates are locked at `dep` 1.0.0.

 @returns repository that `await using` removes

 @example
 ```ts
 await using repo = await makeFakeRepo();
 ```
 */
async function makeFakeRepo(): Promise<AsyncDisposable & { readonly root: string; }> {
  /**
   Repository root.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'rust-catalog-install-fake-',
  ),);
  await writeFileIn(
    join(
      root,
      'rust-catalog.toml',
    ),
    CATALOG_TEXT,
  );
  for (const dir of FAKE_DIRS) {
    // oxlint-disable-next-line no-await-in-loop -- two small files per crate.
    await writeFileIn(
      join(
        root,
        dir,
        'Cargo.toml',
      ),
      '[package]\nname = "a"\nversion = "0.1.0"\n\n[workspace]\n',
    );
    // oxlint-disable-next-line no-await-in-loop -- two small files per crate.
    await writeFileIn(
      join(
        root,
        dir,
        LOCKFILE,
      ),
      lock(VERSIONS.leafOld,),
    );
  }
  return disposableRoot(root,);
}

/**
 What the fake cargo does for a crate.

 @example
 ```ts
 const behavior: FakeBehavior = { staleDirs: ['package/cli/a',], failingUpdateDir: '', newVersion: '1.1.0', };
 ```
 */
type FakeBehavior = {
  /**
   Crate directories (as path suffixes) whose lockfile is stale.
   */
  readonly staleDirs: readonly string[];
  /**
   Crate directory (as a path suffix) whose update fails, empty for none.
   */
  readonly failingUpdateDir: string;
  /**
   Version a successful update locks.
   */
  readonly newVersion: string;
};

/**
 Fake cargo that records every call and answers the install's three commands.

 A `--locked` update fails in a stale crate; a plain update rewrites the lockfile; metadata always passes.

 @param behavior - what the fake does

 @returns runner and its recorded calls

 @example
 ```ts
 fakeCargo({ staleDirs: [], failingUpdateDir: '', newVersion: '1.1.0', });
 ```
 */
function fakeCargo(behavior: FakeBehavior,): { readonly runCargo: RunCargo; readonly calls: string[]; } {
  /**
   Recorded `<crate dir> <arguments>` lines.
   */
  const calls: string[] = [];
  return {
    calls,
    runCargo: async function run({ cwd, args, },) {
      calls.push(`${cwd.split('/',).slice(-3,).join('/',)} ${args.join(' ',)}`,);
      /**
       Whether this crate's lockfile is stale.
       */
      const stale = behavior.staleDirs.some(function matches(dir: string,): boolean {
        return cwd.endsWith(dir,);
      },);
      if (args[0] !== 'update')
        return { ok: true, output: '', };
      if (args.includes('--locked',))
        return stale ? { ok: false, output: 'error: cannot update the lock file because --locked was passed', } : { ok: true, output: '', };
      if ((behavior.failingUpdateDir !== '') && cwd.endsWith(behavior.failingUpdateDir,))
        return { ok: false, output: 'error: failed to select a version for `dep`', };
      await writeFile(
        join(
          cwd,
          LOCKFILE,
        ),
        lock(behavior.newVersion,),
      );
      return { ok: true, output: '', };
    },
  };
}

//endregion Fake cargo

//region Real cargo

/**
 Manifest of a fixture crate that is its own workspace root.

 @param name - package name

 @param version - package version

 @param dependencies - lines of the `[dependencies]` table

 @param leafFloor - floor of the `leaf` entry in `[workspace.dependencies]`

 @returns manifest text

 @example
 ```ts
 crateManifest({ name: 'dep-cli', version: '0.1.0', dependencies: [], leafFloor: '1.0.0', });
 ```
 */
function crateManifest(
  {
    name,
    version,
    dependencies,
    leafFloor,
  }: {
    readonly name: string;
    readonly version: string;
    readonly dependencies: readonly string[];
    readonly leafFloor: string;
  },
): string {
  return [
    '[package]',
    `name = "${name}"`,
    `version = "${version}"`,
    'edition = "2021"',
    '',
    '[lib]',
    'path = "src/lib.rs"',
    '',
    '[dependencies]',
    ...dependencies,
    '',
    '[workspace]',
    '',
    '[workspace.dependencies]',
    `${RAISED_CRATE} = ">=${leafFloor}"`,
    '',
  ].join('\n',);
}

/**
 Writes one crate of the directory source that stands in for crates.io.

 @param vendorDir - directory source

 @param name - crate name

 @param version - crate version

 @example
 ```ts
 await vendorCrate({ vendorDir, name: 'leaf', version: '1.0.0', });
 ```
 */
async function vendorCrate(
  {
    vendorDir,
    name,
    version,
  }: {
    readonly vendorDir: string;
    readonly name: string;
    readonly version: string;
  },
): Promise<void> {
  /**
   Crate directory in the source.
   */
  const dir = join(
    vendorDir,
    `${name}-${version}`,
  );
  await writeFileIn(
    join(
      dir,
      'Cargo.toml',
    ),
    `[package]\nname = "${name}"\nversion = "${version}"\nedition = "2021"\n`,
  );
  await writeFileIn(
    join(
      dir,
      'src/lib.rs',
    ),
    '',
  );
  await writeFileIn(
    join(
      dir,
      '.cargo-checksum.json',
    ),
    JSON.stringify({ files: {}, package: createHash('sha256',)
      .update(`${name}@${version}`,)
      .digest('hex',), },),
  );
}

/**
 Writes every manifest of the fixture at one stage.

 @param root - repository root

 @param stage - `before` is the state the lockfiles are generated in; `after` is the state a catalog bump leaves

 @example
 ```ts
 await writeManifests({ root, stage: 'after', });
 ```
 */
async function writeManifests(
  {
    root,
    stage,
  }: {
    readonly root: string;
    readonly stage: 'before' | 'after';
  },
): Promise<void> {
  /**
   Whether the catalog bump has happened.
   */
  const bumped = stage === 'after';
  /**
   Floor `sync:files` renders into the manifests.
   */
  const leafFloor = bumped ? VERSIONS.leafNew : VERSIONS.leafOld;
  /**
   Version of the path crate, which `sync:files` also carries into its dependents as `=<version>`.
   */
  const pathVersion = bumped ? VERSIONS.pathNew : VERSIONS.pathOld;
  /**
   Dependency on the path crate.
   */
  const pathDependency = `dep-core = { path = "../core", version = "=${pathVersion}" }`;
  /**
   Dependency on the raised crate through the generated table.
   */
  const leafDependency = `${RAISED_CRATE} = { workspace = true }`;
  /**
   Dependency on the crate that must not move.
   */
  const bystanderDependency = `${BYSTANDER_CRATE} = ">=${VERSIONS.bystanderOld}"`;
  await writeFileIn(
    join(root, DIRS.pathCrate, 'Cargo.toml',),
    crateManifest({ name: 'dep-core', version: pathVersion, dependencies: [], leafFloor, },),
  );
  await writeFileIn(
    join(root, DIRS.standalone, 'Cargo.toml',),
    crateManifest({ name: 'dep-cli', version: VERSIONS.pathOld, dependencies: [pathDependency, leafDependency, bystanderDependency,], leafFloor, },),
  );
  await writeFileIn(
    join(root, DIRS.sidecar, 'Cargo.toml',),
    crateManifest({ name: 'dep-cli-fuzz', version: VERSIONS.pathOld, dependencies: [`dep-cli = { path = "../cli", version = "=${VERSIONS.pathOld}" }`, leafDependency,], leafFloor, },),
  );
  await writeFileIn(
    join(root, DIRS.virtualRoot, 'Cargo.toml',),
    `[workspace]\nmembers = ["member"]\nresolver = "2"\n\n[workspace.dependencies]\n${RAISED_CRATE} = ">=${leafFloor}"\n`,
  );
  await writeFileIn(
    join(root, VIRTUAL_MEMBER_DIR, 'Cargo.toml',),
    `[package]\nname = "dep-member"\nversion = "${VERSIONS.pathOld}"\nedition = "2021"\n\n[dependencies]\n${leafDependency}\n`,
  );
  await writeFileIn(
    join(root, VIRTUAL_MEMBER_DIR, 'src/lib.rs',),
    '',
  );
  await writeFileIn(
    join(root, DIRS.steady, 'Cargo.toml',),
    crateManifest({ name: 'dep-steady', version: VERSIONS.pathOld, dependencies: [bystanderDependency,], leafFloor: VERSIONS.leafOld, },),
  );
  for (const dir of [DIRS.pathCrate, DIRS.standalone, DIRS.sidecar, DIRS.steady,])
    // oxlint-disable-next-line no-await-in-loop -- one empty file per crate.
    await writeFileIn(
      join(root, dir, 'src/lib.rs',),
      '',
    );
}

/**
 Whether cargo accepts the crate's lockfile as it is, the owner's check.

 @param cwd - crate root

 @returns whether `cargo metadata --locked --offline` exits 0

 @example
 ```ts
 await lockIsConsistent('/r/package/dep/cli');
 ```
 */
async function lockIsConsistent(cwd: string,): Promise<boolean> {
  return (await spawnCargo({ cwd, args: METADATA_LOCKED_OFFLINE, toolchain: '', },)).ok;
}

/**
 Builds the miniature repository with every lockfile consistent, then applies the catalog bump to the manifests only.

 The bump is the state `mise run sync:files` leaves: floors and path-crate versions rendered into manifests,
 lockfiles untouched, newer releases now present in the (offline) source.

 @returns repository that `await using` removes

 @example
 ```ts
 await using repo = await makeStaleRepo();
 ```
 */
async function makeStaleRepo(): Promise<AsyncDisposable & { readonly root: string; }> {
  /**
   Repository root.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'rust-catalog-install-real-',
  ),);
  /**
   Directory source that replaces crates.io.
   */
  const vendorDir = join(
    root,
    'vendor',
  );
  await writeFileIn(
    join(root, '.cargo/config.toml',),
    `[source.crates-io]\nreplace-with = "fixture-vendor"\n\n[source.fixture-vendor]\ndirectory = ${JSON.stringify(vendorDir,)}\n`,
  );
  await writeFileIn(
    join(root, 'rust-catalog.toml',),
    CATALOG_TEXT,
  );
  await vendorCrate({ vendorDir, name: RAISED_CRATE, version: VERSIONS.leafOld, },);
  await vendorCrate({ vendorDir, name: BYSTANDER_CRATE, version: VERSIONS.bystanderOld, },);
  await writeManifests({ root, stage: 'before', },);
  for (const dir of Object.values(DIRS,)) {
    // oxlint-disable-next-line no-await-in-loop -- cargo serializes on its package cache lock anyway.
    const generated = await spawnCargo({ cwd: join(root, dir,), args: ['generate-lockfile', '--offline',], toolchain: '', },);
    expect(generated.ok,).toBe(true,);
  }
  await vendorCrate({ vendorDir, name: RAISED_CRATE, version: VERSIONS.leafNew, },);
  await vendorCrate({ vendorDir, name: BYSTANDER_CRATE, version: VERSIONS.bystanderNew, },);
  await writeManifests({ root, stage: 'after', },);
  return disposableRoot(root,);
}

/**
 Locked version of a registry crate in a lockfile text.

 @param lockText - `Cargo.lock` text

 @param name - crate name

 @returns the version, empty when the crate is not locked

 @example
 ```ts
 lockedVersion({ lockText, name: 'leaf', }); // '1.1.0'
 ```
 */
function lockedVersion({ lockText, name, }: { readonly lockText: string; readonly name: string; },): string {
  /**
   Text that precedes the version of this package in a lockfile entry.
   */
  const marker = `name = "${name}"\nversion = "`;
  /**
   Text from the version on; empty when the crate is not locked.
   */
  const [, afterMarker = '',] = lockText.split(marker,);
  return afterMarker.split('"',)[0] ?? '';
}

//endregion Real cargo

await describe({
  name: installRustLockfiles.name,
  children: [
    it({
      name: 'updates the lockfile of every workspace a catalog bump leaves stale, with minimal movement, under real cargo',
      skip: CARGO_SKIP_REASON === '' ? false : CARGO_SKIP_REASON,
      fn: async () => {
        /**
         Repository whose manifests were bumped.
         */
        await using repo = await makeStaleRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        for (const dir of CHANGED_DIRS) {
          /**
           Whether cargo accepts this crate's lockfile before the install.
           */
          // oxlint-disable-next-line no-await-in-loop -- cargo serializes on its package cache lock anyway.
          const consistent = await lockIsConsistent(join(root, dir,),);
          expect(consistent,).toBe(false,);
        }
        /**
         Whether cargo accepts the steady crate's lockfile, which no change touched.
         */
        const steadyConsistent = await lockIsConsistent(join(root, DIRS.steady,),);
        expect(steadyConsistent,).toBe(true,);
        /**
         Steady crate's lockfile before the install.
         */
        const steadyBefore = await readFile(join(root, DIRS.steady, LOCKFILE,), 'utf8',);
        /**
         Standalone crate's lockfile before the install.
         */
        const standaloneBefore = await readFile(join(root, DIRS.standalone, LOCKFILE,), 'utf8',);
        /**
         Install outcome.
         */
        const report = await installRustLockfiles({ root, frozenLockfile: false, dryRun: false, now: NOW, runCargo: createSpawnCargo({ toolchain: '', },), registry: fakeRegistry(TRUSTED_HISTORY,), },);
        expect(report.problems,).toEqual([],);
        expect(report.written,).toBe(true,);
        expect(report.stale,).toEqual(CHANGED_DIRS,);
        expect(report.added,).toEqual([{ name: RAISED_CRATE, version: VERSIONS.leafNew, },],);
        for (const dir of Object.values(DIRS,)) {
          /**
           Whether cargo accepts this crate's lockfile after the install.
           */
          // oxlint-disable-next-line no-await-in-loop -- cargo serializes on its package cache lock anyway.
          const consistent = await lockIsConsistent(join(root, dir,),);
          expect(consistent,).toBe(true,);
        }
        expect(await readFile(join(root, DIRS.steady, LOCKFILE,), 'utf8',),).toBe(steadyBefore,);
        /**
         Standalone crate's lockfile after the install.
         */
        const standaloneAfter = await readFile(join(root, DIRS.standalone, LOCKFILE,), 'utf8',);
        expect(lockedVersion({ lockText: standaloneBefore, name: RAISED_CRATE, },),).toBe(VERSIONS.leafOld,);
        expect(lockedVersion({ lockText: standaloneAfter, name: RAISED_CRATE, },),).toBe(VERSIONS.leafNew,);
        expect(lockedVersion({ lockText: standaloneAfter, name: BYSTANDER_CRATE, },),).toBe(VERSIONS.bystanderOld,);
      },
    },),
    it({
      name: 'restores every lockfile on a dry run and still reports what it would regenerate',
      skip: CARGO_SKIP_REASON === '' ? false : CARGO_SKIP_REASON,
      fn: async () => {
        /**
         Repository whose manifests were bumped.
         */
        await using repo = await makeStaleRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Lockfile before the dry run.
         */
        const before = await readFile(join(root, DIRS.sidecar, LOCKFILE,), 'utf8',);
        /**
         Install outcome.
         */
        const report = await installRustLockfiles({ root, frozenLockfile: false, dryRun: true, now: NOW, runCargo: createSpawnCargo({ toolchain: '', },), registry: fakeRegistry(TRUSTED_HISTORY,), },);
        expect(report.written,).toBe(false,);
        expect(report.stale,).toEqual(CHANGED_DIRS,);
        expect(await readFile(join(root, DIRS.sidecar, LOCKFILE,), 'utf8',),).toBe(before,);
        /**
         Whether cargo accepts the sidecar's lockfile, which the dry run restored.
         */
        const sidecarConsistent = await lockIsConsistent(join(root, DIRS.sidecar,),);
        expect(sidecarConsistent,).toBe(false,);
      },
    },),
    it({
      name: 'runs no update in a crate whose lockfile already satisfies its manifests',
      fn: async () => {
        /**
         Repository with consistent lockfiles.
         */
        await using repo = await makeFakeRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Cargo that finds every lockfile consistent.
         */
        const cargo = fakeCargo({ staleDirs: [], failingUpdateDir: '', newVersion: VERSIONS.leafNew, },);
        /**
         Install outcome.
         */
        const report = await installRustLockfiles({ root, frozenLockfile: false, dryRun: false, now: NOW, runCargo: cargo.runCargo, registry: fakeRegistry(TRUSTED_HISTORY,), },);
        expect(report.stale,).toEqual([],);
        expect(report.written,).toBe(true,);
        expect(cargo.calls.every(function isCheck(call: string,): boolean {
          return call.includes('--locked',);
        },),).toBe(true,);
        expect(await readFile(join(root, FAKE_DIRS[0], LOCKFILE,), 'utf8',),).toBe(lock(VERSIONS.leafOld,),);
      },
    },),
    it({
      name: 'updates offline and only the workspace, then verifies with the owner\'s cargo metadata check',
      fn: async () => {
        /**
         Repository where the first crate is stale.
         */
        await using repo = await makeFakeRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Cargo that finds the first crate stale.
         */
        const cargo = fakeCargo({ staleDirs: [FAKE_DIRS[0],], failingUpdateDir: '', newVersion: VERSIONS.leafNew, },);
        await installRustLockfiles({ root, frozenLockfile: false, dryRun: false, now: NOW, runCargo: cargo.runCargo, registry: fakeRegistry(TRUSTED_HISTORY,), },);
        expect(cargo.calls,).toEqual([
          `${FAKE_DIRS[0]} update --workspace --locked --offline`,
          `${FAKE_DIRS[0]} update --workspace --offline`,
          `${FAKE_DIRS[0]} ${METADATA_LOCKED_OFFLINE.join(' ',)}`,
          `${FAKE_DIRS[1]} update --workspace --locked --offline`,
        ],);
      },
    },),
    it({
      name: 'restores every lockfile and names the crate when one update fails',
      fn: async () => {
        /**
         Repository where both crates are stale.
         */
        await using repo = await makeFakeRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Cargo whose second update fails after the first rewrote its lockfile.
         */
        const cargo = fakeCargo({ staleDirs: [...FAKE_DIRS,], failingUpdateDir: FAKE_DIRS[1], newVersion: VERSIONS.leafNew, },);
        await expect(installRustLockfiles({ root, frozenLockfile: false, dryRun: false, now: NOW, runCargo: cargo.runCargo, registry: fakeRegistry(TRUSTED_HISTORY,), },),).rejects.toThrow(FAKE_DIRS[1],);
        for (const dir of FAKE_DIRS)
          // oxlint-disable-next-line no-await-in-loop -- two small files.
          expect(await readFile(join(root, dir, LOCKFILE,), 'utf8',),).toBe(lock(VERSIONS.leafOld,),);
      },
    },),
    it({
      name: 'restores every lockfile when a newly locked version fails the trust policy',
      fn: async () => {
        /**
         Repository where the first crate is stale.
         */
        await using repo = await makeFakeRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Cargo that locks the newest release, which a new account published.
         */
        const cargo = fakeCargo({ staleDirs: [FAKE_DIRS[0],], failingUpdateDir: '', newVersion: VERSIONS.leafNew, },);
        /**
         Install outcome.
         */
        const report = await installRustLockfiles({ root, frozenLockfile: false, dryRun: false, now: NOW, runCargo: cargo.runCargo, registry: fakeRegistry(DOWNGRADED_HISTORY,), },);
        expect(report.written,).toBe(false,);
        expect(report.problems.length,).toBe(1,);
        expect(await readFile(join(root, FAKE_DIRS[0], LOCKFILE,), 'utf8',),).toBe(lock(VERSIONS.leafOld,),);
      },
    },),
  ],
},);

await describe({
  name: `${installRustLockfiles.name} with --frozen-lockfile`,
  children: [
    it({
      name: 'fails naming every stale root and writes nothing, as pnpm install --frozen-lockfile does',
      fn: async () => {
        /**
         Repository whose manifests were bumped.
         */
        await using repo = await makeStaleRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Lockfile before the install.
         */
        const before = await readFile(join(root, DIRS.sidecar, LOCKFILE,), 'utf8',);
        /**
         Failure the frozen install raises, undefined if it did not fail.
         */
        let failure: unknown = undefined;
        try {
          await installRustLockfiles({ root, frozenLockfile: true, dryRun: false, now: NOW, runCargo: createSpawnCargo({ toolchain: '', },), registry: fakeRegistry(TRUSTED_HISTORY,), },);
        } catch (error) {
          failure = error;
        }
        expect(failure,).toBeInstanceOf(StaleLockfilesError,);
        expect(failure instanceof StaleLockfilesError ? failure.staleDirs : [],).toEqual(CHANGED_DIRS,);
        expect(String(failure,),).toContain(DIRS.sidecar,);
        expect(await readFile(join(root, DIRS.sidecar, LOCKFILE,), 'utf8',),).toBe(before,);
      },
      skip: CARGO_SKIP_REASON === '' ? false : CARGO_SKIP_REASON,
    },),
    it({
      name: 'passes, running only read-only checks, when every lockfile satisfies its manifests',
      fn: async () => {
        /**
         Repository with consistent lockfiles.
         */
        await using repo = await makeFakeRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Cargo that finds every lockfile consistent.
         */
        const cargo = fakeCargo({ staleDirs: [], failingUpdateDir: '', newVersion: VERSIONS.leafNew, },);
        /**
         Install outcome.
         */
        const report = await installRustLockfiles({ root, frozenLockfile: true, dryRun: false, now: NOW, runCargo: cargo.runCargo, registry: fakeRegistry(TRUSTED_HISTORY,), },);
        expect(report.stale,).toEqual([],);
        expect(cargo.calls.length,).toBe(FAKE_DIRS.length,);
        expect(cargo.calls.every(function isCheck(call: string,): boolean {
          return call.includes('--locked',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'checks every root, then fails without any cargo update when a root is stale',
      fn: async () => {
        /**
         Repository where the first crate is stale.
         */
        await using repo = await makeFakeRepo();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Cargo that finds the first crate stale.
         */
        const cargo = fakeCargo({ staleDirs: [FAKE_DIRS[0],], failingUpdateDir: '', newVersion: VERSIONS.leafNew, },);
        await expect(installRustLockfiles({ root, frozenLockfile: true, dryRun: false, now: NOW, runCargo: cargo.runCargo, registry: fakeRegistry(TRUSTED_HISTORY,), },),).rejects.toThrow(FAKE_DIRS[0],);
        expect(cargo.calls,).toEqual([
          `${FAKE_DIRS[0]} update --workspace --locked --offline`,
          `${FAKE_DIRS[1]} update --workspace --locked --offline`,
        ],);
      },
    },),
  ],
},);

await describe({
  name: parseMode.name,
  children: [
    it({
      name: 'selects install like pnpm install, with --frozen-lockfile, --no-frozen-lockfile and --dry-run, and rejects other combinations',
      fn: async () => {
        expect(parseMode(['install',],),).toEqual({ mode: 'install', base: 'HEAD', dryRun: false, },);
        expect(parseMode(['install', '--frozen-lockfile',],),).toEqual({ mode: 'install', base: 'HEAD', frozenLockfile: true, dryRun: false, },);
        expect(parseMode(['install', '--no-frozen-lockfile', '--dry-run',],),).toEqual({ mode: 'install', base: 'HEAD', frozenLockfile: false, dryRun: true, },);
        expect(function conflicting(): void {
          parseMode(['install', '--frozen-lockfile', '--no-frozen-lockfile',],);
        },).toThrow('install',);
        expect(function unknown(): void {
          parseMode(['install', '--verify',],);
        },).toThrow('install',);
        expect(function removedKeyword(): void {
          parseMode(['--reconcile',],);
        },).toThrow('unknown argument',);
      },
    },),
  ],
},);

await describe({
  name: isContinuousIntegration.name,
  children: [
    it({
      name: 'is true for a CI variable other than empty, 0 or false, as pnpm defaults --frozen-lockfile in CI',
      fn: async () => {
        expect(isContinuousIntegration('true',),).toBe(true,);
        expect(isContinuousIntegration('1',),).toBe(true,);
        expect(isContinuousIntegration('',),).toBe(false,);
        expect(isContinuousIntegration('FALSE',),).toBe(false,);
        expect(isContinuousIntegration('0',),).toBe(false,);
      },
    },),
  ],
},);

await rm(
  PRIVATE_CARGO_HOME,
  { recursive: true, force: true, },
);
