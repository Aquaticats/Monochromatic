/**
 Transaction tests for the recursive update and the CI verification, on disposable repositories.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RunCargo, } from './cargo.ts';
import { parseMode, } from './cli.ts';
import type { VersionRecord, } from './policy.ts';
import { pruneExcludes, } from './prune.ts';
import type { Registry, } from './registry.ts';
import { updateRustLockfiles, } from './update.ts';
import {
  type ReadBaseLock,
  verifyRustLockfiles,
} from './verify.ts';

//region Fixtures

/**
 Fixed clock.
 */
const NOW = Date.parse('2026-10-29T12:00:00Z',);

/**
 Lockfile locking `dep` at a version.

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
 Catalog with the repository's policy and optional extra settings lines.

 @param extra - lines replacing the default exclude lists

 @returns catalog text

 @example
 ```ts
 catalog([]);
 ```
 */
function catalog(extra: readonly string[],): string {
  return [
    'minimumReleaseAge = 1440',
    'minimumReleaseAgeStrict = true',
    'minimumReleaseAgeExcludePrune = true',
    'trustPolicy = "no-downgrade"',
    'trustPolicyExcludePrune = true',
    ...extra,
    '',
    '[catalog]',
    'dep = ">=1"',
    '',
  ].join('\n',);
}

/**
 Writes a disposable repository with one crate locked at `dep` 1.0.0.

 @param catalogText - `rust-catalog.toml` text

 @returns repository root that `await using` removes

 @example
 ```ts
 await using repo = await makeRepo(catalog([]));
 ```
 */
