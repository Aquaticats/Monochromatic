/**
 * Impact-centered Voronoi fracture math for rectangular glass panes.
 *
 * Real annealed glass breaks into a spider web: radial spokes from the
 * impact point crossed by concentric rings, with cell size growing with
 * distance from the impact. This module reproduces that pattern by placing
 * Voronoi seeds on jittered polar rings around the impact and clipping
 * each seed's region against the perpendicular bisectors of every other
 * seed (Sutherland-Hodgman against half-planes).
 *
 * Pure math on plain number pairs: no three.js types, so the whole module
 * is unit-testable in node.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

/**
 * Logger root shared by the glass demo client modules.
 */
const parentLogger = tagged({ tag: 'web-demo-glass', },);

/**
 * Tagged logger for the fracture module.
 */
const l = tagged({
  tag: 'fracture',
  l: parentLogger,
},);

/**
 * Point in pane-local 2D space, meters, origin at pane center.
 */
export type PanePoint = {
  readonly x: number;
  readonly y: number;
};

/**
 * Convex polygon in pane-local 2D space, counterclockwise winding.
 */
export type PaneCell = readonly PanePoint[];

/**
 * Uniform random source in [0, 1). Injected so tests run deterministic
 * sequences and callers control seeding.
 */
export type RandomSource = () => number;

/**
 * Fewest vertices a surviving cell may have; anything smaller is a sliver.
 */
const MIN_CELL_VERTICES = 3;

/**
 * Tuning constants for {@link fractureCells}. Grouped as one object so the
 * numbers read as a table rather than scattered magic values.
 */
export const FRACTURE_TUNING = {
  /**
   * Fewest radial spokes any web gets; below this the web reads as a star.
   */
  spokeCountMin: 12,
  /**
   * Random extra spokes on top of {@link FRACTURE_TUNING.spokeCountMin}.
   */
  spokeCountExtra: 5,
  /**
   * Innermost ring radius in meters; sets the pulverized-center scale.
   */
  firstRingRadius: 0.055,
  /**
   * Smallest ring-to-ring growth factor; rings spread geometrically.
   */
  ringGrowthMin: 1.55,
  /**
   * Random extra growth on top of {@link FRACTURE_TUNING.ringGrowthMin}.
   */
  ringGrowthExtra: 0.35,
  /**
   * Angular jitter as a fraction of the spoke spacing.
   */
  angularJitter: 0.22,
  /**
   * Radial jitter as a fraction of each ring radius.
   */
  radialJitter: 0.12,
  /**
   * Probability of dropping a seed from the outermost two rings.
   */
  outerDropout: 0.4,
  /**
   * Cells smaller than this area in square meters are discarded as slivers.
   */
  minCellArea: 0.0001,
} as const;

/**
 * Signed doubled area accumulator over polygon edges (shoelace formula).
 *
 * @param polygon - closed convex polygon, consecutive vertices
 *
 * @returns absolute enclosed area in square meters
 *
 * @example
 * ```ts
 * polygonArea([{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }, { x: 0, y: 1 }]);
 * // 1
 * ```
 */
export function polygonArea(polygon: PaneCell,): number {
  /**
   * Doubled signed area summed edge by edge; sign encodes winding.
   */
  const doubled = polygon.reduce(
    function accumulateEdge(
      sum: number,
      vertex: PanePoint,
      index: number,
    ): number {
      /**
       * Edge partner: next vertex, wrapping to close the polygon.
       */
      const next = nonNullishOrThrow(polygon[(index + 1) % polygon.length],);
      return (sum + (vertex.x
        * next.y)) - (next.x * vertex.y);
    },
    0,
  );
  return Math.abs(doubled,) / 2;
}

/**
 * Arithmetic mean of polygon vertices. Exact centroids matter little here:
 * cells are small and convex, so the vertex mean stays inside and is cheap.
 *
 * @param polygon - convex polygon with at least one vertex
 *
 * @returns interior point used as shard pivot
 *
 * @example
 * ```ts
 * polygonCentroid([{ x: 0, y: 0 }, { x: 2, y: 0 }, { x: 1, y: 3 }]);
 * // { x: 1, y: 1 }
 * ```
 */
export function polygonCentroid(polygon: PaneCell,): PanePoint {
  /**
   * Component-wise vertex sum feeding the mean.
   */
  const total = polygon.reduce(
    function accumulateVertex(
      sum: PanePoint,
      vertex: PanePoint,
    ): PanePoint {
      return {
        x: sum.x + vertex.x,
        y: sum.y + vertex.y,
      };
    },
    {
      x: 0,
      y: 0,
    },
  );
  return {
    x: total.x / polygon.length,
    y: total.y / polygon.length,
  };
}

