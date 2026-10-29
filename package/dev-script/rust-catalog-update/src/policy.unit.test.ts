/**
 Unit tests for the release-age and trust-policy decisions.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import type { RustCatalogSettings, } from '@monochromatic-dev/module-rust-catalog/ts';

import {
  releaseAgeProblems,
  trustLevel,
  trustProblems,
  type VersionRecord,
} from './policy.ts';

//region Fixtures

/**
 Settings with the repository's defaults.
 */
const SETTINGS: RustCatalogSettings = {
  catalogPrune: false,
  minimumReleaseAge: 1_440,
  minimumReleaseAgeExclude: [],
  minimumReleaseAgeExcludePrune: false,
  minimumReleaseAgeIgnoreMissingTime: false,
  minimumReleaseAgeStrict: true,
  trustPolicy: 'no-downgrade',
  trustPolicyExclude: [],
  trustPolicyExcludePrune: false,
};

/**
 Fixed clock.
 */
const NOW = Date.parse('2026-10-29T12:00:00Z',);

/**
 Builds a version record.

 @param overrides - fields to set

 @returns record

 @example
 ```ts
 record({ num: '1.0.0', });
 ```
 */
function record(
  {
    num,
    createdAt,
    publisherId = '7',
    trustpubRepository = '',
    repository = 'https://github.com/o/r',
  }: {
    readonly num: string;
    readonly createdAt: string;
    readonly publisherId?: string;
    readonly trustpubRepository?: string;
    readonly repository?: string;
  },
): VersionRecord {
  return { num, createdAt, publisherId, trustpubRepository, repository, };
}

//endregion Fixtures

