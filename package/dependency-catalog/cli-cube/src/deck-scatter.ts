/**
 Glyph mesh-layer factories; one `SimpleMeshLayer` per probe.
 
 Probes are partitioned into three buckets (leaf / non-leaf /
 unknown) by {@link ./deck-scatter-helpers.ts}, then each probe gets
 its own `SimpleMeshLayer` with a per-probe canvas texture from
 {@link ./deck-textures.ts} that bakes the npm name and fill colour
 directly onto the mesh surface. Depth testing then naturally
 occludes back-glyph names behind front glyphs. Spheres for leaf and
 unknown probes, octahedra for non-leaf.
 
 Geometries live in {@link ./deck-geometries.ts}; texture baking in
 {@link ./deck-textures.ts}; per-probe colour / radius / position
 accessors in {@link ./deck-accessors.ts}; partitioning + name-bake
 selection in {@link ./deck-scatter-helpers.ts}.
 
 Per-probe layers (≈ 117 total at this catalog size) trade a higher
 draw-call count for correct depth-of-text rendering. A texture
 atlas with per-instance UV transforms would consolidate this back
 to three draw calls but needs a custom layer extension; not worth
 the complexity at this scale.
 
 @example
 ```ts
 import { buildLeafScatterLayer } from './deck-scatter.ts';
 const layers = buildLeafScatterLayer({ probes, state, bounds, visibleIndices });
 ```
 */

import type { Layer, } from '@deck.gl/core';
import { SimpleMeshLayer, } from '@deck.gl/mesh-layers';
import type { Geometry, } from '@luma.gl/engine';

import {
  POSITION_UNKNOWN,
  probeFillColor,
  probePosition,
  probeRadiusWorld,
  unknownClusterPosition,
} from './deck-accessors.ts';
import type { SceneBounds, } from './deck-config.ts';
import {
  octahedronGeometry,
  sphereGeometry,
} from './deck-geometries.ts';
import {
  computeNameBakeSet,
  partitionProbes,
  type ScatterDatum,
} from './deck-scatter-helpers.ts';
import {
  makeProbeTexture,
  type MeshShape,
  type Rgba,
} from './deck-textures.ts';
import type { PackageProbe, } from './probe.ts';
import type { AppState, } from './script/state.ts';

//region Constants

/**
 Opacity multiplier applied to probes that fail the active filter set. ≈ 5 %.
 */
const OPACITY_FILTERED = 0.05;
/**
 Opacity multiplier for probes that pass every filter.
 */
const OPACITY_VISIBLE = 1;
/**
 Opaque-alpha byte used when baking the per-probe texture (filter fade is applied via `Layer.opacity` instead).
 */
const TEXTURE_ALPHA = 255;

//endregion Constants

//region Per-probe layer factory

/**
 Builds the per-probe `SimpleMeshLayer`. Per-glyph position, scale,
 and texture; per-layer opacity for the filtered-fade effect.
 
 Texture sampling overrides `getColor` in deck.gl's
 `SimpleMeshLayer`, so the colour painted into the canvas IS the
 surface colour; the per-layer opacity supplies the visible /
 filtered fade. Lighting is intentionally left on (`material`
 default) so the mesh keeps its 3D shading.
 
 @param datum - Probe + original index.
 
 @param state - Current state.
 
 @param bounds - Scene bounds.
 
 @param visibleIndices - Set of original indices that pass every filter.
 
 @param mesh - Geometry: sphere or octahedron.
 
 @param shape - Texture-layout selector matching `mesh`.
 
 @param idPrefix - Layer-id prefix; suffixed with `originalIndex` for uniqueness.
 
 @param bake - Whether to draw the npm name into the texture (vs colour-only).
 
 @param positionOverride - Optional explicit position; when set, takes precedence over `probePosition`.
 
 @returns SimpleMeshLayer instance.
 */
function buildProbeLayer(
  {
    datum,
    state,
    bounds,
    visibleIndices,
    mesh,
    shape,
    idPrefix,
    bake,
    positionOverride,
  }: {
    readonly datum: ScatterDatum;
    readonly state: AppState;
    readonly bounds: SceneBounds;
    readonly visibleIndices: ReadonlySet<number>;
    readonly mesh: Geometry;
    readonly shape: MeshShape;
    readonly idPrefix: string;
    readonly bake: boolean;
    readonly positionOverride?: readonly [
      number,
      number,
      number,
    ];
  },
): Layer {
  /**
   `true` when the probe passes every filter; drives the layer's `opacity`.
   */
  const isVisible = visibleIndices.has(datum.originalIndex,);
  /**
   Probe's fill colour computed at full opacity so the baked texture stays opaque.
   */
  const color = probeFillColor({
    probe: datum.probe,
    state,
    bounds,
    isVisible: true,
  },);
  /**
   Same colour with a fixed alpha so the texture canvas paints solid pixels (layer opacity handles fading).
   */
  const opaqueColor: Rgba = [
    color[0],
    color[1],
    color[2],
    TEXTURE_ALPHA,
  ];
  /**
   Probe-specific texture canvas (colour, optional name) reused as the mesh texture.
   */
  const texture = makeProbeTexture({
    probe: datum.probe,
    fillColor: opaqueColor,
    shape,
    withName: bake,
  },);
  /**
   Data-driven position, or {@link POSITION_UNKNOWN} when any spatial dim is unknown for this probe.
   */
  const dataPosition = probePosition({
    probe: datum.probe,
    state,
  },);
  /**
   World-space position; honours the unknown-cluster override, then the data position, then the origin fallback.
   */
  const pos = positionOverride
    ?? (dataPosition === POSITION_UNKNOWN
      ? [
        0,
        0,
        0,
      ]
      : dataPosition);
  /**
   Per-probe scale factor; deck.gl's `getScale` returns the same value on every axis.
   */
  const radius = probeRadiusWorld({
    probe: datum.probe,
    state,
    bounds,
  },);
  return new SimpleMeshLayer<ScatterDatum>({
    id: `${idPrefix}-${datum.originalIndex
      .toString()}`,
    data: [
      datum,
    ],
    mesh,
    texture,
    opacity: isVisible ? OPACITY_VISIBLE : OPACITY_FILTERED,
    getPosition: function getPosition() {
      return pos;
    },
    getScale: function getScale() {
      return [
        radius,
        radius,
        radius,
      ] as const;
    },
    pickable: true,
  },);
}