/**
 * Clips a convex polygon against one half-plane, keeping points where
 * `dot(p, normal) <= offset`. One Sutherland-Hodgman pass; the result of
 * clipping a convex polygon stays convex.
 *
 * @param polygon - convex polygon to clip
 *
 * @param normalX - half-plane normal x component; need not be unit length
 *
 * @param normalY - half-plane normal y component
 *
 * @param offset - half-plane boundary offset along its normal
 *
 * @returns clipped polygon; empty when fully outside
 *
 * @example
 * ```ts
 * clipConvexPolygon({
 *   polygon: [{ x: -1, y: -1 }, { x: 1, y: -1 }, { x: 1, y: 1 }, { x: -1, y: 1 }],
 *   normalX: 1,
 *   normalY: 0,
 *   offset: 0,
 * },);
 * // left half of the square
 * ```
 */
export function clipConvexPolygon(
  {
    polygon,
    normalX,
    normalY,
    offset,
  }: Readonly<{
    polygon: PaneCell;
    normalX: number;
    normalY: number;
    offset: number;
  }>,
): PaneCell {
  /**
   * Vertices surviving the clip plus crossing intersections, built edge by edge.
   */
  const kept: PanePoint[] = [];
  for (const [index, vertex,] of polygon.entries()) {
    /**
     * Edge partner: next vertex, wrapping to close the polygon.
     */
    const next = nonNullishOrThrow(polygon[(index + 1) % polygon.length],);
    /**
     * Signed distance of this vertex from the boundary; non-positive is inside.
     */
    const hereDistance = ((vertex.x * normalX) + (vertex.y * normalY)) - offset;
    /**
     * Signed distance of the edge partner from the boundary.
     */
    const nextDistance = ((next.x * normalX) + (next.y * normalY)) - offset;
    if (hereDistance <= 0)
      kept.push(vertex,);
    if ((hereDistance <= 0) !== (nextDistance <= 0)) {
      /**
       * Interpolation parameter where this edge crosses the boundary.
       */
      const crossing = hereDistance / (hereDistance - nextDistance);
      kept.push({
        x: vertex.x + ((next.x - vertex.x)
          * crossing),
        y: vertex.y + ((next.y - vertex.y)
          * crossing),
      },);
    }
  }
  return kept;
}

/**
 * Places Voronoi seeds on jittered polar rings centered at the impact:
 * shared spoke angles across rings give radial crack lines, geometric ring
 * growth gives concentric bands whose cells enlarge with distance.
 * Seeds outside the pane rectangle are dropped, which lets edge cells run
 * out to the pane border, matching how a web near an edge opens up.
 *
 * @param halfWidth - pane half width in meters
 *
 * @param halfHeight - pane half height in meters
 *
 * @param impact - impact point in pane-local coordinates
 *
 * @param random - uniform random source, injected for determinism
 *
 * @mutates random - every draw advances the caller-supplied generator state.
 *
 * @returns seeds inside the pane, impact point first
 *
 * @example
 * ```ts
 * const seeds = radialFractureSeeds({
 *   halfWidth: 1,
 *   halfHeight: 1.3,
 *   impact: { x: 0.1, y: -0.2 },
 *   random: Math.random,
 * },);
 * ```
 */
