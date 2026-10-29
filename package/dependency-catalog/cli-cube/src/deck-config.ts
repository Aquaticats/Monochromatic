/**
 Top-level deck.gl scene configuration.
 
 Bundled into the output HTML's runtime via rolldown. Holds the
 `OrbitView` instance, the data-driven scene bounds computation, and
 the main {@link buildLayers} entry point that delegates to
 {@link ./deck-layers.ts} factory functions. Per-probe accessors live
 in {@link ./deck-accessors.ts}.
 
 @example
 ```ts
 import { orbitView, computeSceneBounds, buildLayers } from './deck-config.ts';
 const bounds = computeSceneBounds({ probes, dimMapping: state.dimMapping });
 const layers = buildLayers({ probes, state, visibleIndices, bounds });
 new Deck({ views: [orbitView], layers, ... });
 ```
 */

import {
  type Layer,
  OrbitView,
} from '@deck.gl/core';

import {
  buildAxisCapitalsLayer,
  buildAxisSubtitlesLayer,
  buildOriginLabelLayer,
} from './deck-labels.ts';
import { buildAxisTickLayer, } from './deck-layers-ticks.ts';
import {
  buildAxisArrowheadLayers,
  buildAxisShaftLayer,
} from './deck-layers.ts';
import {
  buildCoordinatePlaneLayers,
  buildThresholdLineLayer,
  NO_THRESHOLD_LAYER,
} from './deck-planes.ts';
import {
  buildLeafScatterLayer,
  buildNonLeafScatterLayer,
  buildUnknownClusterLayer,
} from './deck-scatter.ts';
import type { PackageProbe, } from './probe.ts';
import {
  type ChannelKey,
  DIM_UNKNOWN,
  type DimMapping,
  extractDim,
} from './script/filter.ts';
import type { ChromeColors, } from './script/scheme.ts';
import type { AppState, } from './script/state.ts';

//region Types

/**
 Inclusive min/max bounds along every channel, computed once per render
 from the current dim mapping. Drives axis labels, color normalisation,
 the wireframe extent, and the Unknown-cluster offset position.
 */
export type SceneBounds = Readonly<Record<ChannelKey, readonly [
  number,
  number,
]>>;

//endregion Types

//region Constants

/**
 Channel keys, fixed order for iteration.
 */
const CHANNEL_KEYS: readonly ChannelKey[] = [
  'x',
  'y',
  'z',
  'color',
  'shape',
  'size',
];

/**
 Camera FOV in degrees; OrbitView default is 50.
 */
const CAMERA_FOVY = 50;

/**
 Fallback extent when a channel has no known values across the probe set.
 */
const FALLBACK_EXTENT: readonly [
  number,
  number,
] = [
  0,
  1,
];

//endregion Constants

//region Camera

/**
 Shared OrbitView instance. Drag rotates, shift-drag pans, scroll zooms.
 The `orbitAxis: 'Y'` setting matches the convention where y is "up".
 */
export const orbitView: OrbitView = new OrbitView({
  orbitAxis: 'Y',
  fovy: CAMERA_FOVY,
},);

//endregion Camera

//region Bounds

/**
 Computes inclusive `[min, max]` bounds for every channel given the
 current dim mapping. Unknowns (`extractDim` returning {@link DIM_UNKNOWN}) are
 skipped. Channels with zero known values fall back to {@link FALLBACK_EXTENT}.
 
 Run on every render so dim swaps reflow the camera + axis labels.
 
 @param probes - Full probe array.
 
 @param dimMapping - Current channel → data-dim mapping.
 
 @returns Bounds per channel.
 
 @example
 ```ts
 const bounds = computeSceneBounds({ probes, dimMapping });
 // bounds.x === [0, 5.13] for log10 source bytes
 ```
 */
