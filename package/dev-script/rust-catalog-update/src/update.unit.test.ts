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
} from '@monochromatic-dev/module-test/ts';

import type { RunCargo, } from './cargo.ts';
import { parseMode, } from './cli.ts';
import type { VersionRecord, } from './policy.ts';
import { pruneExcludes, } from './prune.ts';
import type { Registry, } from './registry.ts';
import { updateRustLockfiles, } from './update.ts';
import { verifyRustLockfiles, } from './verify.ts';

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

 @returns repository root

 @example
 ```ts
 await makeRepo(catalog([]));
 ```
 */
async function makeRepo(catalogText: string,): Promise<string> {
  /**
   Repository root.
   */
  const root = await mkdtemp(join(tmpdir(), 'rust-catalog-update-',),);
  await mkdir(join(root, 'package/cli/a',), { recursive: true, },);
  await writeFile(join(root, 'rust-catalog.toml',), catalogText,);
  await writeFile(join(root, 'package/cli/a/Cargo.toml',), '[package]\nname = "a"\nversion = "0.1.0"\n\n[workspace]\n',);
  await writeFile(join(root, 'package/cli/a/Cargo.lock',), lock('1.0.0',),);
  return root;
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
  };
}

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
        const root = await makeRepo(catalog(['trustPolicyExclude = ["gone@1.0.0", "dep"]',],),);
        try {
          /**
           Update outcome.
           */
          const report = await updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: true, },), registry: fakeRegistry(TRUSTED_HISTORY,), },);
          expect(report.written,).toBe(true,);
          expect(report.added,).toEqual([{ name: 'dep', version: '1.0.1', },],);
          expect(report.pruned,).toEqual(['trustPolicyExclude: gone@1.0.0',],);
          expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toContain('version = "1.0.1"',);
          expect(await readFile(join(root, 'rust-catalog.toml',), 'utf8',),).toContain('trustPolicyExclude = [ "dep", ]',);
        } finally {
          await rm(root, { recursive: true, force: true, },);
        }
      },
    },),
    it({
      name: 'restores every lockfile when a newly locked version fails the trust policy',
      fn: async () => {
        /**
         Repository.
         */
        const root = await makeRepo(catalog([],),);
        try {
          /**
           Update outcome.
           */
          const report = await updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: true, },), registry: fakeRegistry(DOWNGRADED_HISTORY,), },);
          expect(report.written,).toBe(false,);
          expect(report.problems.length,).toBe(1,);
          expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.0',),);
        } finally {
          await rm(root, { recursive: true, force: true, },);
        }
      },
    },),
    it({
      name: 'restores every lockfile on a dry run',
      fn: async () => {
        /**
         Repository.
         */
        const root = await makeRepo(catalog([],),);
        try {
          /**
           Update outcome.
           */
          const report = await updateRustLockfiles({ root, dryRun: true, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: true, },), registry: fakeRegistry(TRUSTED_HISTORY,), },);
          expect(report.written,).toBe(false,);
          expect(report.added,).toEqual([{ name: 'dep', version: '1.0.1', },],);
          expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.0',),);
        } finally {
          await rm(root, { recursive: true, force: true, },);
        }
      },
    },),
    it({
      name: 'restores every lockfile and rethrows when cargo cannot resolve',
      fn: async () => {
        /**
         Repository.
         */
        const root = await makeRepo(catalog([],),);
        try {
          await expect(updateRustLockfiles({ root, dryRun: false, now: NOW, runCargo: resolvingTo({ version: '1.0.1', ok: false, },), registry: fakeRegistry(TRUSTED_HISTORY,), },),).rejects.toThrow('failed to select a version',);
          expect(await readFile(join(root, 'package/cli/a/Cargo.lock',), 'utf8',),).toBe(lock('1.0.0',),);
        } finally {
          await rm(root, { recursive: true, force: true, },);
        }
      },
    },),
  ],
},);

await describe({
  name: verifyRustLockfiles.name,
  children: [
    it({
      name: 'checks every locked version, not only new ones',
      fn: async () => {
        /**
         Repository whose committed lock already holds the downgraded release.
         */
        const root = await makeRepo(catalog([],),);
        try {
          await writeFile(join(root, 'package/cli/a/Cargo.lock',), lock('1.0.1',),);
          expect((await verifyRustLockfiles({ root, now: NOW, registry: fakeRegistry(DOWNGRADED_HISTORY,), },)).length,).toBe(1,);
          expect(await verifyRustLockfiles({ root, now: NOW, registry: fakeRegistry(TRUSTED_HISTORY,), },),).toEqual([],);
        } finally {
          await rm(root, { recursive: true, force: true, },);
        }
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
        expect(parseMode([],),).toBe('update',);
        expect(parseMode(['--dry-run',],),).toBe('dry-run',);
        expect(parseMode(['--verify',],),).toBe('verify',);
        expect(parseMode(['--help',],),).toBe('help',);
        expect(function unknown(): void {
          parseMode(['--force',],);
        },).toThrow('unknown argument',);
        expect(function combined(): void {
          parseMode(['--dry-run', '--verify',],);
        },).toThrow('at most one flag',);
      },
    },),
  ],
},);
