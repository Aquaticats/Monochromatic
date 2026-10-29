/**
 Tests for URL-hash state serialisation, defaults, and range
 extent computation.
 
 Covers the round-trip path, fallback-on-corrupt-hash behaviour,
 and indirectly exercises the same `extractDim`-based extent
 computation used by `computeSceneBounds` in `deck-config.ts`,
 which cannot be imported here because of the upstream
 `@loaders.gl/schema-utils` → `@math.gl/types` runtime resolver
 bug (see `deck-config.unit.test.ts`).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PackageProbe, } from './probe.ts';
import { extractDim, } from './script/filter.ts';
import {
  type AppState,
  decodeState,
  defaultState,
  encodeState,
  readStateFromHash,
  STATE_INVALID,
  TOGGLE_KEYS,
  writeStateToHash,
} from './script/state.ts';

/**
 Three probes spanning the data range so extent computations have
 something non-degenerate to measure.
 */
const PROBES: readonly PackageProbe[] = [
  {
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
    tsRatioOrNull: 0.95,
    sourceBytesOrNull: 200_000,
    daysSinceLastCommitOrNull: 14,
    repositoryUrlOrNull: 'https://github.com/preactjs/preact',
    isMonorepoHoused: false,  },
  {
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
    isMonorepoHoused: false,  },
  {
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
    isMonorepoHoused: false,  },
];

await describe({
  name: 'state',
  children: [
    //region defaultState
    it({
      name: 'defaultState supplies the plan-defined dim mapping',
      fn: async () => {
        const s = defaultState({ probes: PROBES, },);
        expect(s.dimMapping,).toEqual({
          x: 'logSourceBytes',
          y: 'logDaysStale',
          z: 'logInstallSize',
          color: 'tsRatio',
          shape: 'isLeafNumeric',
          size: 'logDownloads',
        },);
      },
    },),

    it({
      name: 'defaultState sets every toggle to "any"',
      fn: async () => {
        const s = defaultState({ probes: PROBES, },);
        for (const key of TOGGLE_KEYS)
          expect(s.toggles[key],).toBe('any',);
      },
    },),

    it({
      name: 'defaultState computes ranges as the data extent per channel',
      fn: async () => {
        const s = defaultState({ probes: PROBES, },);
        const xValues = PROBES
          .map(function px(probe,) {
            return extractDim({ probe, dim: 'logSourceBytes', },);
          },)
          .filter(function nn(v,): v is number {
            return v !== null;
          },);
        const expectedXmin = Math.min(...xValues,);
        const expectedXmax = Math.max(...xValues,);
        expect(s.ranges.x[0],).toBeCloseTo(expectedXmin, 6,);
        expect(s.ranges.x[1],).toBeCloseTo(expectedXmax, 6,);
      },
    },),

    it({
      name:
        'defaultState turns on wireframe / axis labels / unknown cluster / all name labels; threshold guides off by default',
      fn: async () => {
        const s = defaultState({ probes: PROBES, },);
        expect(s.displayToggles.showWireframe,).toBe(true,);
        expect(s.displayToggles.showThresholdPlanes,).toBe(false,);
        expect(s.displayToggles.showAxisLabels,).toBe(true,);
        expect(s.displayToggles.showUnknownCluster,).toBe(true,);
        expect(s.displayToggles.nameLabels,).toBe('all',);
      },
    },),
    //endregion defaultState

    //region encoding round-trip
    it({
      name: 'encodeState/decodeState round-trip preserves every field',
      fn: async () => {
        const state = defaultState({ probes: PROBES, },);
        const encoded = encodeState({ state, },);
        const decoded = decodeState({ encoded, },);
        expect(decoded,).toEqual(state,);
      },
    },),

    it({
      name: 'decodeState returns STATE_INVALID for malformed input',
      fn: async () => {
        expect(decodeState({ encoded: '%%not-valid-uri%%', },),).toBe(STATE_INVALID,);
        expect(decodeState({ encoded: 'not%20json', },),).toBe(STATE_INVALID,);
      },
    },),

    it({
      name: 'decodeState rejects objects missing required keys',
      fn: async () => {
        const partial = encodeURIComponent(JSON.stringify({ viewState: {}, },),);
        expect(decodeState({ encoded: partial, },),).toBe(STATE_INVALID,);
      },
    },),
    //endregion encoding round-trip

    //region hash helpers
    it({
      name: 'writeStateToHash output is consumed by readStateFromHash unchanged',
      fn: async () => {
        const state = defaultState({ probes: PROBES, },);
        const hash = writeStateToHash({ state, },);
        const restored = readStateFromHash({
          hash,
          fallback: state,
        },);
        expect(restored,).toEqual(state,);
      },
    },),

    it({
      name: 'readStateFromHash falls back when hash is empty',
      fn: async () => {
        const state = defaultState({ probes: PROBES, },);
        const restored = readStateFromHash({
          hash: '',
          fallback: state,
        },);
        expect(restored,).toBe(state,);
      },
    },),

    it({
      name: 'readStateFromHash falls back when state= section is corrupt',
      fn: async () => {
        const fallback = defaultState({ probes: PROBES, },);
        const restored = readStateFromHash({
          hash: '#state=not-valid-base64',
          fallback,
        },);
        expect(restored,).toEqual(fallback,);
      },
    },),

    it({
      name: 'readStateFromHash mutates a property and round-trips the change',
      fn: async () => {
        const state = defaultState({ probes: PROBES, },);
        const mutated: AppState = {
          ...state,
          toggles: { ...state.toggles, tsMajority: 'no', },
          search: 'preact',
        };
        const hash = writeStateToHash({ state: mutated, },);
        const restored = readStateFromHash({
          hash,
          fallback: state,
        },);
        expect(restored.toggles.tsMajority,).toBe('no',);
        expect(restored.search,).toBe('preact',);
      },
    },),
    //endregion hash helpers
  ],
},);
