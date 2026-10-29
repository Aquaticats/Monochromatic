/**
 Coordinate-plane and threshold-guide layer factories.
 
 Holds the green coordinate planes that intersect at the data box's
 min corner and the thin threshold-guide line layer that replaces
 the previous opaque threshold planes. Split out from
 `deck-layers.ts` so each layer file stays under the 300-line cap.
 
 Uses {@link SolidPolygonLayer} with `_full3d: true` rather than the
 higher-level `PolygonLayer`. `SolidPolygonLayer` is the only deck.gl
 layer that exposes `_full3d`, and without it the floor and side wall
 tessellate to zero triangles: their XY projection is a degenerate
 line, and earcut produces nothing. With `_full3d`, deck.gl picks the
 largest-area plane (xy / xz / yz) for tessellation, then permutes
 back; verified by reading the upstream
 `solid-polygon-layer/polygon.ts` getSurfaceIndices implementation.
 
 @example
 ```ts
 import { buildCoordinatePlaneLayers } from './deck-planes.ts';
 const layers = buildCoordinatePlaneLayers({ bounds });
 ```
 */

import type { Layer, } from '@deck.gl/core';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  PathLayer,
  SolidPolygonLayer,
} from '@deck.gl/layers';

import type { SceneBounds, } from './deck-config.ts';
import type { DimMapping, } from './script/filter.ts';

//region Types

/**
 Data shape for the PolygonLayer (coordinate planes); mutable arrays per deck.gl typings.
 */
type PolygonDatum = {
  readonly polygon: [
    number,
    number,
    number,
  ][];
};

/**
 Data shape for the PathLayer (threshold guides); mutable arrays per deck.gl typings.
 */
type PathDatum = {
  readonly path: [
    number,
    number,
    number,
  ][];
};

//endregion Types

//region Constants

/**
 Source-bytes value at the "300 SLOC" boundary; log10'd below.
 */
const SOURCE_BYTES_AT_300_SLOC = 10_000;
/**
 Days-since-commit value at the "1 year stale" boundary.
 */
const DAYS_STALE_AT_ONE_YEAR = 365;
/**
 Install-size value at the "100KB soft boundary"; log10'd below.
 */
const INSTALL_BYTES_AT_100KB = 100_000;
/**
 Source-bytes threshold (~ "300 SLOC" boundary), on log10.
 */
const SOURCE_BYTES_THRESHOLD = Math.log10(SOURCE_BYTES_AT_300_SLOC,);
/**
 Days-since-commit threshold (1 year), on log10.
 */
const DAYS_STALE_THRESHOLD = Math.log10(DAYS_STALE_AT_ONE_YEAR,);
/**
 Install-size threshold (~100KB soft boundary), on log10.
 */
const INSTALL_SIZE_THRESHOLD = Math.log10(INSTALL_BYTES_AT_100KB,);

/**
 Coordinate-plane fill colour: pale green, ~24% opacity.
 
 Iteration-1 used alpha 30 (12%) which was effectively invisible against
 the busy scene. The reference image's planes read more clearly; alpha
 60 gets visibly translucent without fully obscuring the data.
 */
/* oxlint-disable eslint/no-magic-numbers -- rgba components and line-width pixel are domain values; named consts would obscure the colour intent. */
const COORDINATE_PLANE_COLOR: readonly [
  number,
  number,
  number,
  number,
] = [
  140,
  200,
  140,
  60,
];

/**
 Threshold-line colour: muted brown so the line reads as a heuristic guide, not part of the axes.
 */
const THRESHOLD_LINE_COLOR: readonly [
  number,
  number,
  number,
  number,
] = [
  150,
  100,
  60,
  200,
];

/**
 Plane margin past the data box on the +axis side, as a fraction of the axis extent.
 */
const PLANE_MARGIN_FRACTION = 0.05;
/**
 Threshold-guide line width in pixels.
 */
const THRESHOLD_LINE_WIDTH = 1.5;
/* oxlint-enable eslint/no-magic-numbers */

//endregion Constants

//region Accessors

/**
 Returns a `PolygonDatum`'s `polygon` field; deck.gl's `SolidPolygonLayer` calls this per datum.
 
 @param d - Source polygon datum.
 
 @returns The polygon's ring of vertices.
 
 @example
 ```ts
 new SolidPolygonLayer<PolygonDatum>({ getPolygon, ... });
 ```
 */