export function computeSceneBounds(
  {
    probes,
    dimMapping,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly dimMapping: DimMapping;
  },
): SceneBounds {
  /**
   Per-channel `[key, extent]` pairs ready to feed `Object.fromEntries` into a SceneBounds record.
   */
  const entries = CHANNEL_KEYS.map(function extentFor(channel,) {
    /**
     Known values across every probe for this channel; unknowns ({@link DIM_UNKNOWN}) are dropped before min/max.
     */
    const values = probes
      .map(function pluck(probe,) {
        return extractDim({
          probe,
          dim: dimMapping[channel],
        },);
      },)
      .filter(function known(value,): value is number {
        return value !== DIM_UNKNOWN;
      },);
    if (values.length
      === 0) {
      return [
        channel,
        FALLBACK_EXTENT,
      ] as const;
    }
    return [
      channel,
      [
        Math.min(...values,),
        Math.max(...values,),
      ] as const,
    ] as const;
  },);
  /**
   Object built from `entries`; widened by `Object.fromEntries` to `Record<string, ...>` and re-asserted below.
   */
  const record = Object.fromEntries(entries,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- entries exhaust ChannelKey; Object.fromEntries widens to Record<string, V>.
  return record as SceneBounds;
}

//endregion Bounds

//region Public API

/**
 Assembles the deck.gl scene layers for the current state.
 
 Layer order (back-to-front):
 
 1. Coordinate planes (green, semi-transparent)
 2. Threshold guide lines (on the planes)
 3. Axis shafts (thick black PathLayer)
 4. Axis tick marks
 5. Axis arrowhead cones
 6. Unknown cluster (spheres at the +max corner)
 7. Leaf scatter (spheres)
 8. Non-leaf scatter (octahedra)
 9. Origin label (`O`)
 10. Axis capital labels (`X`, `Y`, `Z`)
 11. Axis dim-name subtitles
 12. Per-glyph name labels (when toggled)
 
 Display-toggles skip individual layers; `null` returns from
 factories (e.g. empty unknown cluster, no threshold guides) are
 filtered out. The `showWireframe` toggle now controls the
 coordinate planes (the bounding-box wireframe is gone).
 
 @param probes - Full probe array.
 
 @param state - Current {@link AppState}.
 
 @param visibleIndices - Probes that pass every filter; others fade to 5% alpha.
 
 @param bounds - Output of {@link computeSceneBounds}.
 
 @returns Array of deck.gl `Layer` instances ready for `new Deck({ layers })`.
 
 @example
 ```ts
 deck.setProps({ layers: buildLayers({ probes, state, visibleIndices, bounds }) });
 ```
 */
export function buildLayers(
  {
    probes,
    state,
    visibleIndices,
    bounds,
    chrome,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly state: AppState;
    readonly visibleIndices: ReadonlySet<number>;
    readonly bounds: SceneBounds;
    readonly chrome: ChromeColors;
  },
): readonly Layer[] {
  /**
   Unknown-cluster scatter layers when the toggle is on; empty otherwise so the group flattens to nothing.
   */
  const unknownCluster = state.displayToggles
    .showUnknownCluster
    ? buildUnknownClusterLayer({
      probes,
      state,
      bounds,
      visibleIndices,
    },)
    : [];
  /**
   Threshold guide-line layer when the toggle is on; {@link NO_THRESHOLD_LAYER} is filtered out before flattening.
   */
  const thresholdLines = state.displayToggles
    .showThresholdPlanes
    ? buildThresholdLineLayer({
      bounds,
      dimMapping: state.dimMapping,
    },)
    : NO_THRESHOLD_LAYER;
  /**
   Layer groups in back-to-front order; flattened below into the final layer array deck.gl renders.
   */
  const groups: readonly (readonly Layer[])[] = [
    state.displayToggles
      .showWireframe
      ? buildCoordinatePlaneLayers({
        bounds,
      },)
      : [],
    thresholdLines === NO_THRESHOLD_LAYER ? [] : [thresholdLines,],
    [
      buildAxisShaftLayer({
        bounds,
        chrome,
      },),
    ],
    [
      buildAxisTickLayer({
        bounds,
        chrome,
      },),
    ],
    buildAxisArrowheadLayers({
      bounds,
      chrome,
    },),
    unknownCluster,
    buildLeafScatterLayer({
      probes,
      state,
      bounds,
      visibleIndices,
    },),
    buildNonLeafScatterLayer({
      probes,
      state,
      bounds,
      visibleIndices,
    },),
    state.displayToggles
      .showAxisLabels
      ? [
        buildOriginLabelLayer({
          bounds,
          chrome,
        },),
        buildAxisCapitalsLayer({
          bounds,
          chrome,
        },),
        buildAxisSubtitlesLayer({
          bounds,
          dimMapping: state.dimMapping,
          chrome,
        },),
      ]
      : [],
  ];
  return groups.flat();
}

//endregion Public API