async function makeRepo(catalogText: string,): Promise<AsyncDisposable & { readonly root: string; }> {
  /**
   Repository root.
   */
  const root = await mkdtemp(join(tmpdir(), 'rust-catalog-update-',),);
  await mkdir(join(root, 'package/cli/a',), { recursive: true, },);
  await writeFile(join(root, 'rust-catalog.toml',), catalogText,);
  await writeFile(join(root, 'package/cli/a/Cargo.toml',), '[package]\nname = "a"\nversion = "0.1.0"\n\n[workspace]\n',);
  await writeFile(join(root, 'package/cli/a/Cargo.lock',), lock('1.0.0',),);
  return {
    root,
    [Symbol.asyncDispose]: async function removeRepo(): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Fake cargo whose resolution locks `dep` at a version, optionally failing.

 @param version - version the resolution locks

 @param ok - whether cargo succeeds

 @returns runner

 @example
 ```ts
 resolvingTo({ version: '1.0.1', ok: true, });
 ```
 */
function resolvingTo({ version, ok, }: { readonly version: string; readonly ok: boolean; },): RunCargo {
  return async function run({ cwd, args, },) {
    if (!ok)
      return { ok: false, output: 'error: failed to select a version for `dep`', };
    if (args.includes('generate-lockfile',))
      await writeFile(join(cwd, 'Cargo.lock',), lock(version,),);
    /**
     Version a `cargo update --precise` asks for, empty for any other command.
     */
    const precise = args.includes('--precise',) ? (args[args.indexOf('--precise',) + 1] ?? '') : '';
    if (precise !== '')
      await writeFile(join(cwd, 'Cargo.lock',), lock(precise,),);
    return { ok: true, output: '', };
  };
}

/**
 Fake registry with fixed publish times and histories.

 @param history - every release of `dep`

 @returns registry

 @example
 ```ts
 fakeRegistry(history);
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
 Releases where 1.0.2 comes from a new account, 1.0.1 was published an hour before the test clock,
 and 1.0.0 is the trusted release locked before the update.
 */
const SET_ASIDE_HISTORY: readonly VersionRecord[] = [
  { num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/dep', },
  { num: '1.0.1', createdAt: '2026-10-29T11:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/dep', },
  { num: '1.0.2', createdAt: '2026-10-29T11:30:00Z', publisherId: '9', trustpubRepository: '', repository: 'https://github.com/o/dep', },
];

/**
 Releases by the same account.
 */
const TRUSTED_HISTORY: readonly VersionRecord[] = [
  { num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/dep', },
  { num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/dep', },
];

/**
 Releases where 1.0.1 comes from a new account.
 */
const DOWNGRADED_HISTORY: readonly VersionRecord[] = [
  { num: '0.9.0', createdAt: '2025-12-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/dep', },
  { num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', publisherId: '7', trustpubRepository: '', repository: 'https://github.com/o/dep', },
  { num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', publisherId: '9', trustpubRepository: '', repository: 'https://github.com/o/dep', },
];

/**
 Base-revision reader that returns the same lockfile text for every crate.

 @param source - lockfile text at the base, empty for a lockfile the base lacked

 @returns reader for the verification

 @example
 ```ts
 baseLocks(lock('1.0.0'));
 ```
 */
function baseLocks(source: string,): ReadBaseLock {
  return async function readBaseLock(): Promise<string> {
    return await Promise.resolve(source,);
  };
}

//endregion Fixtures

await describe({
  name: updateRustLockfiles.name,
  children: [
    it({
      name: 'keeps new lockfiles that pass and prunes stale exclude selectors',
      fn: async () => {
        /**
         Repository with a stale trust exclude.
         */
        await using repo = await makeRepo(catalog(['trustPolicyExclude = ["gone@1.0.0", "dep"]',],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Update outcome.
         */
        const report = await updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: true, },), registry: fakeRegistry(TRUSTED_HISTORY,), },);
        expect(report.written,).toBe(true,);
        expect(report.added,).toEqual([{ name: 'dep', version: '1.0.1', },],);
        expect(report.pruned,).toEqual(['trustPolicyExclude: gone@1.0.0',],);
        expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toContain('version = "1.0.1"',);
        expect(await readFile(join(root, 'rust-catalog.toml',), 'utf8',),).toContain('trustPolicyExclude = [ "dep", ]',);
      },
    },),
    it({
      name: 'restores every lockfile when a refused version has no older release to fall back to',
      fn: async () => {
        /**
         Repository.
         */
        await using repo = await makeRepo(catalog([],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Registry whose only compatible older release is yanked.
         */
        const registry: Registry = {
          ...fakeRegistry(DOWNGRADED_HISTORY,),
          yanked: async function yanked(): Promise<ReadonlySet<string>> {
            return new Set(['1.0.0',],);
          },
        };
        /**
         Update outcome.
         */
        const report = await updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: true, },), registry, },);
        expect(report.written,).toBe(false,);
        expect(report.problems.length,).toBe(1,);
        expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.0',),);
      },
    },),
    it({
      name: 'sets aside a refused version and locks the newest older release that passes',
      fn: async () => {
        /**
         Repository without a release-age cutoff, so 1.0.1 qualifies.
         */
        await using repo = await makeRepo(catalog([],).replace('minimumReleaseAge = 1440', 'minimumReleaseAge = 0',),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Update outcome.
         */
        const report = await updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.2', ok: true, },), registry: fakeRegistry(SET_ASIDE_HISTORY,), },);
        expect(report.problems,).toEqual([],);
        expect(report.written,).toBe(true,);
        expect(report.added,).toEqual([{ name: 'dep', version: '1.0.1', },],);
        expect(report.warnings,).toEqual(['Set aside dep@1.0.2 (trust policy) and locked 1.0.1 in package/cli/a.',],);
        expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.1',),);
      },
    },),
    it({
      name: 'skips a set-aside candidate published after the release-age cutoff',
      fn: async () => {
        /**
         Repository with the one-day cutoff, which 1.0.1 (an hour old) misses.
         */
        await using repo = await makeRepo(catalog([],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Update outcome.
         */
        const report = await updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.2', ok: true, },), registry: fakeRegistry(SET_ASIDE_HISTORY,), },);
        expect(report.warnings,).toEqual(['Set aside dep@1.0.2 (trust policy) and locked 1.0.0 in package/cli/a.',],);
        expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.0',),);
      },
    },),
    it({
      name: 'restores every lockfile on a dry run',
      fn: async () => {
        /**
         Repository.
         */
        await using repo = await makeRepo(catalog([],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Update outcome.
         */
        const report = await updateRustLockfiles({ root, dryRun: true, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: true, },), registry: fakeRegistry(TRUSTED_HISTORY,), },);
        expect(report.written,).toBe(false,);
        expect(report.added,).toEqual([{ name: 'dep', version: '1.0.1', },],);
        expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.0',),);
      },
    },),
    it({
      name: 'restores every lockfile and rethrows when cargo cannot resolve',
      fn: async () => {
        /**
         Repository.
         */
        await using repo = await makeRepo(catalog([],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        await expect(updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: false, },), registry: fakeRegistry(TRUSTED_HISTORY,), },),).rejects.toThrow('failed to select a version',);
        expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.0',),);
      },
    },),
  ],
},);

await describe({
  name: verifyRustLockfiles.name,
  children: [
    it({
      name: 'checks a version that no lockfile held at the base',
      fn: async () => {
        /**
         Repository whose working lock moved to the downgraded release.
         */
        await using repo = await makeRepo(catalog([],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        await writeFile(join(root, 'package/cli/a/Cargo.lock',), lock('1.0.1',),);
        expect((await verifyRustLockfiles({ root, now: NOW, registry: fakeRegistry(DOWNGRADED_HISTORY,), readBaseLock: baseLocks(lock('1.0.0',),), },)).length,).toBe(1,);
        expect(await verifyRustLockfiles({ root, now: NOW, registry: fakeRegistry(TRUSTED_HISTORY,), readBaseLock: baseLocks(lock('1.0.0',),), },),).toEqual([],);
      },
    },),
    it({
      name: 'skips a version the base already locked, even when its history would fail',
      fn: async () => {
        /**
         Repository whose base and working lock both hold the downgraded release.
         */
        await using repo = await makeRepo(catalog([],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        await writeFile(join(root, 'package/cli/a/Cargo.lock',), lock('1.0.1',),);
        expect(await verifyRustLockfiles({ root, now: NOW, registry: fakeRegistry(DOWNGRADED_HISTORY,), readBaseLock: baseLocks(lock('1.0.1',),), },),).toEqual([],);
      },
    },),
    it({
      name: 'treats a lockfile absent at the base as new in full',
      fn: async () => {
        /**
         Repository whose crate has no lockfile at the base.
         */
        await using repo = await makeRepo(catalog([],),);
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        await writeFile(join(root, 'package/cli/a/Cargo.lock',), lock('1.0.1',),);
        expect((await verifyRustLockfiles({ root, now: NOW, registry: fakeRegistry(DOWNGRADED_HISTORY,), readBaseLock: baseLocks('',), },)).length,).toBe(1,);
      },
    },),
  ],
},);

await describe({
  name: pruneExcludes.name,
  children: [
    it({
      name: 'leaves the text unchanged when pruning is off or nothing is stale',
      fn: async () => {
        /**
         Catalog text.
         */
        const source = 'trustPolicyExclude = ["gone"]\n';
        /**
         Settings with pruning off.
         */
        const settings = {
          catalogPrune: false, minimumReleaseAge: 0, minimumReleaseAgeExclude: [], minimumReleaseAgeExcludePrune: false, minimumReleaseAgeIgnoreMissingTime: false,
          minimumReleaseAgeStrict: true, trustPolicy: 'off', trustPolicyExclude: ['gone',], trustPolicyExcludePrune: false,
        } as const;
        expect(pruneExcludes({ source, settings, locked: [], },),).toEqual({ source, removed: [], },);
        expect(pruneExcludes({ source, settings: { ...settings, trustPolicyExcludePrune: true, trustPolicyExclude: ['*',], }, locked: [], },),).toEqual({ source, removed: [], },);
      },
    },),
  ],
},);

await describe({
  name: parseMode.name,
  children: [
    it({
      name: 'selects update by default and one mode per flag, rejecting unknown or combined flags',
      fn: async () => {
        expect(parseMode([],),).toEqual({ mode: 'update', base: 'HEAD', },);
        expect(parseMode(['--dry-run',],),).toEqual({ mode: 'dry-run', base: 'HEAD', },);
        expect(parseMode(['--verify',],),).toEqual({ mode: 'verify', base: 'HEAD', },);
        expect(parseMode(['--help',],),).toEqual({ mode: 'help', base: 'HEAD', },);
        expect(function unknown(): void {
          parseMode(['--force',],);
        },).toThrow('unknown argument',);
        expect(function combined(): void {
          parseMode(['--dry-run', '--verify',],);
        },).toThrow('at most one flag',);
      },
    },),
    it({
      name: 'takes one --base revision after --verify only',
      fn: async () => {
        expect(parseMode(['--verify', '--base', 'abc123',],),).toEqual({ mode: 'verify', base: 'abc123', },);
        expect(function missingRevision(): void {
          parseMode(['--verify', '--base',],);
        },).toThrow('--verify accepts only --base <rev>',);
        expect(function flagAsRevision(): void {
          parseMode(['--verify', '--base', '--dry-run',],);
        },).toThrow('--verify accepts only --base <rev>',);
        expect(function extraArgument(): void {
          parseMode(['--verify', '--base', 'a', 'b',],);
        },).toThrow('--verify accepts only --base <rev>',);
        expect(function otherFlag(): void {
          parseMode(['--verify', '--dry-run',],);
        },).toThrow('--verify accepts only --base <rev>',);
        expect(function baseWithoutVerify(): void {
          parseMode(['--dry-run', '--base', 'a',],);
        },).toThrow('at most one flag',);
      },
    },),
  ],
},);