function getPolygonAccessor(
  d: ForeignBorrowed<PolygonDatum>,
): PolygonDatum['polygon'] {
  return d.polygon;
}

/**
 Returns a `PathDatum`'s `path` field; deck.gl's `PathLayer` calls this per datum.
 
 @param d - Source path datum.
 
 @returns The path's vertex list.
 
 @example
 ```ts
 new PathLayer<PathDatum>({ getPath: getPathAccessor, ... });
 ```
 */
function getPathAccessor(d: ForeignBorrowed<PathDatum>,): PathDatum['path'] {
  return d.path;
}

//endregion Accessors

//region Coordinate planes

/**
 Builds the three semi-transparent green coordinate planes that
 intersect at the data box's min corner. Visually matches the
 reference image's role: three quadrant walls forming a 3D
 coordinate system.
 
 @param bounds - Scene bounds.
 
 @returns Three PolygonLayers (floor / back / side).
 
 @example
 ```ts
 const planes = buildCoordinatePlaneLayers({ bounds });
 // → [floor, back, side] SolidPolygonLayer instances
 ```
 */
export function buildCoordinatePlaneLayers(
  { bounds, }: { readonly bounds: SceneBounds; },
): readonly Layer[] {
  /**
   X-axis min and max destructured from `bounds.x` for polygon corner math.
   */
  const [
    xMin,
    xMax,
  ] = bounds.x;
  /**
   Y-axis min and max destructured from `bounds.y` for polygon corner math.
   */
  const [
    yMin,
    yMax,
  ] = bounds.y;
  /**
   Z-axis min and max destructured from `bounds.z` for polygon corner math.
   */
  const [
    zMin,
    zMax,
  ] = bounds.z;
  /**
   X-axis total extent; basis for the X margin.
   */
  const dx = xMax - xMin;
  /**
   Y-axis total extent; basis for the Y margin.
   */
  const dy = yMax - yMin;
  /**
   Z-axis total extent; basis for the Z margin.
   */
  const dz = zMax - zMin;
  /**
   Outward X margin so planes overhang the data box on the +/- X faces.
   */
  const mx = dx * PLANE_MARGIN_FRACTION;
  /**
   Outward Y margin so planes overhang the data box on the +/- Y faces.
   */
  const my = dy * PLANE_MARGIN_FRACTION;
  /**
   Outward Z margin so planes overhang the data box on the +/- Z faces.
   */
  const mz = dz * PLANE_MARGIN_FRACTION;
  /**
   Floor polygon (XZ plane at `yMin`); first of the three coordinate walls.
   */
  const floor: PolygonDatum = {
    polygon: [
      [
        xMin - mx,
        yMin,
        zMin - mz,
      ],
      [
        xMax + mx,
        yMin,
        zMin - mz,
      ],
      [
        xMax + mx,
        yMin,
        zMax + mz,
      ],
      [
        xMin - mx,
        yMin,
        zMax + mz,
      ],
    ],
  };
  /**
   Back polygon (XY plane at `zMin`); second of the three coordinate walls.
   */
  const back: PolygonDatum = {
    polygon: [
      [
        xMin - mx,
        yMin - my,
        zMin,
      ],
      [
        xMax + mx,
        yMin - my,
        zMin,
      ],
      [
        xMax + mx,
        yMax + my,
        zMin,
      ],
      [
        xMin - mx,
        yMax + my,
        zMin,
      ],
    ],
  };
  /**
   Side polygon (YZ plane at `xMin`); third of the three coordinate walls.
   */
  const side: PolygonDatum = {
    polygon: [
      [
        xMin,
        yMin - my,
        zMin - mz,
      ],
      [
        xMin,
        yMin - my,
        zMax + mz,
      ],
      [
        xMin,
        yMax + my,
        zMax + mz,
      ],
      [
        xMin,
        yMax + my,
        zMin - mz,
      ],
    ],
  };
  return [
    new SolidPolygonLayer<PolygonDatum>({
      id: 'plane-floor',
      data: [
        floor,
      ],
      getPolygon: getPolygonAccessor,
      getFillColor: COORDINATE_PLANE_COLOR,
      _full3d: true,
    },),
    new SolidPolygonLayer<PolygonDatum>({
      id: 'plane-back',
      data: [
        back,
      ],
      getPolygon: getPolygonAccessor,
      getFillColor: COORDINATE_PLANE_COLOR,
      _full3d: true,
    },),
    new SolidPolygonLayer<PolygonDatum>({
      id: 'plane-side',
      data: [
        side,
      ],
      getPolygon: getPolygonAccessor,
      getFillColor: COORDINATE_PLANE_COLOR,
      _full3d: true,
    },),
  ];
}

