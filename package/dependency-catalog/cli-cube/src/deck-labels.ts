/**
 Text-layer factories; axis tip capitals, axis dim subtitles, origin
 marker.
 
 The axis-label rendering is split across **two** TextLayer
 instances: one for the bold capital `X`/`Y`/`Z` letters that sit at
 the arrow tips, another for the smaller dim-name subtitles. The
 origin gets a single TextLayer with the character `O`. Splitting by
 purpose keeps font sizes, anchor strategies, and per-instance
 positions independent; a single combined layer would have to
 compromise.
 
 Per-glyph package-name labels used to live here too as a separate
 `TextLayer`, but iteration-5 moved them onto the mesh surface itself
 via a per-probe canvas texture (see `./deck-textures.ts` and
 `./deck-scatter.ts`) so depth testing correctly hides the name when
 the glyph rotates away from the camera.
 
 Split out from `deck-layers.ts` to stay under the 300-line cap.
 
 Iteration-2 dropped the opaque-white label backgrounds (they
 dominated dark-mode scenes) and threaded a {@link ChromeColors}
 palette through every factory so colours respect
 `prefers-color-scheme`. Subtitles moved from the arrow-tip
 neighbourhood to the axis midpoint to clear the capitals.
 
 @example
 ```ts
 import { buildAxisCapitalsLayer } from './deck-labels.ts';
 import { detectScheme } from './script/scheme.ts';
 const layer = buildAxisCapitalsLayer({ bounds, chrome: detectScheme() });
 ```
 */

import type { Layer, } from '@deck.gl/core';
import { TextLayer, } from '@deck.gl/layers';

import type { SceneBounds, } from './deck-config.ts';
import { DIM_DISPLAY_NAMES, } from './dim-meta.ts';
import type { DimMapping, } from './script/filter.ts';
import type { ChromeColors, } from './script/scheme.ts';

//region Types

/**
 Data shape for the TextLayer.
 */
type TextDatum = {
  readonly position: [
    number,
    number,
    number,
  ];
  readonly text: string;
};

//endregion Types

//region Datum accessors

/**
 Reads the `position` field off a {@link TextDatum} for `TextLayer.getPosition`.
 
 Module-scoped to avoid recreating the closure per layer; it captures
 no outer state, so hoisting is safe.
 
 @param d - One TextLayer datum.
 
 @returns The 3D position tuple for the label.
 
 @example
 ```ts
 getDatumPosition({ position: [0, 0, 0], text: 'O' }); // [0, 0, 0]
 ```
 */
function getDatumPosition(d: TextDatum,): [
  number,
  number,
  number,
] {
  return d.position;
}

/**
 Reads the `text` field off a {@link TextDatum} for `TextLayer.getText`.
 
 @param d - One TextLayer datum.
 
 @returns The label string to render at the datum's position.
 
 @example
 ```ts
 getDatumText({ position: [0, 0, 0], text: 'X' }); // 'X'
 ```
 */
function getDatumText(d: TextDatum,): string {
  return d.text;
}

//endregion Datum accessors

//region Constants

/**
 Axis-tip capital-letter font size in pixels. Larger than subtitles for hierarchy.
 */
const TIP_LABEL_SIZE_PX = 24;
/**
 Axis dim-name subtitle font size in pixels. Smaller than iteration-1
 (was 12) so the secondary text doesn't compete with the capitals.
 */
const SUBTITLE_LABEL_SIZE_PX = 10;
/**
 Origin marker font size in pixels.
 */
const ORIGIN_LABEL_SIZE_PX = 18;
/**
 Half-coefficient used for centring helpers.
 */
const HALF = 1 / 2;

/**
 Fraction of the axis extent that the capital sits past the arrow tip.
 
 Iteration-1 used 0.18; bumped to 0.22 so capitals never collide with
 the cone arrowheads even on axes with tight extents.
 */
const TIP_LABEL_OFFSET_FRACTION = 0.22;
/**
 Fraction of the axis extent the dim-name subtitle is offset from the
 data box. Subtitles now sit at the axis midpoint (between min and
 max), offset OUTWARD perpendicular to the axis by this fraction.
 */
const SUBTITLE_OFFSET_FRACTION = 0.05;
/**
 Fraction of the axis extent the origin label sits behind the min
 corner. Bumped from 0.04 to 0.08 so the `O` clears the data box and
 the axis-shaft origin point.
 */
