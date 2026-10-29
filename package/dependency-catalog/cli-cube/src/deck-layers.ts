/**
 Axis-shaft and arrowhead layer factories.
 
 Two structural elements of the coordinate-system backdrop:
 
 - {@link buildAxisShaftLayer}: three thick black line segments
   running from the min corner outward along +x, +y, +z.
 - {@link buildAxisArrowheadLayers}: three small dark cones at the
   tips of the axis shafts, pointing along the respective axis.
   Pre-rotated cone geometries live in {@link ./deck-geometries.ts}
   so no runtime orientation math is needed.
 
 The tick-mark PathLayer (`buildAxisTickLayer`) moved to
 `./deck-layers-ticks.ts` so each file stays under the 300-line cap.
 Coordinate planes and threshold guide lines live in
 `./deck-planes.ts`; scatter and text-layer factories live in
 `./deck-scatter.ts` and `./deck-labels.ts`.
 
 The shared {@link computeAxisGeometry} helper and {@link getDatumPath}
 accessor are exported so siblings (`deck-layers-ticks.ts`) can reuse
 them without duplication.
 
 @example
 ```ts
 import { buildAxisShaftLayer } from './deck-layers.ts';
 const shaft = buildAxisShaftLayer({ bounds });
 ```
 */

import type { Layer, } from '@deck.gl/core';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { PathLayer, } from '@deck.gl/layers';
import { SimpleMeshLayer, } from '@deck.gl/mesh-layers';

import type { SceneBounds, } from './deck-config.ts';
import {
  coneGeometryX,
  coneGeometryY,
  coneGeometryZ,
} from './deck-geometries.ts';
import type { ChromeColors, } from './script/scheme.ts';

//region Types

/**
 Data shape for the axis-shaft PathLayer; deck.gl expects mutable nested arrays.
 */
export type PathDatum = {
  readonly path: [
    number,
    number,
    number,
  ][];
};

/**
 Data shape for arrowhead mesh-layer instances.
 */
type ArrowheadDatum = {
  readonly position: [
    number,
    number,
    number,
  ];
};

//endregion Types

//region Datum accessors

/**
 Reads the `path` field off a {@link PathDatum} for `PathLayer.getPath`.
 
 Module-scoped to avoid recreating the closure per layer; it captures
 no outer state, so hoisting is safe.
 
 @param d - One PathLayer datum.
 
 @returns The list of 3D points that form the path.
 
 @example
 ```ts
 getDatumPath({ path: [[0, 0, 0], [1, 0, 0]] }); // [[0, 0, 0], [1, 0, 0]]
 ```
 */
export function getDatumPath(d: ForeignBorrowed<PathDatum>,): [
  number,
  number,
  number,
][] {
  return d.path;
}

/**
 Reads the `position` field off an {@link ArrowheadDatum} for `SimpleMeshLayer.getPosition`.
 
 @param d - One arrowhead datum.
 
 @returns The 3D position of the cone base centre.
 
 @example
 ```ts
 getArrowheadPosition({ position: [1, 0, 0] }); // [1, 0, 0]
 ```
 */
function getArrowheadPosition(d: ArrowheadDatum,): [
  number,
  number,
  number,
] {
  return d.position;
}

//endregion Datum accessors

//region Constants

/**
 Axis shaft width in pixels (with `widthMinPixels` floor).
 */
const AXIS_SHAFT_WIDTH = 3;

/**
 Fraction of the axis extent the arrow tip extends past `max`.
 */
const AXIS_EXTENSION_FRACTION = 0.12;
/**
 Cone arrowhead length, as a fraction of the axis extent.
 */
const ARROWHEAD_LENGTH_FRACTION = 0.06;
/**
 Cone arrowhead radius, as a fraction of the axis extent.
 */
const ARROWHEAD_RADIUS_FRACTION = 0.018;

//endregion Constants

//region Helpers

/**
 Computes the per-axis extents and the arrow-tip positions, the most
 common derived quantities across this module.
 
 @param bounds - Scene bounds.
 
 @returns Object with `dx`/`dy`/`dz` extents, `tipX`/`tipY`/`tipZ` tip positions, and the min corner.
 
 @example
 ```ts
 const g = computeAxisGeometry({ bounds: { x: [0, 6], y: [0, 6], z: [0, 6] } });
 g.dx; // 6
 g.tipX; // 6.72  (= 6 + 6 * 0.12)
 ```
 */
export function computeAxisGeometry(
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
  tipX: number;
  tipY: number;
  tipZ: number;
} {
  /**
   Inclusive `[xMin, xMax]` extracted from the X bounds for downstream tip / extent math.
   */
  const [
    xMin,
    xMax,
  ] = bounds.x;
  /**
   Inclusive `[yMin, yMax]` extracted from the Y bounds.
   */
  const [
    yMin,
    yMax,
  ] = bounds.y;
  /**
   Inclusive `[zMin, zMax]` extracted from the Z bounds.
   */
  const [
    zMin,
    zMax,
  ] = bounds.z;
  /**
   X extent of the data box; arrow tips and tick spacings are fractions of this.
   */
  const dx = xMax - xMin;
  /**
   Y extent of the data box.
   */
  const dy = yMax - yMin;
  /**
   Z extent of the data box.
   */
  const dz = zMax - zMin;
  return {
    xMin,
    yMin,
    zMin,
    xMax,
    yMax,
    zMax,
    dx,
    dy,
    dz,
    tipX: xMax + (dx * AXIS_EXTENSION_FRACTION),
    tipY: yMax + (dy * AXIS_EXTENSION_FRACTION),
    tipZ: zMax + (dz * AXIS_EXTENSION_FRACTION),
  };
}

