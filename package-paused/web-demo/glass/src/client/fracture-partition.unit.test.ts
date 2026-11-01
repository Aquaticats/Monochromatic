/**
 * Tests for the hole/rim cell partition.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  PaneCell,
  PanePoint,
} from './fracture.ts';
import {
  partitionCellsByHole,
  pointInConvexPolygon,
} from './fracture-partition.ts';

/**
 * Constant random source pinning every draw to the midpoint, which zeroes
 * the symmetric hole-radius jitter.
 *
 * @returns 0.5 on every draw
 *
 * @example
 * ```ts
 * partitionCellsByHole({ ...input, random: midRandom },);
 * ```
 */
function midRandom(): number {
  return 1 / 2;
}

/**
 * Half edge length of each grid cell square.
 */
const CELL_HALF = 0.1;

/**
 * Builds one square cell centered at a point, counterclockwise winding.
 *
 * @param center - square center in pane-local meters
 *
 * @returns square cell polygon
 *
 * @example
 * ```ts
 * const cell = squareCellAt({ x: 0.2, y: 0 },);
 * ```
 */
function squareCellAt(center: PanePoint,): PaneCell {
  return [
    {
      x: center.x - CELL_HALF,
      y: center.y - CELL_HALF,
    },
    {
      x: center.x + CELL_HALF,
      y: center.y - CELL_HALF,
    },
    {
      x: center.x + CELL_HALF,
      y: center.y + CELL_HALF,
    },
    {
      x: center.x - CELL_HALF,
      y: center.y + CELL_HALF,
    },
  ];
}

/**
 * Grid of square cells at x in {-0.4..0.4}, y in {-0.4..0.4}, step 0.4,
 * standing in for a fractured pane.
 */
const GRID_CELLS: readonly PaneCell[] = [
  -0.4,
  0,
  0.4,
].flatMap(function rowAt(y: number,): PaneCell[] {
  return [
    -0.4,
    0,
    0.4,
  ].map(function cellAt(x: number,): PaneCell {
    return squareCellAt({
      x,
      y,
    },);
  },);
},);

await describe({
  name: pointInConvexPolygon.name,
  children: [
    it({
      name: 'accepts interior points and rejects exterior points',
      fn: async function classifiesPoints(): Promise<void> {
        /**
         * Unit-ish triangle under test.
         */
        const triangle: PaneCell = [
          {
            x: 0,
            y: 0,
          },
          {
            x: 1,
            y: 0,
          },
          {
            x: 0,
            y: 1,
          },
        ];
        expect(pointInConvexPolygon({
          polygon: triangle,
          point: {
            x: 0.2,
            y: 0.2,
          },
        },),).toBe(true,);
        expect(pointInConvexPolygon({
          polygon: triangle,
          point: {
            x: 0.8,
            y: 0.8,
          },
        },),).toBe(false,);
      },
    },),

    it({
      name: 'accepts on-edge points under either winding',
      fn: async function acceptsEdgesBothWindings(): Promise<void> {
        /**
         * Square in counterclockwise winding.
         */
        const square = squareCellAt({
          x: 0,
          y: 0,
        },);
        /**
         * Same square with the winding reversed.
         */
        const reversed = square.toReversed();
        /**
         * Point exactly on one square edge.
         */
        const onEdge = {
          x: CELL_HALF,
          y: 0,
        };
        expect(pointInConvexPolygon({
          polygon: square,
          point: onEdge,
        },),).toBe(true,);
        expect(pointInConvexPolygon({
          polygon: reversed,
          point: onEdge,
        },),).toBe(true,);
        expect(pointInConvexPolygon({
          polygon: reversed,
          point: {
            x: 1,
            y: 1,
          },
        },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: partitionCellsByHole.name,
  children: [
    it({
      name: 'covers every cell exactly once across hole and rim',
      fn: async function coversAllCells(): Promise<void> {
        /**
         * Partition around a center impact with a mid-grid radius.
         */
        const { hole, rim, } = partitionCellsByHole({
          cells: GRID_CELLS,
          impact: {
            x: 0,
            y: 0,
          },
          holeRadius: 0.5,
          random: midRandom,
        },);
        expect(hole.length + rim.length,).toBe(GRID_CELLS.length,);
        for (const cell of GRID_CELLS)
          expect(hole.includes(cell,) !== rim.includes(cell,),).toBe(true,);
      },
    },),

    it({
      name: 'keeps near cells in the hole and far cells in the rim',
      fn: async function splitsByDistance(): Promise<void> {
        /**
         * Partition catching the center cell and the four edge-adjacent
         * cells (distance 0.4) but not the corner cells (distance ~0.57).
         */
        const { hole, rim, } = partitionCellsByHole({
          cells: GRID_CELLS,
          impact: {
            x: 0,
            y: 0,
          },
          holeRadius: 0.5,
          random: midRandom,
        },);
        expect(hole.length,).toBe(5,);
        expect(rim.length,).toBe(4,);
      },
    },),

    it({
      name: 'always drafts the nearest cell when the radius catches none',
      fn: async function alwaysRemovesGlass(): Promise<void> {
        /**
         * Partition with a radius far smaller than any centroid distance
         * to an off-center impact.
         */
        const { hole, rim, } = partitionCellsByHole({
          cells: GRID_CELLS,
          impact: {
            x: 0.32,
            y: 0.05,
          },
          holeRadius: 0.001,
          random: midRandom,
        },);
        expect(hole.length,).toBe(1,);
        expect(rim.length,).toBe(GRID_CELLS.length - 1,);
        /**
         * Drafted cell; nearest centroid to the impact is (0.4, 0).
         */
        const [drafted,] = hole;
        expect(drafted,).toBe(GRID_CELLS[5],);
      },
    },),
  ],
},);
