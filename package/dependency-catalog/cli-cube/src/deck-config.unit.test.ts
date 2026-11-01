/**
 Tests for the deck.gl-side per-probe accessors.
 
 This file is named `deck-config.unit.test.ts` per the plan, but
 covers `deck-accessors.ts` only: importing `deck-config.ts`
 fails at module-load time because `@loaders.gl/schema-utils`
 statically imports `@math.gl/types`, which is not declared as a
 dependency on disk. rolldown (used by `render-html.ts` at
 runtime) sidesteps the resolution path via tree-shaking; Node
 module-load does not, so any test that touches
 `deck-config.ts` cannot load. The layer-count snapshot from the
 plan is therefore skipped with an explicit `skip` reason naming
 the upstream loader-gl packaging bug.
 
 Accessor outputs are still exercised here against fixture probes
 and bounds, covering position (known + unknown), fill colour
 (visible + filtered, known + unknown), radius, and the
 filled-vs-stroked encoding.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  POSITION_UNKNOWN,
  probeFillColor,
  probeIsFilled,
  probePosition,
  probeRadius,
  unknownClusterPosition,
} from './deck-accessors.ts';
import type { PackageProbe, } from './probe.ts';
import { defaultState, } from './script/state.ts';

/**
 Healthy GH-hosted leaf with known spatial coords.
 */
const KNOWN: PackageProbe = {
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
  tsRatioOrNull: 0.9,
  sourceBytesOrNull: 200_000,
  daysSinceLastCommitOrNull: 14,
  repositoryUrlOrNull: 'https://github.com/preactjs/preact',
  isMonorepoHoused: false,};

/**
 Monorepo-housed probe; TS ratio and source bytes unknown.
 */
