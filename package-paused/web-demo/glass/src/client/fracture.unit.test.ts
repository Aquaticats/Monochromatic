/**
 * Tests for the impact-centered Voronoi fracture math.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  clipConvexPolygon,
  fractureCells,
  type PaneCell,
  polygonArea,
  polygonCentroid,
  radialFractureSeeds,
  type RandomSource,
} from './fracture.ts';

//region Fixtures and deterministic randomness

/**
 * Deterministic linear congruential generator so fracture tests replay the
 * same random sequence on every run.
 *
 * @param seed - starting state; equal seeds give equal sequences
 *
 * @returns uniform source in [0, 1)
 *
 * @example
 * ```ts
 * const random = deterministicRandom(7);
 * ```
 */
function deterministicRandom(seed: number,): RandomSource {
  /**
   * Generator state advanced on every draw; object-wrapped so the closure
   * mutates a property instead of a root binding.
   */
  const state = { value: Math.trunc(Math.abs(seed,),) % (2 ** 32), };
  return function draw(): number {
    state.value = ((state.value * 1_664_525) + 1_013_904_223) % (2 ** 32);
    return state.value / (2 ** 32);
  };
}

/**
 * Unit square fixture reused across area and clip tests.
 */
const UNIT_SQUARE: PaneCell = [
  {
    x: 0,
    y: 0,
  },
  {
    x: 1,
    y: 0,
  },
  {
    x: 1,
    y: 1,
  },
  {
    x: 0,
    y: 1,
  },
];

/**
 * Shared geometry for seed and cell tests: a door-sized pane hit
 * off-center.
 */
const PANE_INPUT = {
  halfWidth: 0.9,
  halfHeight: 1.25,
  impact: {
    x: 0.2,
    y: -0.3,
  },
} as const;

/**
 * Distance from a cell's centroid to the shared impact point.
 *
 * @param cell - fracture cell to measure
 *
 * @returns centroid distance in meters
 *
 * @example
 * ```ts
 * const distance = centroidDistance([{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 0, y: 1 }]);
 * ```
 */
function centroidDistance(cell: PaneCell,): number {
  /**
   * Cell centroid reused for the distance measure.
   */
  const centroid = polygonCentroid(cell,);
  return Math.hypot(
    centroid.x - PANE_INPUT.impact.x,
    centroid.y - PANE_INPUT.impact.y,
  );
}

/**
 * Mean polygon area across a cell group.
 *
 * @param group - cells to average over
 *
 * @returns mean area in square meters
 *
 * @example
 * ```ts
 * const mean = meanArea([[{ x: 0, y: 0 }, { x: 1, y: 0 }, { x: 0, y: 1 }]]);
 * ```
 */
function meanArea(group: readonly PaneCell[],): number {
  return group.reduce(
    function accumulateArea(sum: number, cell: PaneCell,): number {
      return sum + polygonArea(cell,);
    },
    0,
  ) / group.length;
}

//endregion Fixtures and deterministic randomness

await describe({
  name: polygonArea.name,
  children: [
    it({
      name: 'measures the unit square as 1',
      fn: async function measuresUnitSquare(): Promise<void> {
        expect(polygonArea(UNIT_SQUARE,),).toBeCloseTo(1, 10,);
      },
    },),

    it({
      name: 'measures a right triangle as half its bounding box',
      fn: async function measuresTriangle(): Promise<void> {
        expect(
          polygonArea([
            {
              x: 0,
              y: 0,
            },
            {
              x: 2,
              y: 0,
            },
            {
              x: 0,
              y: 3,
            },
          ],),
        ).toBeCloseTo(3, 10,);
      },
    },),

    it({
      name: 'ignores winding direction',
      fn: async function ignoresWinding(): Promise<void> {
        expect(
          polygonArea([...UNIT_SQUARE,].toReversed(),),
        ).toBeCloseTo(1, 10,);
      },
    },),
  ],
},);

await describe({
  name: polygonCentroid.name,
  children: [
    it({
      name: 'returns the vertex mean',
      fn: async function returnsVertexMean(): Promise<void> {
        /**
         * Centroid of the unit square fixture.
         */
        const centroid = polygonCentroid(UNIT_SQUARE,);
        expect(centroid.x,).toBeCloseTo(1 / 2, 10,);
        expect(centroid.y,).toBeCloseTo(1 / 2, 10,);
      },
    },),
  ],
},);

await describe({
  name: clipConvexPolygon.name,
  children: [
    it({
      name: 'keeps the inside half when the boundary bisects the polygon',
      fn: async function keepsInsideHalf(): Promise<void> {
        /**
         * Left half of the unit square: keep x <= 0.5.
         */
        const half = clipConvexPolygon({
          polygon: UNIT_SQUARE,
          normalX: 1,
          normalY: 0,
          offset: 1 / 2,
        },);
        expect(polygonArea(half,),).toBeCloseTo(1 / 2, 10,);
        for (const vertex of half)
          expect(vertex.x,).toBeLessThanOrEqual((1 / 2) + 1e-12,);
      },
    },),

    it({
      name: 'returns the polygon unchanged when fully inside',
      fn: async function keepsFullyInside(): Promise<void> {
        expect(
          polygonArea(clipConvexPolygon({
            polygon: UNIT_SQUARE,
            normalX: 1,
            normalY: 0,
            offset: 2,
          },),),
        ).toBeCloseTo(1, 10,);
      },
    },),

    it({
      name: 'returns an empty polygon when fully outside',
      fn: async function dropsFullyOutside(): Promise<void> {
        expect(clipConvexPolygon({
          polygon: UNIT_SQUARE,
          normalX: 1,
          normalY: 0,
          offset: -1,
        },),).toHaveLength(0,);
      },
    },),
  ],
},);