//endregion Coordinate planes

//region Threshold guide lines

/**
 Absence marker for {@link buildThresholdLineLayer} meaning "no threshold
 guide segment qualifies under the current dim mapping"; never a layer.
 
 @example
 ```ts
 const layer = buildThresholdLineLayer({ bounds, dimMapping, },);
 if (layer !== NO_THRESHOLD_LAYER)
   layers.push(layer,);
 ```
 */
export const NO_THRESHOLD_LAYER: unique symbol = Symbol('deps-cube/no-threshold-layer',);

/**
 Builds a thin {@link PathLayer} drawing the three threshold guide
 lines (300 SLOC, 365 days, 100KB) on the coordinate planes when
 the relevant channel's dim mapping matches.
 
 @param bounds - Scene bounds.
 
 @param dimMapping - Current dim mapping.
 
 @returns PathLayer with zero to three guide segments, or {@link NO_THRESHOLD_LAYER} if none qualify.
 
 @example
 ```ts
 const layer = buildThresholdLineLayer({ bounds, dimMapping: state.dimMapping });
 if (layer !== NO_THRESHOLD_LAYER) layers.push(layer);
 ```
 */
export function buildThresholdLineLayer(
  {
    bounds,
    dimMapping,
  }: {
    readonly bounds: SceneBounds;
    readonly dimMapping: DimMapping;
  },
): Layer | typeof NO_THRESHOLD_LAYER {
  /**
   X-axis min and max destructured from `bounds.x` for guide-line endpoints.
   */
  const [
    xMin,
    xMax,
  ] = bounds.x;
  /**
   Y-axis min and max destructured from `bounds.y` for guide-line endpoints.
   */
  const [
    yMin,
    yMax,
  ] = bounds.y;
  /**
   Z-axis minimum; guides sit on the back wall at `zMin`.
   */
  const [
    zMin,
  ] = bounds.z;
  /**
   Accumulator for guide-line paths; one entry per threshold whose dim is mapped and within bounds.
   */
  const segments: PathDatum[] = [];
  if (
    (dimMapping.x
      === 'logSourceBytes')
      && (SOURCE_BYTES_THRESHOLD > xMin)
      && (SOURCE_BYTES_THRESHOLD < xMax)
  ) {
    segments.push({
      path: [
        [
          SOURCE_BYTES_THRESHOLD,
          yMin,
          zMin,
        ],
        [
          SOURCE_BYTES_THRESHOLD,
          yMax,
          zMin,
        ],
      ],
    },);
  }
  if (
    (dimMapping.y
      === 'logDaysStale')
      && (DAYS_STALE_THRESHOLD > yMin)
      && (DAYS_STALE_THRESHOLD < yMax)
  ) {
    segments.push({
      path: [
        [
          xMin,
          DAYS_STALE_THRESHOLD,
          zMin,
        ],
        [
          xMax,
          DAYS_STALE_THRESHOLD,
          zMin,
        ],
      ],
    },);
  }
  if (
    (dimMapping.z
      === 'logInstallSize')
      && (INSTALL_SIZE_THRESHOLD > bounds
        .z[0])
      && (INSTALL_SIZE_THRESHOLD < bounds
        .z[1])
  ) {
    segments.push({
      path: [
        [
          xMin,
          yMin,
          INSTALL_SIZE_THRESHOLD,
        ],
        [
          xMax,
          yMin,
          INSTALL_SIZE_THRESHOLD,
        ],
      ],
    },);
  }
  if (segments.length
    === 0)
    return NO_THRESHOLD_LAYER;
  return new PathLayer<PathDatum>({
    id: 'threshold-guides',
    data: segments,
    getPath: getPathAccessor,
    getColor: THRESHOLD_LINE_COLOR,
    getWidth: THRESHOLD_LINE_WIDTH,
    widthUnits: 'pixels',
    widthMinPixels: THRESHOLD_LINE_WIDTH,
  },);
}

//endregion Threshold guide lines
