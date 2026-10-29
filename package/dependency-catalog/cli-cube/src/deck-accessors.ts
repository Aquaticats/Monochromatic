/**
 Per-probe accessor functions used by deck.gl layer factories.
 
 Pure, side-effect-free: given a probe + the current {@link AppState} +
 the scene bounds + a visibility flag, return the displayed value
 for one visual property (position, color, radius, shape).
 
 Layer factories wrap these into closures suitable for deck.gl's
 `Accessor<T, V>` API. Separating accessors from layer factories
 keeps the layer file under the 300-line cap and lets the accessors
 be unit-tested in isolation against fixture probes.
 
 @example
 ```ts
 const color = probeFillColor({
   probe, state, bounds, isVisible: true,
 });
 ```
 */

import type { SceneBounds, } from './deck-config.ts';
import {
  type Oklch,
  oklchLerpToSrgb,
} from './oklch.ts';
import type { PackageProbe, } from './probe.ts';
import {
  DIM_UNKNOWN,
  extractDim,
} from './script/filter.ts';
import type { AppState, } from './script/state.ts';

//region Constants

/**
 Alpha channel value when a probe passes every filter.
 */
const ALPHA_VISIBLE = 255;
/**
 Alpha channel value when a probe is filtered out (≈ 5% opacity).
 */
const ALPHA_FILTERED = 13;
/**
 Low end of the colour ramp (t = 0). Perceptually red; chroma kept
 under sRGB gamut limits for the chosen lightness.
 */
const COLOR_RAMP_LOW: Oklch = {
  L: 0.65,
  C: 0.22,
  H: 29,
};
/**
 High end of the colour ramp (t = 1). Perceptually green; same chroma
 band as {@link COLOR_RAMP_LOW} so the lerp has stable saturation
 while the hue rotates through amber and yellow at the midpoint.
 */
const COLOR_RAMP_HIGH: Oklch = {
  L: 0.74,
  C: 0.2,
  H: 145,
};
/**
 Per-RGB-channel grey level (0xff/2 ≈ 50% lightness) used for unknown color values.
 */
const COLOR_UNKNOWN_GREY = 136;
/**
 Mid-grey used for unknown color values.
 */
const COLOR_UNKNOWN: readonly [
  number,
  number,
  number,
] = [
  COLOR_UNKNOWN_GREY,
  COLOR_UNKNOWN_GREY,
  COLOR_UNKNOWN_GREY,
];
/**
 Minimum glyph radius in pixels.
 */
const RADIUS_MIN_PX = 3;
/**
 Maximum glyph radius in pixels.
 */
const RADIUS_MAX_PX = 30;
/**
 Minimum glyph radius in world units, as a fraction of the bounds diagonal.
 
 Halved from the iteration-2 value (0.005) so glyphs no longer obscure
 each other in dense regions and so per-glyph name labels have room to
 breathe.
 */
const RADIUS_MIN_WORLD_FRACTION = 0.0025;
/**
 Maximum glyph radius in world units, as a fraction of the bounds diagonal.
 
 Halved from the iteration-2 value (0.03) to match the min reduction.
 */
const RADIUS_MAX_WORLD_FRACTION = 0.015;
/**
 Offset applied to the unknown cluster, in scene-units, from the data box's max corner.
 */
const UNKNOWN_CLUSTER_OFFSET = 2;
/**
 Half-extent of the unknown-cluster jitter cube so glyphs don't pile on one point.
 */
const UNKNOWN_CLUSTER_JITTER = 0.5;
/**
 Binary "is filled" threshold: shape dim values below this render filled, otherwise stroked.
 */
const SHAPE_FILLED_THRESHOLD = 0.5;

//endregion Constants

//region Helpers

/* oxlint-disable eslint/no-magic-numbers -- `0.5` is the degenerate-bounds centre; named const would obscure intent. */
/**
 Maps a value into the unit interval `[0, 1]` given inclusive bounds.
 Degenerate bounds (`lo === hi`) return `0.5` to centre the result.
 
 @param value - Source value.
 
 @param lo - Lower bound, inclusive.
 
 @param hi - Upper bound, inclusive.
 
 @returns `t` in `[0, 1]`.
 
 @example
 ```ts
 normalise({ value: 5, lo: 0, hi: 10 }); // → 0.5
 normalise({ value: 7, lo: 5, hi: 5 });  // → 0.5 (degenerate)
 ```
 */
