/**
 Regression coverage for catalog-sized bounds at report and dropdown boundaries.

 @module
 */

import assert from 'node:assert/strict';
import { describe, it, } from '@monochromatic-dev/test-harness-core/ts';
import { renderControls, } from '../dist/final/node/index.mjs';
import {
  CHANNEL_KEYS,
  computeRangeExtent,
  computeSceneBounds,
  defaultState,
  type PackageProbe,
  type Session,
  wireDimDropdowns,
} from '../dist/final/node/range-verification.mjs';

//region Catalog fixtures shared by boundary checks

/** Known probe whose readings remain valid for every selectable dimension. */
const PROBE: PackageProbe = {
  catalogKey: 'range-fixture',
  npmName: 'range-fixture',
  resolvedVersion: '1.0.0',
  isLeaf: true,
  weeklyDownloads: 1,
  installSizeBytes: 1,
  packageAgeDays: 1,
  licenseClass: 'permissive',
  runtimeDepCount: 0,
  transitiveDepCount: 0,
  tsRatioOrNull: 0.5,
  sourceBytesOrNull: 1,
  daysSinceLastCommitOrNull: 1,
  isMonorepoHoused: false,
};

/** Exceeds V8's call-argument ceiling: the spread implementation throws on Node 26.13.0. */
const LARGE_CATALOG_SIZE = 2 ** 18;

/** Upper endpoint differs from the first probe, so dropping later readings is observable. */
const HIGH_PROBE: PackageProbe = { ...PROBE, runtimeDepCount: LARGE_CATALOG_SIZE - 1, };

/** Reuses immutable fixtures to test catalog cardinality without duplicating probe payloads. */
const LARGE_PROBES: readonly PackageProbe[] = Array.from(
  { length: LARGE_CATALOG_SIZE, },
  (_, index,) => index === 0 ? PROBE : HIGH_PROBE,
);

/** Expected extent derived from the catalog endpoints. */
const LARGE_EXTENT: readonly [number, number,] = [PROBE.runtimeDepCount, HIGH_PROBE.runtimeDepCount,];

//endregion Catalog fixtures shared by boundary checks

//region DOM boundary fixture

/** Control surface used by the dropdown handler, with real event dispatch. */
class ControlElement extends EventTarget {
  value = '';
  min = '';
  max = '';
}

/**
 Exercises the real dropdown listener and its subsequent scene-bounds update.

 @returns Completion after restoring every replaced DOM global
 */
function checkDropdown(): void {
  /** Controls generated with the same ids as the report. */
  const controls = new Map<string, ControlElement>(CHANNEL_KEYS.flatMap(channel => [
    [`#dim-${channel}`, new ControlElement(),],
    [`#range-${channel}-min`, new ControlElement(),],
    [`#range-${channel}-max`, new ControlElement(),],
  ] as const,),);
  /** Original descriptors let the test restore both absent and present globals. */
  const original = new Map(['document', 'HTMLInputElement', 'HTMLSelectElement',].map(
    key => [key, Object.getOwnPropertyDescriptor(globalThis, key,),] as const,
  ),);
  /** Restore every DOM global when the boundary check leaves scope. */
  using cleanup = {
    [Symbol.dispose](): void {
      for (const [key, descriptor,] of original) {
        if (descriptor === undefined)
          Reflect.deleteProperty(globalThis, key,);
        else
          Object.defineProperty(globalThis, key, descriptor,);
      }
    },
  };
  Object.defineProperty(globalThis, 'document', {
    configurable: true,
    value: { querySelector: (selector: string,) => controls.get(selector,) ?? null, },
  },);
  Object.defineProperty(globalThis, 'HTMLInputElement', { configurable: true, value: ControlElement, },);
  Object.defineProperty(globalThis, 'HTMLSelectElement', { configurable: true, value: ControlElement, },);
  /** Initial state must itself support the full catalog before the dropdown can be used. */
  const state = defaultState({ probes: LARGE_PROBES, },);
  /** Mutable controller session, matching the event module's public contract. */
  const session: Session = {
    state,
    bounds: computeSceneBounds({ probes: LARGE_PROBES, dimMapping: state.dimMapping, },),
    visibleIndices: new Set<number>(),
  };
  /** Counts render notifications after the listener has finished all bounds updates. */
  const commits = { count: 0, };
  wireDimDropdowns({
    session,
    probes: LARGE_PROBES,
    commit: () => {
      commits.count += 1;
    },
  },);
  /** Real event target wired by the production function. */
  const select = controls.get('#dim-x',);
  assert.ok(select,);
  select.value = 'runtimeDepCount';
  select.dispatchEvent(new Event('change',),);
  assert.equal(commits.count, 1,);
  assert.deepEqual(session.state.ranges.x, LARGE_EXTENT,);
  assert.deepEqual(session.bounds.x, LARGE_EXTENT,);
  assert.equal(controls.get('#range-x-min',)?.value, String(LARGE_EXTENT[0],),);
  assert.equal(controls.get('#range-x-max',)?.value, String(LARGE_EXTENT[1],),);
}

