/**
 Internal helpers for {@link ./deck-scatter.ts}.
 
 Split out so the scatter file stays within the per-file line cap;
 none of these are part of the public layer-factory surface.
 
 @example
 ```ts
 import { partitionProbes, computeNameBakeSet } from './deck-scatter-helpers.ts';
 const { leaf, nonLeaf, unknown } = partitionProbes({ probes, state });
 ```
 */

import {
  POSITION_UNKNOWN,
  probeIsFilled,
  probePosition,
} from './deck-accessors.ts';
import type { PackageProbe, } from './probe.ts';
import type { AppState, } from './script/state.ts';

//region Types

/**
 Probe + its original index in the source array; preserved through partitioning so visibility lookups stay accurate.
 */
export type ScatterDatum = Readonly<{
  probe: PackageProbe;
  originalIndex: number;
}>;

//endregion Types

//region Constants

/**
 Maximum number of probes that receive baked-name textures under `nameLabels === 'topN'`.
 */
const TOP_N_NAMES = 10;

//endregion Constants

//region Probe partitioning

/**
 Splits the probe array into leaf / non-leaf / unknown buckets,
 preserving the original index of every probe.
 
 @param probes - Full probe array.
 
 @param state - Current state.
 
 @returns Three disjoint arrays.
 
 @example
 ```ts
 const { leaf, nonLeaf, unknown } = partitionProbes({ probes, state });
 // leaf.length + nonLeaf.length + unknown.length === probes.length
 ```
 */
export function partitionProbes(
  {
    probes,
    state,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly state: AppState;
  },
): {
  leaf: readonly ScatterDatum[];
  nonLeaf: readonly ScatterDatum[];
  unknown: readonly ScatterDatum[];
} {
  /**
   Leaf-package bucket (no transitive deps, drawn as filled spheres).
   */
  const leaf: ScatterDatum[] = [];
  /**
   Non-leaf bucket (has transitive deps, drawn as octahedra).
   */
  const nonLeaf: ScatterDatum[] = [];
  /**
   Unknown bucket (missing dim values or flagged via `unknownReason`); drawn at the +max corner.
   */
  const unknown: ScatterDatum[] = [];
  probes.forEach(function bucket(
    probe,
    originalIndex,
  ) {
    if (probe.unknownReason
      !== undefined) {
      unknown.push({
        probe,
        originalIndex,
      },);
      return;
    }
    if (probePosition({
      probe,
      state,
    },)
      === POSITION_UNKNOWN) {
      unknown.push({
        probe,
        originalIndex,
      },);
      return;
    }
    if (probeIsFilled({
      probe,
      state,
    },)) {
      leaf.push({
        probe,
        originalIndex,
      },);
    }
    else {
      nonLeaf.push({
        probe,
        originalIndex,
      },);
    }
  },);
  return {
    leaf,
    nonLeaf,
    unknown,
  };
}

//endregion Probe partitioning

//region Name-bake selection

/**
 Returns the set of `originalIndex` values whose textures should
 include the baked name, given the current `nameLabels` toggle.
 
 - `'none'`: empty set; every probe gets a name-less colour texture.
 - `'all'`: every probe gets a name-baked texture.
 - `'topN'`: only the {@link TOP_N_NAMES} oldest probes (by
   `daysSinceLastCommitOrNull` descending) get baked names.
 
 @param probes - Full probe array.
 
 @param state - Current state (uses `displayToggles.nameLabels`).
 
 @returns Set of original indices to bake the name into.
 
 @example
 ```ts
 const bakeSet = computeNameBakeSet({ probes, state });
 if (bakeSet.has(originalIndex)) renderWithName();
 ```
 */
export function computeNameBakeSet(
  {
    probes,
    state,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly state: AppState;
  },
): ReadonlySet<number> {
  if (state.displayToggles
    .nameLabels
    === 'none')
    return new Set();
  if (state.displayToggles
    .nameLabels
    === 'all') {
    return new Set(probes.map(function indexOf(
      _,
      i,
    ) {
      return i;
    },),);
  }
  /**
   Probes paired with their original indices, sorted oldest-first by last-commit age, then truncated to {@link TOP_N_NAMES}.
   */
  const ranked = probes
    .map(function withIndex(
      probe,
      originalIndex,
    ): ScatterDatum {
      return {
        probe,
        originalIndex,
      };
    },)
    .toSorted(function byStale(
      a,
      b,
    ) {
      return (b.probe
        .daysSinceLastCommitOrNull
        ?? 0)
        - (a.probe
          .daysSinceLastCommitOrNull
          ?? 0);
    },)
    .slice(
    0,
    TOP_N_NAMES,
    );
  return new Set(ranked.map(function pickIndex({ originalIndex, },) {
    return originalIndex;
  },),);
}

//endregion Name-bake selection