function normalise(
  {
    value,
    lo,
    hi,
  }: {
    readonly value: number;
    readonly lo: number;
    readonly hi: number;
  },
): number {
  if (hi === lo)
    return 0.5;
  return Math.min(
    1,
    Math.max(
      0,
      (value - lo) / (hi - lo),
    ),
  );
}
/* oxlint-enable eslint/no-magic-numbers */

//endregion Helpers

//region Position

/**
 Absence marker for {@link probePosition} meaning "at least one spatial dim is
 unknown for this probe"; never a `[x, y, z]` coordinate. Probes that return
 it belong in the Unknown-cluster layer instead.
 
 @example
 ```ts
 const pos = probePosition({ probe, state, },);
 if (pos !== POSITION_UNKNOWN)
   draw(pos,);
 ```
 */
export const POSITION_UNKNOWN: unique symbol = Symbol('deps-cube spatial position cannot be computed',);

/**
 Returns the 3D scene-space position of a probe, or {@link POSITION_UNKNOWN}
 when any spatial dim is unknown. Probes returning {@link POSITION_UNKNOWN}
 belong in the Unknown-cluster layer instead.
 
 @param probe - Source probe.
 
 @param state - Current state (uses `dimMapping.x/y/z`).
 
 @returns `[x, y, z]` in scene coords, or {@link POSITION_UNKNOWN} when undefined.
 
 @example
 ```ts
 const pos = probePosition({ probe, state });
 if (pos !== POSITION_UNKNOWN) draw(pos);
 ```
 */
export function probePosition(
  {
    probe,
    state,
  }: {
    readonly probe: PackageProbe;
    readonly state: AppState;
  },
): [
  number,
  number,
  number,
] | typeof POSITION_UNKNOWN {
  /**
   Scene-space X coordinate, or {@link DIM_UNKNOWN} when the X dim is unknown for this probe.
   */
  const x = extractDim({
    probe,
    dim: state.dimMapping
      .x,
  },);
  /**
   Scene-space Y coordinate, or {@link DIM_UNKNOWN} when the Y dim is unknown for this probe.
   */
  const y = extractDim({
    probe,
    dim: state.dimMapping
      .y,
  },);
  /**
   Scene-space Z coordinate, or {@link DIM_UNKNOWN} when the Z dim is unknown for this probe.
   */
  const z = extractDim({
    probe,
    dim: state.dimMapping
      .z,
  },);
  if ((x === DIM_UNKNOWN) || (y === DIM_UNKNOWN)
    || (z === DIM_UNKNOWN))
    return POSITION_UNKNOWN;
  return [
    x,
    y,
    z,
  ];
}

/**
 Returns a deterministic position inside the unknown-cluster region.
 
 Unknown probes are pushed off to one corner of the scene with a
 stable per-index offset so they don't all collapse to one point.
 
 @param index - Probe index (deterministic input).
 
 @param bounds - Scene bounds.
 
 @returns `[x, y, z]` in scene coords.
 
 @example
 ```ts
 const pos = unknownClusterPosition({ index: 0, bounds });
 // → near the +max corner of the data box, plus deterministic jitter
 ```
 */
export function unknownClusterPosition(
  {
    index,
    bounds,
  }: {
    readonly index: number;
    readonly bounds: SceneBounds;
  },
): [
  number,
  number,
  number,
] {
  /**
   Upper-bound corner of the data box on X; the unknown cluster sits beyond this.
   */
  const [
    ,
    xMax,
  ] = bounds.x;
  /**
   Upper-bound corner of the data box on Y; the unknown cluster sits beyond this.
   */
  const [
    ,
    yMax,
  ] = bounds.y;
  /**
   Upper-bound corner of the data box on Z; the unknown cluster sits beyond this.
   */
  const [
    ,
    zMax,
  ] = bounds.z;
  /* oxlint-disable eslint/no-magic-numbers, eslint-plugin-unicorn/number-literal-case, eslint-plugin-unicorn/prefer-math-trunc -- Knuth-multiplicative-hash constant + per-byte shift offsets are intrinsic to the algorithm; the `>>> 0` is the canonical unsigned-32-bit coercion (Math.trunc returns signed); named consts would obscure the bit layout. */
  /**
   Knuth multiplicative hash on the probe index, giving a deterministic 32-bit
   value to derive three independent jitter offsets from.
   */
  const hash = (index * 2_654_435_761) >>> 0;
  /**
   X jitter in `[-UNKNOWN_CLUSTER_JITTER, +UNKNOWN_CLUSTER_JITTER]`, derived from the low byte of `hash`.
   */
  const jx = (((hash & 0xff) / 0xff) - 0.5) * 2
    * UNKNOWN_CLUSTER_JITTER;
  /**
   Y jitter from the second byte of `hash`, same range as `jx`.
   */
  const jy = ((((hash >> 8) & 0xff) / 0xff) - 0.5) * 2
    * UNKNOWN_CLUSTER_JITTER;
  /**
   Z jitter from the third byte of `hash`, same range as `jx`.
   */
  const jz = ((((hash >> 16) & 0xff) / 0xff) - 0.5) * 2
    * UNKNOWN_CLUSTER_JITTER;
  /* oxlint-enable eslint/no-magic-numbers, eslint-plugin-unicorn/number-literal-case, eslint-plugin-unicorn/prefer-math-trunc */
  return [
    xMax + UNKNOWN_CLUSTER_OFFSET
      + jx,
    yMax + UNKNOWN_CLUSTER_OFFSET
      + jy,
    zMax + UNKNOWN_CLUSTER_OFFSET
      + jz,
  ];
}