const ORIGIN_OFFSET_FRACTION = 0.08;

//endregion Constants

//region Helpers

/**
 Computes the per-axis extents shared by the layer factories.
 
 @param bounds - Scene bounds.
 
 @returns Object with min/max and delta for each axis.
 */
function axisExtents(
  { bounds, }: { readonly bounds: SceneBounds; },
): {
  xMin: number;
  yMin: number;
  zMin: number;
  xMax: number;
  yMax: number;
  zMax: number;
  dx: number;
  dy: number;
  dz: number;
} {
  /**
   X-axis min and max destructured from `bounds.x` for use in delta math.
   */
  const [
    xMin,
    xMax,
  ] = bounds.x;
  /**
   Y-axis min and max destructured from `bounds.y` for use in delta math.
   */
  const [
    yMin,
    yMax,
  ] = bounds.y;
  /**
   Z-axis min and max destructured from `bounds.z` for use in delta math.
   */
  const [
    zMin,
    zMax,
  ] = bounds.z;
  return {
    xMin,
    yMin,
    zMin,
    xMax,
    yMax,
    zMax,
    dx: xMax - xMin,
    dy: yMax - yMin,
    dz: zMax - zMin,
  };
}

//endregion Helpers

//region Axis tip capitals

/**
 Builds the bold `X` / `Y` / `Z` capital letters at the arrow tips.
 
 Centred on the position via `getTextAnchor: 'middle'` + the
 `getAlignmentBaseline: 'center'` defaults so the capital sits
 cleanly past the cone arrowhead.
 
 @param bounds - Scene bounds.
 
 @param chrome - Theme-aware colour palette.
 
 @returns TextLayer with three capital-letter labels.
 
 @example
 ```ts
 const layer = buildAxisCapitalsLayer({
   bounds: { x: [0, 6], y: [0, 6], z: [0, 6] },
   chrome: detectScheme(),
 });
 ```
 */
export function buildAxisCapitalsLayer(
  {
    bounds,
    chrome,
  }: {
    readonly bounds: SceneBounds;
    readonly chrome: ChromeColors;
  },
): Layer {
  /**
   Per-axis min/max/delta extents shared across position computations.
   */
  const g = axisExtents({
    bounds,
  },);
  /**
   X-axis offset past the arrow tip so capital `X` clears the cone.
   */
  const offX = g.dx
    * TIP_LABEL_OFFSET_FRACTION;
  /**
   Y-axis offset past the arrow tip so capital `Y` clears the cone.
   */
  const offY = g.dy
    * TIP_LABEL_OFFSET_FRACTION;
  /**
   Z-axis offset past the arrow tip so capital `Z` clears the cone.
   */
  const offZ = g.dz
    * TIP_LABEL_OFFSET_FRACTION;
  /**
   Three capital-letter labels, one per axis tip.
   */
  const data: TextDatum[] = [
    {
      position: [
        g.xMax
          + offX,
        g.yMin,
        g.zMin,
      ],
      text: 'X',
    },
    {
      position: [
        g.xMin,
        g.yMax
          + offY,
        g.zMin,
      ],
      text: 'Y',
    },
    {
      position: [
        g.xMin,
        g.yMin,
        g.zMax
          + offZ,
      ],
      text: 'Z',
    },
  ];
  return new TextLayer<TextDatum>({
    id: 'axis-capitals',
    data,
    getPosition: getDatumPosition,
    getText: getDatumText,
    getSize: TIP_LABEL_SIZE_PX,
    getColor: chrome.axisLabel,
    sizeUnits: 'pixels',
    fontFamily: 'serif',
    getTextAnchor: 'middle',
    getAlignmentBaseline: 'center',
  },);
}

//endregion Axis tip capitals

//region Axis dim-name subtitles

/**
 Builds the smaller dim-name subtitle labels (`log10(source bytes)`,
 etc.), drawn just inside each arrow tip along the axis line.
 
 @param bounds - Scene bounds.
 
 @param dimMapping - Current dim mapping.
 
 @param chrome - Theme-aware colour palette.
 
 @returns TextLayer with three subtitle labels.
 
 @example
 ```ts
 const layer = buildAxisSubtitlesLayer({
   bounds: { x: [0, 6], y: [0, 6], z: [0, 6] },
   dimMapping: { x: 'logSourceBytes', y: 'logDownloads', z: 'tsRatio', color: 'logDaysStale', shape: 'isLeafNumeric', size: 'logInstallSize' },
   chrome: detectScheme(),
 });
 ```
 */