//endregion DOM boundary fixture

await describe({
  name: 'catalog range extents',
  children: [
    it({
      name: 'keeps the empty and all-unknown defaults, including the scene fallback',
      fn: async () => {
        /** Removing the optional reading represents an unknown dimension. */
        const { tsRatioOrNull, ...unknown } = PROBE;
        assert.notEqual(tsRatioOrNull, undefined,);
        for (const probes of [[], [unknown,],]) {
          assert.deepEqual(computeRangeExtent({ probes, dim: 'tsRatio', },), [0, 0,],);
          /** Scene bounds use a non-degenerate fallback, unlike slider ranges. */
          const state = defaultState({ probes, },);
          assert.deepEqual(computeSceneBounds({ probes, dimMapping: state.dimMapping, },).color, [0, 1,],);
        }
      },
    },),
    it({
      name: 'skips unknown readings and finds both extrema regardless of ordering',
      fn: async () => {
        /** Unknown before and between known values must not become a numeric bound. */
        const { tsRatioOrNull, ...unknown } = PROBE;
        assert.notEqual(tsRatioOrNull, undefined,);
        /** Unsorted readings distinguish the minimum and maximum updates. */
        const probes = [unknown, { ...PROBE, tsRatioOrNull: 0.75, }, unknown, { ...PROBE, tsRatioOrNull: 0.25, }, PROBE,];
        assert.deepEqual(computeRangeExtent({ probes, dim: 'tsRatio', },), [0.25, 0.75,],);
        assert.deepEqual(computeRangeExtent({ probes: [PROBE,], dim: 'tsRatio', },), [tsRatioOrNull, tsRatioOrNull,],);
      },
    },),
    it({
      name: 'preserves NaN, infinity and signed-zero min/max semantics',
      fn: async () => {
        for (const values of [[0, -0,], [-0, 0,], [1, Number.NaN,], [Infinity,], [-Infinity,],]) {
          /** Each case is bounded, so spreading it is a valid reference oracle. */
          const probes = values.map(runtimeDepCount => ({ ...PROBE, runtimeDepCount, }),);
          assert.deepEqual(computeRangeExtent({ probes, dim: 'runtimeDepCount', },), [Math.min(...values,), Math.max(...values,),],);
        }
      },
    },),
    it({
      name: 'reduces a catalog larger than the engine argument limit',
      fn: async () => {
        assert.deepEqual(computeRangeExtent({ probes: LARGE_PROBES, dim: 'runtimeDepCount', },), LARGE_EXTENT,);
      },
    },),
    it({
      name: 'creates initial state and report controls for the full catalog',
      fn: async () => {
        /** Report generation and the initial controller state share the same bounds reduction. */
        const state = defaultState({ probes: LARGE_PROBES, },);
        assert.equal(state.ranges.x[0], 0,);
        assert.ok(renderControls({ probes: LARGE_PROBES, state, },).includes('id="range-x-min"',),);
      },
    },),
    it({
      name: 'changes a dropdown and updates sliders and scene bounds for the full catalog',
      fn: async () => { checkDropdown(); },
    },),
  ],
},);