await describe({
  name: releaseAgeProblems.name,
  children: [
    it({
      name: 'passes a mature version and fails a young one with the time it matures',
      fn: async () => {
        expect(releaseAgeProblems({ name: 'a', version: '1.0.0', pubtime: '2026-10-27T12:00:00Z', settings: SETTINGS, now: NOW, },),).toEqual([],);
        /**
         Findings for a version published two hours ago.
         */
        const young = releaseAgeProblems({ name: 'a', version: '1.0.1', pubtime: '2026-10-29T10:00:00Z', settings: SETTINGS, now: NOW, },);
        expect(young.length,).toBe(1,);
        expect(young[0],).toContain('2026-10-30T10:00:00.000Z',);
      },
    },),
    it({
      name: 'skips excluded versions',
      fn: async () => {
        expect(releaseAgeProblems({ name: 'a', version: '1.0.1', pubtime: '2026-10-29T10:00:00Z', settings: { ...SETTINGS, minimumReleaseAgeExclude: ['a',], }, now: NOW, },),).toEqual([],);
      },
    },),
    it({
      name: 'fails or passes a missing publish time by minimumReleaseAgeIgnoreMissingTime',
      fn: async () => {
        expect(releaseAgeProblems({ name: 'a', version: '1.0.0', pubtime: '', settings: SETTINGS, now: NOW, },).length,).toBe(1,);
        expect(releaseAgeProblems({ name: 'a', version: '1.0.0', pubtime: '', settings: { ...SETTINGS, minimumReleaseAgeIgnoreMissingTime: true, }, now: NOW, },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: trustLevel.name,
  children: [
    it({
      name: 'rates trusted publishing 3, a returning publisher 2, and a first-time publisher 1',
      fn: async () => {
        /**
         First release by account 7.
         */
        const first = record({ num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', },);
        expect(trustLevel({ record: record({ num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', trustpubRepository: 'o/r', publisherId: '', },), earlier: [first,], },),).toBe(3,);
        expect(trustLevel({ record: record({ num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', },), earlier: [first,], },),).toBe(2,);
        expect(trustLevel({ record: first, earlier: [], },),).toBe(1,);
      },
    },),
  ],
},);

await describe({
  name: trustProblems.name,
  children: [
    it({
      name: 'passes a returning publisher and fails a token publish after trusted publishing',
      fn: async () => {
        /**
         Two trusted releases, then a token publish.
         */
        const history = [
          record({ num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', trustpubRepository: 'o/r', publisherId: '', },),
          record({ num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', trustpubRepository: 'o/r', publisherId: '', },),
          record({ num: '1.0.2', createdAt: '2026-03-01T00:00:00Z', publisherId: '9', },),
        ];
        expect(trustProblems({ name: 'a', version: '1.0.1', history, settings: SETTINGS, now: NOW, },),).toEqual([],);
        /**
         Findings for the downgraded release.
         */
        const downgraded = trustProblems({ name: 'a', version: '1.0.2', history, settings: SETTINGS, now: NOW, },);
        expect(downgraded.length,).toBe(1,);
        expect(downgraded[0],).toContain('trust level 1, below an earlier release\'s 3',);
      },
    },),
    it({
      name: 'rates each earlier stable release against the full history, prereleases included',
      fn: async () => {
        // Account 7's prerelease makes its later stable 1.0.0 a returning-publisher
        // release (level 2), so 1.0.1 by first-time account 9 (level 1) is a downgrade.
        // Rating 1.0.0 against a prerelease-filtered history would call it level 1
        // and miss the downgrade.
        /**
         Account 7 publishes a prerelease and 1.0.0; account 9 then publishes 1.0.1.
         */
        const history = [
          record({ num: '1.0.0-rc.1', createdAt: '2026-01-01T00:00:00Z', },),
          record({ num: '1.0.0', createdAt: '2026-02-01T00:00:00Z', },),
          record({ num: '1.0.1', createdAt: '2026-03-01T00:00:00Z', publisherId: '9', },),
        ];
        /**
         Findings for the new account's release.
         */
        const problems = trustProblems({ name: 'a', version: '1.0.1', history, settings: SETTINGS, now: NOW, },);
        expect(problems.length,).toBe(1,);
        expect(problems[0],).toContain('trust level 1, below an earlier release\'s 2',);
      },
    },),
    it({
      name: 'fails a trusted-publishing repository change and a declared repository change',
      fn: async () => {
        /**
         Trusted releases from o/r, then one from evil/r with a new declared repository.
         */
        const history = [
          record({ num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', trustpubRepository: 'o/r', publisherId: '', },),
          record({ num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', trustpubRepository: 'evil/r', publisherId: '', repository: 'https://github.com/evil/r', },),
        ];
        /**
         Findings.
         */
        const problems = trustProblems({ name: 'a', version: '1.0.1', history, settings: SETTINGS, now: NOW, },);
        expect(problems.length,).toBe(2,);
        expect(problems[0],).toContain('trusted-published from evil/r, not o/r',);
        expect(problems[1],).toContain('declares repository "https://github.com/evil/r"',);
      },
    },),
    it({
      name: 'skips the check when off, excluded, or older than trustPolicyIgnoreAfter',
      fn: async () => {
        /**
         Downgrading history.
         */
        const history = [
          record({ num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', trustpubRepository: 'o/r', publisherId: '', },),
          record({ num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', publisherId: '9', },),
        ];
        expect(trustProblems({ name: 'a', version: '1.0.1', history, settings: { ...SETTINGS, trustPolicy: 'off', }, now: NOW, },),).toEqual([],);
        expect(trustProblems({ name: 'a', version: '1.0.1', history, settings: { ...SETTINGS, trustPolicyExclude: ['a@1.0.1',], }, now: NOW, },),).toEqual([],);
        expect(trustProblems({ name: 'a', version: '1.0.1', history, settings: { ...SETTINGS, trustPolicyIgnoreAfter: 60, }, now: NOW, },),).toEqual([],);
      },
    },),
    it({
      name: 'reports a version crates.io does not list',
      fn: async () => {
        expect(trustProblems({ name: 'a', version: '9.9.9', history: [], settings: SETTINGS, now: NOW, },).length,).toBe(1,);
      },
    },),
  ],
},);