await describe({
  name: radialFractureSeeds.name,
  children: [
    it({
      name: 'places the impact point as the first seed',
      fn: async function impactSeedsFirst(): Promise<void> {
        /**
         * Seeds from a fixed random sequence.
         */
        const seeds = radialFractureSeeds({
          ...PANE_INPUT,
          random: deterministicRandom(1,),
        },);
        expect(seeds[0]?.x,).toBeCloseTo(PANE_INPUT.impact.x, 10,);
        expect(seeds[0]?.y,).toBeCloseTo(PANE_INPUT.impact.y, 10,);
      },
    },),

    it({
      name: 'keeps every seed inside the pane rectangle',
      fn: async function seedsStayInside(): Promise<void> {
        /**
         * Seeds from a fixed random sequence.
         */
        const seeds = radialFractureSeeds({
          ...PANE_INPUT,
          random: deterministicRandom(2,),
        },);
        expect(seeds.length,).toBeGreaterThan(10,);
        for (const seed of seeds) {
          expect(Math.abs(seed.x,),).toBeLessThanOrEqual(PANE_INPUT.halfWidth,);
          expect(Math.abs(seed.y,),).toBeLessThanOrEqual(PANE_INPUT.halfHeight,);
        }
      },
    },),

    it({
      name: 'packs seeds denser near the impact than far away',
      fn: async function seedsDenserNearImpact(): Promise<void> {
        /**
         * Seeds from a fixed random sequence.
         */
        const seeds = radialFractureSeeds({
          ...PANE_INPUT,
          random: deterministicRandom(3,),
        },);
        /**
         * Seeds within 0.3 m of the impact.
         */
        const near = seeds.filter(function isNear(seed,): boolean {
          return Math.hypot(seed.x - PANE_INPUT.impact.x, seed.y - PANE_INPUT.impact.y,)
            < 0.3;
        },);
        /**
         * Seeds beyond 0.8 m from the impact.
         */
        const far = seeds.filter(function isFar(seed,): boolean {
          return Math.hypot(seed.x - PANE_INPUT.impact.x, seed.y - PANE_INPUT.impact.y,)
            > 0.8;
        },);
        /**
         * Near-region area for the density comparison, square meters.
         */
        const nearArea = Math.PI * (0.3 ** 2);
        /**
         * Whole-pane area: generous upper bound for the far region, which
         * only weakens the assertion.
         */
        const farArea = 4 * PANE_INPUT.halfWidth * PANE_INPUT.halfHeight;
        expect(near.length / nearArea,).toBeGreaterThan(far.length / farArea,);
      },
    },),
  ],
},);

await describe({
  name: fractureCells.name,
  children: [
    it({
      name: 'partitions the pane: cell areas sum to the pane area',
      fn: async function partitionsPane(): Promise<void> {
        /**
         * Cells from a fixed random sequence.
         */
        const cells = fractureCells({
          ...PANE_INPUT,
          random: deterministicRandom(4,),
        },);
        /**
         * Total area across every cell.
         */
        const total = cells.reduce(
          function accumulateArea(sum: number, cell: PaneCell,): number {
            return sum + polygonArea(cell,);
          },
          0,
        );
        /**
         * Exact pane area the partition must reproduce.
         */
        const paneArea = 4 * PANE_INPUT.halfWidth * PANE_INPUT.halfHeight;
        expect(total,).toBeGreaterThan(paneArea * 0.98,);
        expect(total,).toBeLessThanOrEqual(paneArea * 1.0001,);
      },
    },),

    it({
      name: 'keeps every cell vertex inside the pane rectangle',
      fn: async function cellsStayInside(): Promise<void> {
        /**
         * Cells from a fixed random sequence.
         */
        const cells = fractureCells({
          ...PANE_INPUT,
          random: deterministicRandom(5,),
        },);
        expect(cells.length,).toBeGreaterThan(10,);
        for (const cell of cells)
          for (const vertex of cell) {
            expect(Math.abs(vertex.x,),).toBeLessThanOrEqual(PANE_INPUT.halfWidth + 1e-9,);
            expect(Math.abs(vertex.y,),).toBeLessThanOrEqual(
              PANE_INPUT.halfHeight + 1e-9,
            );
          }
      },
    },),

    it({
      name: 'makes cells near the impact smaller than far cells',
      fn: async function nearCellsSmaller(): Promise<void> {
        /**
         * Cells from a fixed random sequence.
         */
        const cells = fractureCells({
          ...PANE_INPUT,
          random: deterministicRandom(6,),
        },);
        /**
         * Mean area of cells whose centroid lies within 0.25 m of the impact.
         */
        const nearMean = meanArea(cells.filter(function isNear(cell,): boolean {
          return centroidDistance(cell,) < 0.25;
        },),);
        /**
         * Mean area of cells whose centroid lies beyond 0.7 m from the impact.
         */
        const farMean = meanArea(cells.filter(function isFar(cell,): boolean {
          return centroidDistance(cell,) > 0.7;
        },),);
        expect(nearMean,).toBeLessThan(farMean,);
      },
    },),

    it({
      name: 'replays identically for an equal random sequence',
      fn: async function replaysDeterministically(): Promise<void> {
        expect(fractureCells({
          ...PANE_INPUT,
          random: deterministicRandom(7,),
        },),).toEqual(fractureCells({
          ...PANE_INPUT,
          random: deterministicRandom(7,),
        },),);
      },
    },),
  ],
},);