//endregion Position

//region Color

/**
 Returns the RGBA fill colour for one probe.
 
 - Visible probes use an OKLCH lerp between {@link COLOR_RAMP_LOW}
   (red) and {@link COLOR_RAMP_HIGH} (green) over the colour
   channel's bounds. Perceptually uniform, so the midpoint reads as
   amber/yellow instead of the muddy brown sRGB-space interpolation
   produces.
 - Probes with unknown colour-dim value get {@link COLOR_UNKNOWN} grey.
 - Filtered-out probes get alpha={@link ALPHA_FILTERED} (≈ 5%);
   visible probes get alpha={@link ALPHA_VISIBLE}.
 
 @param probe - Source probe.
 
 @param state - Current state (uses `dimMapping.color`).
 
 @param bounds - Scene bounds.
 
 @param isVisible - `true` when the probe passes every filter.
 
 @returns RGBA tuple, each component in `[0, 255]`.
 
 @example
 ```ts
 const rgba = probeFillColor({ probe, state, bounds, isVisible: true });
 // → [r, g, b, 255]
 ```
 */
export function probeFillColor(
  {
    probe,
    state,
    bounds,
    isVisible,
  }: {
    readonly probe: PackageProbe;
    readonly state: AppState;
    readonly bounds: SceneBounds;
    readonly isVisible: boolean;
  },
): [
  number,
  number,
  number,
  number,
] {
  /**
   Alpha selected by filter visibility so filtered probes fade out instead of disappearing.
   */
  const alpha = isVisible ? ALPHA_VISIBLE : ALPHA_FILTERED;
  /**
   Raw probe value for the colour dim, or {@link DIM_UNKNOWN} when the dim is unknown.
   */
  const value = extractDim({
    probe,
    dim: state.dimMapping
      .color,
  },);
  if (value === DIM_UNKNOWN) {
    return [
      COLOR_UNKNOWN[0],
      COLOR_UNKNOWN[1],
      COLOR_UNKNOWN[2],
      alpha,
    ];
  }
  /**
   Inclusive `[lo, hi]` range for the colour dim across the whole dataset, used to normalise `value`.
   */
  const [
    lo,
    hi,
  ] = bounds.color;
  /**
   Normalised colour-dim value in `[0, 1]`, used as the lerp parameter.
   */
  const t = normalise({
    value,
    lo,
    hi,
  },);
  /**
   sRGB triplet from the OKLCH lerp; perceptually uniform so the midpoint reads as amber rather than mud.
   */
  const [
    r,
    g,
    b,
  ] = oklchLerpToSrgb({
    start: COLOR_RAMP_LOW,
    end: COLOR_RAMP_HIGH,
    t,
  },);
  return [
    r,
    g,
    b,
    alpha,
  ];
}

//endregion Color

//region Radius

/**
 Returns the glyph radius in pixels, linearly interpolated between
 {@link RADIUS_MIN_PX} and {@link RADIUS_MAX_PX} over the size
 channel's bounds. Unknown size-dim values get the minimum radius.
 
 @param probe - Source probe.
 
 @param state - Current state (uses `dimMapping.size`).
 
 @param bounds - Scene bounds.
 
 @returns Radius in pixels.
 
 @example
 ```ts
 const r = probeRadius({ probe, state, bounds });
 // → in [RADIUS_MIN_PX, RADIUS_MAX_PX]
 ```
 */