//endregion Per-probe layer factory

//region Layer factories

/**
 Builds one `SimpleMeshLayer` per leaf probe; sphere mesh, texture
 baked with the probe's colour and (per the toggle) its npm name.
 
 @param probes - Full probe array.
 
 @param state - Current state.
 
 @param bounds - Scene bounds.
 
 @param visibleIndices - Set of original indices that pass every filter.
 
 @returns Array of SimpleMeshLayers, one per leaf probe.
 
 @example
 ```ts
 const layers = buildLeafScatterLayer({ probes, state, bounds, visibleIndices });
 ```
 */
export function buildLeafScatterLayer(
  {
    probes,
    state,
    bounds,
    visibleIndices,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly state: AppState;
    readonly bounds: SceneBounds;
    readonly visibleIndices: ReadonlySet<number>;
  },
): readonly Layer[] {
  /**
   Leaf-only partition; non-leaf and unknown probes are handled by sibling factories.
   */
  const { leaf, } = partitionProbes({
    probes,
    state,
  },);
  /**
   Set of probe indices whose npm name should be baked into the texture this frame.
   */
  const nameSet = computeNameBakeSet({
    probes,
    state,
  },);
  return leaf.map(function asLayer(datum,) {
    return buildProbeLayer({
      datum,
      state,
      bounds,
      visibleIndices,
      mesh: sphereGeometry,
      shape: 'sphere',
      idPrefix: 'scatter-leaf',
      bake: nameSet.has(datum.originalIndex,),
    },);
  },);
}

/**
 Builds one `SimpleMeshLayer` per non-leaf probe; octahedron mesh,
 texture baked with the probe's colour and (per the toggle) its npm
 name on every face.
 
 @param probes - Full probe array.
 
 @param state - Current state.
 
 @param bounds - Scene bounds.
 
 @param visibleIndices - Set of original indices that pass every filter.
 
 @returns Array of SimpleMeshLayers, one per non-leaf probe.
 
 @example
 ```ts
 const layers = buildNonLeafScatterLayer({ probes, state, bounds, visibleIndices });
 ```
 */
export function buildNonLeafScatterLayer(
  {
    probes,
    state,
    bounds,
    visibleIndices,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly state: AppState;
    readonly bounds: SceneBounds;
    readonly visibleIndices: ReadonlySet<number>;
  },
): readonly Layer[] {
  /**
   Non-leaf partition; leaves and unknown probes are handled by sibling factories.
   */
  const { nonLeaf, } = partitionProbes({
    probes,
    state,
  },);
  /**
   Set of probe indices whose npm name should be baked into the texture this frame.
   */
  const nameSet = computeNameBakeSet({
    probes,
    state,
  },);
  return nonLeaf.map(function asLayer(datum,) {
    return buildProbeLayer({
      datum,
      state,
      bounds,
      visibleIndices,
      mesh: octahedronGeometry,
      shape: 'octahedron',
      idPrefix: 'scatter-nonleaf',
      bake: nameSet.has(datum.originalIndex,),
    },);
  },);
}

/**
 Builds one `SimpleMeshLayer` per unknown probe; sphere mesh placed
 at the unknown-cluster jitter position, texture with the mid-grey
 unknown colour and (per the toggle) the npm name.
 
 @param probes - Full probe array.
 
 @param state - Current state.
 
 @param bounds - Scene bounds.
 
 @param visibleIndices - Set of original indices that pass every filter.
 
 @returns Array of SimpleMeshLayers, empty when the bucket has no probes.
 
 @example
 ```ts
 const layers = buildUnknownClusterLayer({ probes, state, bounds, visibleIndices });
 ```
 */
export function buildUnknownClusterLayer(
  {
    probes,
    state,
    bounds,
    visibleIndices,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly state: AppState;
    readonly bounds: SceneBounds;
    readonly visibleIndices: ReadonlySet<number>;
  },
): readonly Layer[] {
  /**
   Unknown-bucket partition; leaves and non-leaves are handled by sibling factories.
   */
  const { unknown, } = partitionProbes({
    probes,
    state,
  },);
  /**
   Set of probe indices whose npm name should be baked into the texture this frame.
   */
  const nameSet = computeNameBakeSet({
    probes,
    state,
  },);
  return unknown.map(function asLayer(datum,) {
    /**
     Jittered cluster position so unknown probes don't stack at the same point.
     */
    const pos = unknownClusterPosition({
      index: datum.originalIndex,
      bounds,
    },);
    return buildProbeLayer({
      datum,
      state,
      bounds,
      visibleIndices,
      mesh: sphereGeometry,
      shape: 'sphere',
      idPrefix: 'scatter-unknown',
      bake: nameSet.has(datum.originalIndex,),
      positionOverride: pos,
    },);
  },);
}

//endregion Layer factories
