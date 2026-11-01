/**
 Tick-mark PathLayer factory.
 
 Split out from {@link ./deck-layers.ts} so each file stays under the
 300-line cap. Imports the shared axis-geometry helper
 ({@link computeAxisGeometry}) and {@link PathDatum} accessor from
 `deck-layers.ts`.
 
 @example
 ```ts
 import { buildAxisTickLayer } from './deck-layers-ticks.ts';
 const ticks = buildAxisTickLayer({ bounds, chrome: detectScheme() });
 ```
 */

import type { Layer, } from '@deck.gl/core';
import { PathLayer, } from '@deck.gl/layers';

import type { SceneBounds, } from './deck-config.ts';
import {
  computeAxisGeometry,
  getDatumPath,
  type PathDatum,
} from './deck-layers.ts';
import type { ChromeColors, } from './script/scheme.ts';

//region Constants

/**
 Axis tick line width in pixels.
 */
const AXIS_TICK_WIDTH = 1.5;
/**
 Number of tick marks per axis (evenly spaced including endpoints).
 */
const TICK_COUNT = 5;
/**
 Tick mark length, as a fraction of the axis extent.
 */
const TICK_LENGTH_FRACTION = 0.02;

//endregion Constants

//region Tick marks

/**
 Builds the tick-marks PathLayer: short perpendicular segments at
 evenly spaced intervals along each axis, derived from
 {@link computeAxisGeometry}.
 
 @param bounds - Scene bounds.
 
 @param chrome - Theme-aware colour palette.
 
 @returns PathLayer with `3 * (TICK_COUNT + 1)` tick segments.
 
 @example
 ```ts
 const ticks = buildAxisTickLayer({
   bounds: { x: [0, 6], y: [0, 6], z: [0, 6] },
   chrome: detectScheme(),
 });
 ```
 */
export function buildAxisTickLayer(
  {
    bounds,
    chrome,
  }: {
    readonly bounds: SceneBounds;
    readonly chrome: ChromeColors;
  },
): Layer {
  /**
   Cached axis geometry; tick positions and lengths derive from its extents.
   */
  const g = computeAxisGeometry({
    bounds,
  },);
  /**
   Tick mark length on the X axis in world units; reused for Z ticks since both share the X extent.
   */
  const tx = g.dx
    * TICK_LENGTH_FRACTION;
  /**
   Tick mark length on the Y axis in world units.
   */
  const ty = g.dy
    * TICK_LENGTH_FRACTION;
  /**
   `[0, 1/N, …, 1]` normalised positions for the `TICK_COUNT + 1` evenly-spaced ticks.
   */
  const ts: readonly number[] = Array.from(
    {
      length: TICK_COUNT + 1,
    },
    function tForIndex(
      _,
      i,
    ) {
      return i / TICK_COUNT;
    },
  );
  /**
   Flattened tick paths: three perpendicular segments per `t` value, one per axis.
   */
  const ticks: PathDatum[] = ts.flatMap(function tickTriple(t,) {
    /**
     World-space X coordinate of the tick on the X axis for parameter `t`.
     */
    const xAt = g.xMin
      + (g.dx
        * t);
    /**
     World-space Y coordinate of the tick on the Y axis for parameter `t`.
     */
    const yAt = g.yMin
      + (g.dy
        * t);
    /**
     World-space Z coordinate of the tick on the Z axis for parameter `t`.
     */
    const zAt = g.zMin
      + (g.dz
        * t);
    return [
      {
        path: [
          [
            xAt,
            g.yMin
              - ty,
            g.zMin,
          ],
          [
            xAt,
            g.yMin
              + ty,
            g.zMin,
          ],
        ],
      },
      {
        path: [
          [
            g.xMin
              - tx,
            yAt,
            g.zMin,
          ],
          [
            g.xMin
              + tx,
            yAt,
            g.zMin,
          ],
        ],
      },
      {
        path: [
          [
            g.xMin,
            g.yMin
              - ty,
            zAt,
          ],
          [
            g.xMin,
            g.yMin
              + ty,
            zAt,
          ],
        ],
      },
    ];
  },);
  return new PathLayer<PathDatum>({
    id: 'axis-ticks',
    data: ticks,
    getPath: getDatumPath,
    getColor: chrome.axisTick,
    getWidth: AXIS_TICK_WIDTH,
    widthUnits: 'pixels',
    widthMinPixels: AXIS_TICK_WIDTH,
  },);
}

//endregion Tick marks
