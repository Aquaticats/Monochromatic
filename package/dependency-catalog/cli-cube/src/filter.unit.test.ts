/**
 Tests for the pure filter logic.
 
 Exercised against a small fixture probe set that covers leaves,
 non-leaves, TS-majority and non-TS, recent and stale, known and
 unknown sources. Composition is asserted to make sure conjunctive
 filters behave like AND, not OR.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PackageProbe, } from './probe.ts';
import {
  computeVisibleIndices,
  DERIVED_BOOL_UNKNOWN,
  derivedBool,
  DIM_UNKNOWN,
  type DimMapping,
  extractDim,
  type RangeState,
  searchMatches,
  type ToggleState,
} from './script/filter.ts';

/**
 Default dim mapping mirroring the plan's recommended channel binding.
 */
const DEFAULT_MAPPING: DimMapping = {
  x: 'logSourceBytes',
  y: 'logDaysStale',
  z: 'logInstallSize',
  color: 'tsRatio',
  shape: 'isLeafNumeric',
  size: 'logDownloads',
};

/**
 Default "don't care" toggle state; every filter inactive.
 */
const ANY_TOGGLES: ToggleState = {
  isLeaf: 'any',
  tsMajority: 'any',
  large: 'any',
  recent: 'any',
  permissive: 'any',
  copyleft: 'any',
  hasKnownRepo: 'any',
};

/**
 Wide-open ranges, large enough to admit every fixture.
 */
const WIDE_RANGES: RangeState = {
  x: [-10, 10,],
  y: [-10, 10,],
  z: [-10, 10,],
  color: [-10, 10,],
  shape: [-10, 10,],
  size: [-10, 10,],
};

/**
 Healthy TS-majority leaf, GitHub-hosted, recent commits.
 */
const TS_LEAF: PackageProbe = {
  catalogKey: 'preact',
  npmName: 'preact',
  resolvedVersion: '10.26.0',
  isLeaf: true,
  weeklyDownloads: 5_000_000,
  installSizeBytes: 250_000,
  packageAgeDays: 1_500,
  licenseClass: 'permissive',
  runtimeDepCount: 0,
  transitiveDepCount: 0,
  tsRatioOrNull: 0.99,
  sourceBytesOrNull: 200_000,
  daysSinceLastCommitOrNull: 14,
  repositoryUrlOrNull: 'https://github.com/preactjs/preact',
  isMonorepoHoused: false,};

/**
 Non-leaf, JavaScript-majority, stale, copyleft, GitHub-hosted.
 */
const JS_NON_LEAF_STALE: PackageProbe = {
  catalogKey: 'ms',
  npmName: 'ms',
  resolvedVersion: '2.1.3',
  isLeaf: false,
  weeklyDownloads: 100_000,
  installSizeBytes: 5_000,
  packageAgeDays: 4_000,
  licenseClass: 'copyleft',
  runtimeDepCount: 2,
  transitiveDepCount: 5,
  tsRatioOrNull: 0.1,
  sourceBytesOrNull: 5_000,
  daysSinceLastCommitOrNull: 1_500,
  repositoryUrlOrNull: 'https://github.com/vercel/ms',
  isMonorepoHoused: false,};

/**
 Monorepo-housed entry: TS ratio + stale days unknown.
 */
const MONOREPO_UNKNOWN: PackageProbe = {
  catalogKey: '@lezer/common',
  npmName: '@lezer/common',
  resolvedVersion: '1.0.0',
  isLeaf: true,
  weeklyDownloads: 200_000,
  installSizeBytes: 50_000,
  packageAgeDays: 1_000,
  licenseClass: 'permissive',
  runtimeDepCount: 0,
  transitiveDepCount: 0,
  repositoryUrlOrNull: 'https://github.com/lezer-parser/common',
  isMonorepoHoused: true,
  unknownReason: 'monorepo',
};

/**
 Permissive leaf with a recent commit, not TS-majority; sits in
 the audit-target band.
 */
const AUDIT_TARGET: PackageProbe = {
  catalogKey: 'etag',
  npmName: 'etag',
  resolvedVersion: '1.8.1',
  isLeaf: true,
  weeklyDownloads: 30_000_000,
  installSizeBytes: 4_000,
  packageAgeDays: 5_000,
  licenseClass: 'permissive',
  runtimeDepCount: 0,
  transitiveDepCount: 0,
  tsRatioOrNull: 0,
  sourceBytesOrNull: 3_000,
  daysSinceLastCommitOrNull: 1_200,
  repositoryUrlOrNull: 'https://github.com/jshttp/etag',
  isMonorepoHoused: false,};