export function buildAxisSubtitlesLayer(
  {
    bounds,
    dimMapping,
    chrome,
  }: {
    readonly bounds: SceneBounds;
    readonly dimMapping: DimMapping;
    readonly chrome: ChromeColors;
  },
): Layer {
  /**
   Per-axis min/max/delta extents shared across position computations.
   */
  const g = axisExtents({
    bounds,
  },);
  /**
   Perpendicular X offset that pushes subtitles outward from the data box.
   */
  const offX = g.dx
    * SUBTITLE_OFFSET_FRACTION;
  /**
   Perpendicular Y offset that pushes subtitles outward from the data box.
   */
  const offY = g.dy
    * SUBTITLE_OFFSET_FRACTION;
  /**
   Three dim-name subtitle labels positioned at each axis midpoint.
   */
  const data: TextDatum[] = [
    {
      position: [
        (g.xMin
          + g
            .xMax) * HALF,
        g.yMin
          - offY,
        g.zMin,
      ],
      text: DIM_DISPLAY_NAMES[dimMapping.x],
    },
    {
      position: [
        g.xMin
          - offX,
        (g.yMin
          + g
            .yMax) * HALF,
        g.zMin,
      ],
      text: DIM_DISPLAY_NAMES[dimMapping.y],
    },
    {
      position: [
        g.xMin,
        g.yMin
          - offY,
        (g.zMin
          + g
            .zMax) * HALF,
      ],
      text: DIM_DISPLAY_NAMES[dimMapping.z],
    },
  ];
  return new TextLayer<TextDatum>({
    id: 'axis-subtitles',
    data,
    getPosition: getDatumPosition,
    getText: getDatumText,
    getSize: SUBTITLE_LABEL_SIZE_PX,
    getColor: chrome.axisLabel,
    sizeUnits: 'pixels',
    fontFamily: 'monospace',
    getTextAnchor: 'middle',
    getAlignmentBaseline: 'center',
  },);
}

//endregion Axis dim-name subtitles

//region Origin marker

/**
 Builds the origin marker; a single `O` character at the min corner
 of the data box, offset slightly into the back wall so it doesn't
 collide with the axis shafts.
 
 @param bounds - Scene bounds.
 
 @param chrome - Theme-aware colour palette.
 
 @returns TextLayer with one origin label.
 
 @example
 ```ts
 const layer = buildOriginLabelLayer({
   bounds: { x: [0, 6], y: [0, 6], z: [0, 6] },
   chrome: detectScheme(),
 });
 ```
 */
export function buildOriginLabelLayer(
  {
    bounds,
    chrome,
  }: {
    readonly bounds: SceneBounds;
    readonly chrome: ChromeColors;
  },
): Layer {
  /**
   Per-axis min/max/delta extents shared across position computations.
   */
  const g = axisExtents({
    bounds,
  },);
  /**
   X-axis offset that nudges `O` behind the min corner so it clears the shaft.
   */
  const offX = g.dx
    * ORIGIN_OFFSET_FRACTION;
  /**
   Y-axis offset that nudges `O` behind the min corner so it clears the shaft.
   */
  const offY = g.dy
    * ORIGIN_OFFSET_FRACTION;
  /**
   Z-axis offset that nudges `O` behind the min corner so it clears the shaft.
   */
  const offZ = g.dz
    * ORIGIN_OFFSET_FRACTION;
  /**
   Single-entry data array; only the origin marker is rendered by this layer.
   */
  const data: TextDatum[] = [
    {
      position: [
        g.xMin
          - offX,
        g.yMin
          - offY,
        g.zMin
          - offZ,
      ],
      text: 'O',
    },
  ];
  return new TextLayer<TextDatum>({
    id: 'origin-label',
    data,
    getPosition: getDatumPosition,
    getText: getDatumText,
    getSize: ORIGIN_LABEL_SIZE_PX,
    getColor: chrome.originLabel,
    sizeUnits: 'pixels',
    fontFamily: 'serif',
    getTextAnchor: 'middle',
    getAlignmentBaseline: 'center',
  },);
}

//endregion Origin marker