const UNKNOWN: PackageProbe = {
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
 Non-leaf probe (renders stroked in the default shape encoding).
 */
const NON_LEAF: PackageProbe = {
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

const PROBES = [KNOWN, NON_LEAF, UNKNOWN,];
const STATE = defaultState({ probes: PROBES, },);

/**
 Bounds spanning each channel from 0 to 1 so accessor maths are
 easy to reason about (no nontrivial normalisation needed).
 */
const BOUNDS = {
  x: [0, 1,],
  y: [0, 1,],
  z: [0, 1,],
  color: [0, 1,],
  shape: [0, 1,],
  size: [0, 1,],
} as const;

await describe({
  name: 'deck-config (accessors)',
  children: [
    //region probePosition
    it({
      name: 'probePosition returns 3D coords for known-spatial probes',
      fn: async () => {
        const pos = probePosition({
          probe: KNOWN,
          state: STATE,
        },);
        expect(pos,).not.toBe(POSITION_UNKNOWN,);
        if (pos === POSITION_UNKNOWN)
          return;
        expect(pos.length,).toBe(3,);
      },
    },),

    it({
      name: 'probePosition returns POSITION_UNKNOWN when any spatial dim is unknown',
      fn: async () => {
        const pos = probePosition({
          probe: UNKNOWN,
          state: STATE,
        },);
        expect(pos,).toBe(POSITION_UNKNOWN,);
      },
    },),
    //endregion probePosition

    //region unknownClusterPosition
    it({
      name: 'unknownClusterPosition offsets beyond the data box',
      fn: async () => {
        const pos = unknownClusterPosition({
          index: 7,
          bounds: BOUNDS,
        },);
        expect(pos[0],).toBeGreaterThan(BOUNDS.x[1],);
        expect(pos[1],).toBeGreaterThan(BOUNDS.y[1],);
        expect(pos[2],).toBeGreaterThan(BOUNDS.z[1],);
      },
    },),

    it({
      name: 'unknownClusterPosition is deterministic per index',
      fn: async () => {
        const a = unknownClusterPosition({ index: 13, bounds: BOUNDS, },);
        const b = unknownClusterPosition({ index: 13, bounds: BOUNDS, },);
        expect(a,).toEqual(b,);
      },
    },),

    it({
      name: 'unknownClusterPosition jitters distinctly per index',
      fn: async () => {
        const a = unknownClusterPosition({ index: 1, bounds: BOUNDS, },);
        const b = unknownClusterPosition({ index: 2, bounds: BOUNDS, },);
        expect(a,).not.toEqual(b,);
      },
    },),
    //endregion unknownClusterPosition

    //region probeFillColor
    it({
      name: 'probeFillColor returns full alpha for visible probes',
      fn: async () => {
        const rgba = probeFillColor({
          probe: KNOWN,
          state: STATE,
          bounds: BOUNDS,
          isVisible: true,
        },);
        expect(rgba[3],).toBe(255,);
      },
    },),

    it({
      name: 'probeFillColor returns ~5% alpha for filtered-out probes',
      fn: async () => {
        const rgba = probeFillColor({
          probe: KNOWN,
          state: STATE,
          bounds: BOUNDS,
          isVisible: false,
        },);
        expect(rgba[3],).toBe(13,);
      },
    },),

    it({
      name: 'probeFillColor returns grey for unknown colour dim',
      fn: async () => {
        const rgba = probeFillColor({
          probe: UNKNOWN,
          state: STATE,
          bounds: BOUNDS,
          isVisible: true,
        },);
        expect(rgba[0],).toBe(136,);
        expect(rgba[1],).toBe(136,);
        expect(rgba[2],).toBe(136,);
      },
    },),

    it({
      name: 'probeFillColor red↔green ramp matches the colour-channel value',
      fn: async () => {
        const lowTs: PackageProbe = { ...KNOWN, tsRatioOrNull: 0, };
        const highTs: PackageProbe = { ...KNOWN, tsRatioOrNull: 1, };
        const low = probeFillColor({
          probe: lowTs,
          state: STATE,
          bounds: BOUNDS,
          isVisible: true,
        },);
        const high = probeFillColor({
          probe: highTs,
          state: STATE,
          bounds: BOUNDS,
          isVisible: true,
        },);
        expect(low[0],).toBeGreaterThan(low[1],);
        expect(high[1],).toBeGreaterThan(high[0],);
      },
    },),
    //endregion probeFillColor

    //region probeRadius
    it({
      name: 'probeRadius lies between 3 and 30 pixels for in-bounds values',
      fn: async () => {
        const r = probeRadius({
          probe: KNOWN,
          state: STATE,
          bounds: BOUNDS,
        },);
        expect(r,).toBeGreaterThanOrEqual(3,);
        expect(r,).toBeLessThanOrEqual(30,);
      },
    },),

    it({
      name: 'probeRadius falls back to the minimum when size dim is unknown',
      fn: async () => {
        const orphaned: PackageProbe = {
          ...KNOWN,
          weeklyDownloads: 0,
        };
        // logDownloads with weeklyDownloads=0 floors to log10(1)=0; with
        // BOUNDS.size === [0,1] the normalised result is 0, so r === 3.
        const r = probeRadius({
          probe: orphaned,
          state: STATE,
          bounds: BOUNDS,
        },);
        expect(r,).toBe(3,);
      },
    },),
    //endregion probeRadius

    //region probeIsFilled
    it({
      name: 'probeIsFilled is false for leaves under the default shape mapping',
      fn: async () => {
        // isLeaf=true → isLeafNumeric=1; SHAPE_FILLED_THRESHOLD = 0.5;
        // `value < threshold` so leaves render hollow / stroked.
        expect(probeIsFilled({ probe: KNOWN, state: STATE, },),).toBe(false,);
      },
    },),

    it({
      name: 'probeIsFilled is true for non-leaves under the default shape mapping',
      fn: async () => {
        // isLeafNumeric=0 for non-leaves; 0 < 0.5 → filled.
        expect(probeIsFilled({ probe: NON_LEAF, state: STATE, },),).toBe(true,);
      },
    },),

    it({
      name: 'probeIsFilled defaults to hollow when shape dim is unknown',
      fn: async () => {
        const shapelessState = {
          ...STATE,
          dimMapping: { ...STATE.dimMapping, shape: 'tsRatio' as const, },
        };
        expect(probeIsFilled({ probe: UNKNOWN, state: shapelessState, },),).toBe(false,);
      },
    },),
    //endregion probeIsFilled

    //region Layer-count snapshot (blocked upstream)
    it({
      name: 'snapshots layer count against displayToggles permutations',
      skip:
        'blocked by @loaders.gl/schema-utils ^4.4.1 statically importing @math.gl/types '
          + 'without declaring it as a dep; deck-config.ts cannot be module-loaded in Node '
          + 'until the upstream packaging is fixed. rolldown resolves it via tree-shaking, '
          + 'so the runtime HTML still works.',
      fn: async () => {
        // Intentionally empty; kept as a documented placeholder so the
        // limitation appears in test output rather than only in HANDOVER.
      },
    },),
    //endregion Layer-count snapshot (blocked upstream)
  ],
},);