const PROBES: readonly PackageProbe[] = [
  TS_LEAF,
  JS_NON_LEAF_STALE,
  MONOREPO_UNKNOWN,
  AUDIT_TARGET,
];

await describe({
  name: 'filter',
  children: [
    //region extractDim
    it({
      name: 'extractDim log-scales source bytes',
      fn: async () => {
        const v = extractDim({
          probe: TS_LEAF,
          dim: 'logSourceBytes',
        },);
        expect(v,).toBeCloseTo(Math.log10(200_000,), 4,);
      },
    },),

    it({
      name: 'extractDim returns null when sourceBytesOrNull is null',
      fn: async () => {
        const v = extractDim({
          probe: MONOREPO_UNKNOWN,
          dim: 'logSourceBytes',
        },);
        expect(v,).toBe(DIM_UNKNOWN,);
      },
    },),

    it({
      name: 'extractDim binary leaf maps to 0/1',
      fn: async () => {
        expect(extractDim({ probe: TS_LEAF, dim: 'isLeafNumeric', },),).toBe(1,);
        expect(extractDim({ probe: JS_NON_LEAF_STALE, dim: 'isLeafNumeric', },),).toBe(
          0,
        );
      },
    },),

    it({
      name: 'extractDim license codes are 0/1/2/3',
      fn: async () => {
        expect(extractDim({ probe: TS_LEAF, dim: 'licenseClassNumeric', },),).toBe(0,);
        expect(extractDim({ probe: JS_NON_LEAF_STALE, dim: 'licenseClassNumeric', },),)
          .toBe(1,);
      },
    },),
    //endregion extractDim

    //region derivedBool
    it({
      name: 'derivedBool tsMajority returns DERIVED_BOOL_UNKNOWN when ratio is unknown',
      fn: async () => {
        expect(derivedBool({ probe: MONOREPO_UNKNOWN, key: 'tsMajority', },),).toBe(
          DERIVED_BOOL_UNKNOWN,
        );
      },
    },),

    it({
      name: 'derivedBool tsMajority is true at 99% TS',
      fn: async () => {
        expect(derivedBool({ probe: TS_LEAF, key: 'tsMajority', },),).toBe(true,);
      },
    },),

    it({
      name: 'derivedBool tsMajority is false at 10% TS',
      fn: async () => {
        expect(derivedBool({ probe: JS_NON_LEAF_STALE, key: 'tsMajority', },),).toBe(
          false,
        );
      },
    },),

    it({
      name: 'derivedBool hasKnownRepo distinguishes null vs non-null unknownReason',
      fn: async () => {
        expect(derivedBool({ probe: TS_LEAF, key: 'hasKnownRepo', },),).toBe(true,);
        expect(derivedBool({ probe: MONOREPO_UNKNOWN, key: 'hasKnownRepo', },),).toBe(
          false,
        );
      },
    },),
    //endregion derivedBool

    //region searchMatches
    it({
      name: 'searchMatches empty string matches everything',
      fn: async () => {
        expect(searchMatches({ probe: TS_LEAF, search: '', },),).toBe(true,);
      },
    },),

    it({
      name: 'searchMatches substring is case-insensitive',
      fn: async () => {
        expect(searchMatches({ probe: TS_LEAF, search: 'PREACT', },),).toBe(true,);
        expect(searchMatches({ probe: TS_LEAF, search: 'react', },),).toBe(true,);
        expect(searchMatches({ probe: TS_LEAF, search: 'vue', },),).toBe(false,);
      },
    },),

    it({
      name: 'searchMatches /regex/ form is case-insensitive',
      fn: async () => {
        expect(searchMatches({ probe: MONOREPO_UNKNOWN, search: '/^@lezer/', },),).toBe(
          true,
        );
        expect(searchMatches({ probe: TS_LEAF, search: '/^@lezer/', },),).toBe(false,);
      },
    },),

    it({
      name: 'searchMatches malformed regex falls back to no-match',
      fn: async () => {
        expect(searchMatches({ probe: TS_LEAF, search: '/[unclosed/', },),).toBe(false,);
      },
    },),
    //endregion searchMatches

    //region computeVisibleIndices
    it({
      name:
        'all "any" toggles + wide ranges + empty search admits every probe including partial-unknowns',
      fn: async () => {
        const visible = computeVisibleIndices({
          probes: PROBES,
          toggles: ANY_TOGGLES,
          ranges: WIDE_RANGES,
          search: '',
          dimMapping: DEFAULT_MAPPING,
        },);
        // Partial-unknowns pass range filters: null channel values are
        // treated as "not on the active scale, so the range doesn't
        // exclude them". The hasKnownRepo='yes' toggle is the explicit
        // knob for hiding partial-unknowns.
        expect(visible.has(0,),).toBe(true,);
        expect(visible.has(1,),).toBe(true,);
        expect(visible.has(2,),).toBe(true,);
        expect(visible.has(3,),).toBe(true,);
      },
    },),

    it({
      name: 'hasKnownRepo=yes hides partial-unknowns explicitly',
      fn: async () => {
        const visible = computeVisibleIndices({
          probes: PROBES,
          toggles: { ...ANY_TOGGLES, hasKnownRepo: 'yes', },
          ranges: WIDE_RANGES,
          search: '',
          dimMapping: DEFAULT_MAPPING,
        },);
        // MONOREPO_UNKNOWN has unknownReason !== null → hidden by this toggle.
        expect(visible.has(2,),).toBe(false,);
        expect(visible.has(0,),).toBe(true,);
        expect(visible.has(1,),).toBe(true,);
        expect(visible.has(3,),).toBe(true,);
      },
    },),

    it({
      name: 'toggle isLeaf=yes filters out non-leaves',
      fn: async () => {
        const visible = computeVisibleIndices({
          probes: PROBES,
          toggles: { ...ANY_TOGGLES, isLeaf: 'yes', },
          ranges: WIDE_RANGES,
          search: '',
          dimMapping: DEFAULT_MAPPING,
        },);
        expect(visible.has(1,),).toBe(false,);
      },
    },),

    it({
      name: 'toggle tsMajority=no rejects both unknown and TS-majority',
      fn: async () => {
        const visible = computeVisibleIndices({
          probes: PROBES,
          toggles: { ...ANY_TOGGLES, tsMajority: 'no', },
          ranges: WIDE_RANGES,
          search: '',
          dimMapping: DEFAULT_MAPPING,
        },);
        expect(visible.has(0,),).toBe(false,);
        expect(visible.has(2,),).toBe(false,);
        expect(visible.has(1,),).toBe(true,);
        expect(visible.has(3,),).toBe(true,);
      },
    },),

    it({
      name: 'audit-target pattern composes (non-TS + leaf + stale + permissive)',
      fn: async () => {
        const visible = computeVisibleIndices({
          probes: PROBES,
          toggles: {
            ...ANY_TOGGLES,
            tsMajority: 'no',
            isLeaf: 'yes',
            recent: 'no',
            permissive: 'yes',
          },
          ranges: WIDE_RANGES,
          search: '',
          dimMapping: DEFAULT_MAPPING,
        },);
        expect(visible.has(3,),).toBe(true,);
        expect(visible.has(0,),).toBe(false,);
        expect(visible.has(1,),).toBe(false,);
        expect(visible.has(2,),).toBe(false,);
      },
    },),

    it({
      name: 'name-search narrows visibility to substring matches',
      fn: async () => {
        const visible = computeVisibleIndices({
          probes: PROBES,
          toggles: ANY_TOGGLES,
          ranges: WIDE_RANGES,
          search: 'etag',
          dimMapping: DEFAULT_MAPPING,
        },);
        expect(visible.has(3,),).toBe(true,);
        expect(visible.has(0,),).toBe(false,);
      },
    },),

    it({
      name: 'narrow color range excludes out-of-range probes',
      fn: async () => {
        const visible = computeVisibleIndices({
          probes: PROBES,
          toggles: ANY_TOGGLES,
          ranges: {
            ...WIDE_RANGES,
            color: [0, 0.5,],
          },
          search: '',
          dimMapping: DEFAULT_MAPPING,
        },);
        expect(visible.has(0,),).toBe(false,);
        expect(visible.has(1,),).toBe(true,);
        expect(visible.has(3,),).toBe(true,);
      },
    },),
    //endregion computeVisibleIndices
  ],
},);