//endregion Helpers

//region Axis shafts + arrowheads

/**
 Builds a single {@link PathLayer} with three line segments; one per
 axis; running from the min corner to the arrow tip position.
 
 @param bounds - Scene bounds.
 
 @param chrome - Theme-aware colour palette.
 
 @returns PathLayer.
 
 @example
 ```ts
 const shaft = buildAxisShaftLayer({
   bounds: { x: [0, 6], y: [0, 6], z: [0, 6] },
   chrome: detectScheme(),
 });
 ```
 */
export function buildAxisShaftLayer(
  {
    bounds,
    chrome,
  }: {
    readonly bounds: SceneBounds;
    readonly chrome: ChromeColors;
  },
): Layer {
  /**
   Cached axis geometry (mins, extents, arrow tips) shared by every path datum below.
   */
  const g = computeAxisGeometry({
    bounds,
  },);
  /**
   Three two-point paths, one per axis, running from the origin corner to each arrow tip.
   */
  const data: PathDatum[] = [
    {
      path: [
        [
          g.xMin,
          g.yMin,
          g.zMin,
        ],
        [
          g.tipX,
          g.yMin,
          g.zMin,
        ],
      ],
    },
    {
      path: [
        [
          g.xMin,
          g.yMin,
          g.zMin,
        ],
        [
          g.xMin,
          g.tipY,
          g.zMin,
        ],
      ],
    },
    {
      path: [
        [
          g.xMin,
          g.yMin,
          g.zMin,
        ],
        [
          g.xMin,
          g.yMin,
          g.tipZ,
        ],
      ],
    },
  ];
  return new PathLayer<PathDatum>({
    id: 'axis-shafts',
    data,
    getPath: getDatumPath,
    getColor: chrome.axis,
    getWidth: AXIS_SHAFT_WIDTH,
    widthUnits: 'pixels',
    widthMinPixels: AXIS_SHAFT_WIDTH,
  },);
}

/**
 Builds three {@link SimpleMeshLayer} instances; one per axis;
 each rendering a single cone at the tip of its axis.
 
 @param bounds - Scene bounds.
 
 @param chrome - Theme-aware colour palette.
 
 @returns Three SimpleMeshLayers (one per axis).
 
 @example
 ```ts
 const [arrowX, arrowY, arrowZ] = buildAxisArrowheadLayers({
   bounds: { x: [0, 6], y: [0, 6], z: [0, 6] },
   chrome: detectScheme(),
 });
 ```
 */
export function buildAxisArrowheadLayers(
  {
    bounds,
    chrome,
  }: {
    readonly bounds: SceneBounds;
    readonly chrome: ChromeColors;
  },
): readonly Layer[] {
  /**
   Cached axis geometry; cone positions and scales derive from its extents.
   */
  const g = computeAxisGeometry({
    bounds,
  },);
  /**
   X arrowhead cone length in world units; scales with X extent so cones stay proportional.
   */
  const coneLengthX = g.dx
    * ARROWHEAD_LENGTH_FRACTION;
  /**
   Y arrowhead cone length; same fraction as X but applied to the Y extent.
   */
  const coneLengthY = g.dy
    * ARROWHEAD_LENGTH_FRACTION;
  /**
   Z arrowhead cone length; same fraction as X but applied to the Z extent.
   */
  const coneLengthZ = g.dz
    * ARROWHEAD_LENGTH_FRACTION;
  /**
   X arrowhead cone base radius; visually balances the cone length.
   */
  const coneRadiusX = g.dx
    * ARROWHEAD_RADIUS_FRACTION;
  /**
   Y arrowhead cone base radius.
   */
  const coneRadiusY = g.dy
    * ARROWHEAD_RADIUS_FRACTION;
  /**
   Z arrowhead cone base radius.
   */
  const coneRadiusZ = g.dz
    * ARROWHEAD_RADIUS_FRACTION;
  return [
    new SimpleMeshLayer<ArrowheadDatum>({
      id: 'arrowhead-x',
      data: [
        {
          position: [
            g.tipX
              - coneLengthX,
            g.yMin,
            g.zMin,
          ],
        },
      ],
      mesh: coneGeometryX,
      getPosition: getArrowheadPosition,
      getColor: chrome.axis,
      getScale: [
        coneLengthX,
        coneRadiusX,
        coneRadiusX,
      ] as const,
      pickable: false,
    },),
    new SimpleMeshLayer<ArrowheadDatum>({
      id: 'arrowhead-y',
      data: [
        {
          position: [
            g.xMin,
            g.tipY
              - coneLengthY,
            g.zMin,
          ],
        },
      ],
      mesh: coneGeometryY,
      getPosition: getArrowheadPosition,
      getColor: chrome.axis,
      getScale: [
        coneRadiusY,
        coneLengthY,
        coneRadiusY,
      ] as const,
      pickable: false,
    },),
    new SimpleMeshLayer<ArrowheadDatum>({
      id: 'arrowhead-z',
      data: [
        {
          position: [
            g.xMin,
            g.yMin,
            g.tipZ
              - coneLengthZ,
          ],
        },
      ],
      mesh: coneGeometryZ,
      getPosition: getArrowheadPosition,
      getColor: chrome.axis,
      getScale: [
        coneRadiusZ,
        coneRadiusZ,
        coneLengthZ,
      ] as const,
      pickable: false,
    },),
  ];
}

//endregion Axis shafts + arrowheads