export function probeRadius(
  {
    probe,
    state,
    bounds,
  }: {
    readonly probe: PackageProbe;
    readonly state: AppState;
    readonly bounds: SceneBounds;
  },
): number {
  /**
   Raw probe value for the size dim, or {@link DIM_UNKNOWN} when the dim is unknown.
   */
  const value = extractDim({
    probe,
    dim: state.dimMapping
      .size,
  },);
  if (value === DIM_UNKNOWN)
    return RADIUS_MIN_PX;
  /**
   Inclusive `[lo, hi]` range for the size dim, used to normalise `value`.
   */
  const [
    lo,
    hi,
  ] = bounds.size;
  /**
   Normalised size-dim value in `[0, 1]`, used to interpolate between min and max radius.
   */
  const t = normalise({
    value,
    lo,
    hi,
  },);
  return RADIUS_MIN_PX + (t * (RADIUS_MAX_PX - RADIUS_MIN_PX));
}

/**
 Returns the glyph radius in world units, scaled to a fraction of the
 scene's bounding-box diagonal. Used by SimpleMeshLayer's `getScale`
 (mesh geometries are constructed with unit radius; the scale factor
 times unit radius yields the rendered world-space size).
 
 Computed as `diagonal * (RADIUS_MIN + t * (RADIUS_MAX - RADIUS_MIN))`
 where `t` is the size-channel value normalised to `[0, 1]`. Unknown
 size values get the minimum radius.
 
 Pixel-space radii (the `probeRadius` companion) keep glyphs the same
 apparent size regardless of zoom; world-space radii scale naturally
 with the camera. Spheres and octahedra need the latter so they look
 like true 3D objects, not screen-aligned sprites.
 
 @param probe - Source probe.
 
 @param state - Current state (uses `dimMapping.size`).
 
 @param bounds - Scene bounds.
 
 @returns Radius in world units.
 
 @example
 ```ts
 const r = probeRadiusWorld({ probe, state, bounds });
 new SimpleMeshLayer({ getScale: () => [r, r, r], ... });
 ```
 */
export function probeRadiusWorld(
  {
    probe,
    state,
    bounds,
  }: {
    readonly probe: PackageProbe;
    readonly state: AppState;
    readonly bounds: SceneBounds;
  },
): number {
  /**
   Width of the scene bounding box along X, one component of the diagonal.
   */
  const dx = bounds.x[1]
    - bounds
      .x[0];
  /**
   Depth of the scene bounding box along Y, one component of the diagonal.
   */
  const dy = bounds.y[1]
    - bounds
      .y[0];
  /**
   Height of the scene bounding box along Z, one component of the diagonal.
   */
  const dz = bounds.z[1]
    - bounds
      .z[0];
  /**
   Bounding-box diagonal length; world-space radii are expressed as fractions of this so they scale with scene size.
   */
  const diagonal = Math.hypot(
    dx,
    dy,
    dz,
  );
  /**
   Raw probe value for the size dim, or {@link DIM_UNKNOWN} when the dim is unknown.
   */
  const value = extractDim({
    probe,
    dim: state.dimMapping
      .size,
  },);
  if (value === DIM_UNKNOWN)
    return diagonal * RADIUS_MIN_WORLD_FRACTION;
  /**
   Inclusive `[lo, hi]` range for the size dim, used to normalise `value`.
   */
  const [
    lo,
    hi,
  ] = bounds.size;
  /**
   Normalised size-dim value in `[0, 1]`, used to interpolate between min and max world-space radius fractions.
   */
  const t = normalise({
    value,
    lo,
    hi,
  },);
  return diagonal
    * (RADIUS_MIN_WORLD_FRACTION
      + (t * (RADIUS_MAX_WORLD_FRACTION - RADIUS_MIN_WORLD_FRACTION)));
}

//endregion Radius

//region Shape

/**
 Returns `true` if the probe should be rendered as a filled glyph
 (vs stroked / hollow). Filled = shape-dim value below
 {@link SHAPE_FILLED_THRESHOLD}; hollow otherwise. Unknown shape
 value defaults to hollow so partially-unknown glyphs are visually
 distinguishable from the all-known set.
 
 @param probe - Source probe.
 
 @param state - Current state (uses `dimMapping.shape`).
 
 @returns `true` if filled, `false` if stroked.
 
 @example
 ```ts
 const filled = probeIsFilled({ probe, state });
 // → true for leaf packages under the default mapping
 ```
 */
export function probeIsFilled(
  {
    probe,
    state,
  }: {
    readonly probe: PackageProbe;
    readonly state: AppState;
  },
): boolean {
  /**
   Raw probe value for the shape dim; {@link DIM_UNKNOWN} falls through to the hollow default below.
   */
  const value = extractDim({
    probe,
    dim: state.dimMapping
      .shape,
  },);
  if (value === DIM_UNKNOWN)
    return false;
  return value < SHAPE_FILLED_THRESHOLD;
}

//endregion Shape
