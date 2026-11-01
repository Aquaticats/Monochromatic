/**
 * Splitting fracture cells into the blasted-out hole and the holding rim.
 *
 * A ball strike no longer converts the whole pane at once: cells around
 * the impact fly immediately as shards (Smash Hit's "objects always break
 * where they get hit"), while cells beyond the hole radius survive as a
 * cracked rim with a real opening. Pure math on plain number pairs, so
 * the whole module unit-tests in node.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  type PaneCell,
  type PanePoint,
  polygonCentroid,
  type RandomSource,
} from './fracture.ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the fracture partition module.
 */
const l = tagged({
  tag: 'fracture-partition',
  l: parentLogger,
},);

/**
 * Cross products this close to zero count as on-edge, which reads as
 * inside; edges are shared between cells, so either owner may claim
 * boundary points.
 */
const EDGE_EPSILON = 1e-9;

/**
 * Fraction of the hole radius each hole cell's escape may vary by, so
 * hole boundaries come out jagged instead of circular.
 */
const HOLE_JITTER = 0.25;

/**
 * One fracture split into flying and holding cells.
 */
export type HolePartition = {
  /**
   * Cells blasted out immediately, becoming shards at strike time.
   */
  readonly hole: readonly PaneCell[];
  /**
   * Cells still holding, drawn as cracked glass until collapse.
   */
  readonly rim: readonly PaneCell[];
};

/**
 * Tests whether a point lies inside a convex polygon, tolerant of either
 * winding direction.
 *
 * @param polygon - convex polygon under test
 *
 * @param point - candidate point in pane-local meters
 *
 * @returns true when inside or on an edge
 *
 * @example
 * ```ts
 * pointInConvexPolygon({
 *   polygon: [{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 0, y: 1 }],
 *   point: { x: 0.2, y: 0.2 },
 * },);
 * // true
 * ```
 */
export function pointInConvexPolygon(
  {
    polygon,
    point,
  }: {
    readonly polygon: PaneCell;
    readonly point: PanePoint;
  },
): boolean {
  /**
   * Edge cross products; consistent sign means the point never leaves
   * one side of the boundary walk.
   */
  const crosses = polygon.map(
    function edgeCross(
      vertex: PanePoint,
      index: number,
    ): number {
      /**
       * Edge partner: next vertex, wrapping to close the polygon.
       */
      const next = polygon[(index + 1) % polygon.length] ?? vertex;
      return ((next.x - vertex.x) * (point.y - vertex.y))
        - ((next.y - vertex.y) * (point.x - vertex.x));
    },
  );
  return crosses.every(function nonNegative(cross: number,): boolean {
    return cross >= (-EDGE_EPSILON);
  },)
    || crosses.every(function nonPositive(cross: number,): boolean {
      return cross <= EDGE_EPSILON;
    },);
}

/**
 * Splits fracture cells into hole and rim by centroid distance from the
 * impact, with per-cell radius jitter so the opening tears raggedly.
 * When no centroid falls inside the radius, whichever cell sits nearest
 * still flies, so a strike always removes glass.
 *
 * @param cells - fracture cells covering one pane
 *
 * @param impact - impact point in pane-local meters
 *
 * @param holeRadius - blast radius in meters around the impact
 *
 * @param random - uniform random source
 *
 * @mutates random - per-cell jitter draws advance the caller-supplied generator state.
 *
 * @returns hole and rim cell lists; together they cover every input cell
 *
 * @example
 * ```ts
 * const { hole, rim, } = partitionCellsByHole({
 *   cells,
 *   impact: { x: 0.1, y: -0.2 },
 *   holeRadius: 0.34,
 *   random: Math.random,
 * },);
 * ```
 */
export function partitionCellsByHole(
  {
    cells,
    impact,
    holeRadius,
    random,
  }: {
    readonly cells: readonly PaneCell[];
    readonly impact: PanePoint;
    readonly holeRadius: number;
    readonly random: RandomSource;
  },
): HolePartition {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: partitionCellsByHole.name,
    l,
  },);
  if (cells.length === 0)
    return {
      hole: [],
      rim: [],
    };
  /**
   * Cells annotated with centroid distance and their jittered escape
   * radius, deciding membership in one pass.
   */
  const measured = cells.map(function measureCell(cell: PaneCell,): {
    readonly cell: PaneCell;
    readonly distance: number;
    readonly escape: number;
  } {
    /**
     * Cell centroid standing in for its position.
     */
    const centroid = polygonCentroid(cell,);
    return {
      cell,
      distance: Math.hypot(
        centroid.x - impact.x,
        centroid.y - impact.y,
      ),
      escape: holeRadius
        * (1 + ((random() - (1
          / 2))
          * HOLE_JITTER
          * 2)),
    };
  },);
  /**
   * Cells inside their jittered escape radius, flying at strike time.
   */
  const flying = measured.filter(function insideHole(entry: {
    readonly distance: number;
    readonly escape: number;
  },): boolean {
    return entry.distance < entry.escape;
  },);
  /**
   * Nearest cell, drafted into the hole when the radius caught nothing,
   * so a strike always removes glass.
   */
  const nearest = measured.reduce(
    function closer(
      best: (typeof measured)[number],
      entry: (typeof measured)[number],
    ): (typeof measured)[number] {
      return entry.distance < best.distance
        ? entry
        : best;
    },
    nonNullishOrThrow(measured[0],),
  );
  /**
   * Final hole membership, never empty while cells exist.
   */
  const holeSet = new Set(
    (flying.length > 0 ? flying : [nearest,])
      .map(function unwrap(entry: { readonly cell: PaneCell; },): PaneCell {
        return entry.cell;
      },),
  );
  innerL.debug(`hole ${String(holeSet.size,)} cells, rim ${
    String(cells.length - holeSet.size,)
  } cells`,);
  return {
    hole: cells.filter(function inHole(cell: PaneCell,): boolean {
      return holeSet.has(cell,);
    },),
    rim: cells.filter(function inRim(cell: PaneCell,): boolean {
      return !holeSet.has(cell,);
    },),
  };
}