export function radialFractureSeeds(
  {
    halfWidth,
    halfHeight,
    impact,
    random,
  }: {
    readonly halfWidth: number;
    readonly halfHeight: number;
    readonly impact: PanePoint;
    readonly random: RandomSource;
  },
): PanePoint[] {
  /**
   * Spoke count for this web; fixed per fracture so rings align radially.
   */
  const spokeCount = FRACTURE_TUNING.spokeCountMin
    + Math.floor(random() * FRACTURE_TUNING.spokeCountExtra,);
  /**
   * Base angle per spoke, jittered once so the web is irregular but the
   * same spoke stays aligned across every ring.
   */
  const spokeAngles = Array.from(
    { length: spokeCount, },
    function spokeAngle(
      _ignored: unknown,
      index: number,
    ): number {
      return (index + ((random() - (1
        / 2))
        * FRACTURE_TUNING.angularJitter
        * 2))
        * ((Math.PI * 2) / spokeCount);
    },
  );
  /**
   * Farthest pane corner distance from the impact; rings beyond it cannot
   * contribute cells, so seeding stops there.
   */
  const maxReach = Math.hypot(
    halfWidth + Math.abs(impact.x,),
    halfHeight + Math.abs(impact.y,),
  );
  /**
   * Ring radii growing geometrically from the pulverized center outward.
   */
  const ringRadii: number[] = [];
  for (
    let radius = FRACTURE_TUNING.firstRingRadius;
    radius < maxReach;
    radius *= FRACTURE_TUNING.ringGrowthMin + (random()
      * FRACTURE_TUNING.ringGrowthExtra)
  )
    ringRadii.push(radius,);
  /**
   * Seeds accumulated ring by ring; the impact itself seeds the center cell.
   */
  const seeds: PanePoint[] = [{ ...impact, },];
  for (const [ringIndex, ringRadius,] of ringRadii.entries()) {
    /**
     * Whether this ring belongs to the outer two, which thin out so far
     * cells grow even larger than the geometric growth alone gives.
     */
    const outer = ringIndex >= (ringRadii.length
      - 2);
    for (const angle of spokeAngles) {
      if (outer && (random() < FRACTURE_TUNING.outerDropout))
        continue;
      /**
       * Ring radius with per-seed jitter so rings read as cracks, not circles.
       */
      const jitteredRadius = ringRadius
        * (1 + ((random() - (1
          / 2))
          * FRACTURE_TUNING.radialJitter
          * 2));
      /**
       * Candidate seed position before the pane-bounds check.
       */
      const candidate = {
        x: impact.x + (Math.cos(angle,)
          * jitteredRadius),
        y: impact.y + (Math.sin(angle,)
          * jitteredRadius),
      };
      if ((Math.abs(candidate.x,) < halfWidth) && (Math.abs(candidate.y,) < halfHeight))
        seeds.push(candidate,);
    }
  }
  return seeds;
}

/**
 * Computes the full fracture: seeds via {@link radialFractureSeeds}, then
 * one Voronoi cell per seed by clipping the pane rectangle against the
 * perpendicular bisector toward every other seed. Quadratic in seed count,
 * which stays acceptable because fracture runs once per shatter, not per
 * frame.
 *
 * @param halfWidth - pane half width in meters
 *
 * @param halfHeight - pane half height in meters
 *
 * @param impact - impact point in pane-local coordinates
 *
 * @param random - uniform random source, injected for determinism
 *
 * @mutates random - seed generation draws advance the caller-supplied generator state.
 *
 * @returns convex cells covering the pane, slivers dropped
 *
 * @example
 * ```ts
 * const cells = fractureCells({
 *   halfWidth: 0.9,
 *   halfHeight: 1.25,
 *   impact: { x: 0, y: 0 },
 *   random: Math.random,
 * },);
 * ```
 */
export function fractureCells(
  {
    halfWidth,
    halfHeight,
    impact,
    random,
  }: {
    readonly halfWidth: number;
    readonly halfHeight: number;
    readonly impact: PanePoint;
    readonly random: RandomSource;
  },
): PaneCell[] {
  /**
   * Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: fractureCells.name,
    l,
  },);
  /**
   * Voronoi sites for this fracture, impact-centered.
   */
  const seeds = radialFractureSeeds({
    halfWidth,
    halfHeight,
    impact,
    random,
  },);
  /**
   * Pane rectangle every cell starts from before bisector clipping.
   */
  const paneRect: PaneCell = [
    {
      x: -halfWidth,
      y: -halfHeight,
    },
    {
      x: halfWidth,
      y: -halfHeight,
    },
    {
      x: halfWidth,
      y: halfHeight,
    },
    {
      x: -halfWidth,
      y: halfHeight,
    },
  ];
  /**
   * Finished cells, one per surviving seed.
   */
  const cells = seeds
    .map(function cellForSeed(seed: PanePoint,): PaneCell {
      // Region owned by this seed, shrunk bisector by bisector.
      return seeds.reduce(
        function clipAgainstBisector(
          region: PaneCell,
          other: PanePoint,
        ): PaneCell {
          if ((other === seed) || (region.length === 0))
            return region;
          return clipConvexPolygon({
            polygon: region,
            normalX: other.x - seed.x,
            normalY: other.y - seed.y,
            offset: (((other.x * other.x) + (other.y * other.y))
              - (seed.x * seed.x)
              - (seed.y * seed.y)) / 2,
          },);
        },
        paneRect,
      );
    },)
    .filter(function keepSubstantial(cell: PaneCell,): boolean {
      return (cell.length >= MIN_CELL_VERTICES) && (polygonArea(cell,) > FRACTURE_TUNING.minCellArea);
    },);
  innerL.debug(`fractured pane into ${String(cells.length,)} cells from ${
    String(seeds.length,)
  } seeds`,);
  return cells;
}
