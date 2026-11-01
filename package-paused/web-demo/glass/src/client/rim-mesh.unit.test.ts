/**
 * Tests for the merged rim mesh arrays.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type {
  PaneCell,
  PanePoint,
} from './fracture.ts';
import { rimMeshArrays, } from './rim-mesh.ts';

/**
 * Pane thickness every test extrudes with.
 */
const THICKNESS = 0.012;

/**
 * Vertices one triangle prism carries: front ring, back ring, and four
 * per side wall.
 */
const TRIANGLE_PRISM_VERTICES = 18;

/**
 * Indices one triangle prism carries: two fan triangles plus two per wall.
 */
const TRIANGLE_PRISM_INDICES = 24;

/**
 * Two triangle cells at distinct pane positions, exercising the merge
 * offsets.
 */
const TWO_TRIANGLES: readonly PaneCell[] = [
  [
    {
      x: 0.2,
      y: 0.3,
    },
    {
      x: 0.3,
      y: 0.3,
    },
    {
      x: 0.25,
      y: 0.42,
    },
  ],
  [
    {
      x: -0.5,
      y: -0.1,
    },
    {
      x: -0.38,
      y: -0.1,
    },
    {
      x: -0.44,
      y: -0.22,
    },
  ],
];

await describe({
  name: rimMeshArrays.name,
  children: [
    it({
      name: 'merges buffer sizes as the sum of per-cell prisms',
      fn: async function mergesBufferSizes(): Promise<void> {
        /**
         * Merged arrays for both triangles.
         */
        const arrays = rimMeshArrays({
          cells: TWO_TRIANGLES,
          thickness: THICKNESS,
        },);
        expect(arrays.positions
          .length,).toBe(TRIANGLE_PRISM_VERTICES * 3
          * 2,);
        expect(arrays.normals
          .length,).toBe(arrays.positions
          .length,);
        expect(arrays.indices
          .length,).toBe(TRIANGLE_PRISM_INDICES * 2,);
      },
    },),

    it({
      name: 'restores each cell to its pane-local position',
      fn: async function restoresPanePositions(): Promise<void> {
        /**
         * Merged arrays for both triangles.
         */
        const arrays = rimMeshArrays({
          cells: TWO_TRIANGLES,
          thickness: THICKNESS,
        },);
        /**
         * First vertex of the second prism inside the merged buffer.
         */
        const secondBase = TRIANGLE_PRISM_VERTICES * 3;
        /**
         * Second cell's first polygon vertex, the merge's expected output.
         */
        const [expected,] = nonNullishOrThrow(TWO_TRIANGLES[1],);
        expect(nonNullishOrThrow(arrays.positions[0],),).toBeCloseTo(
          0.2,
          6,
        );
        expect(nonNullishOrThrow(arrays.positions[1],),).toBeCloseTo(
          0.3,
          6,
        );
        expect(nonNullishOrThrow(arrays.positions[secondBase],),).toBeCloseTo(
          nonNullishOrThrow(expected,).x,
          6,
        );
        expect(nonNullishOrThrow(arrays.positions[secondBase + 1],),)
          .toBeCloseTo(
            nonNullishOrThrow(expected,).y,
            6,
          );
        expect(nonNullishOrThrow(arrays.positions[secondBase + 2],),)
          .toBeCloseTo(
            THICKNESS / 2,
            6,
          );
      },
    },),

    it({
      name: 'offsets second-cell indices past the first cell block',
      fn: async function offsetsIndices(): Promise<void> {
        /**
         * Merged arrays for both triangles.
         */
        const arrays = rimMeshArrays({
          cells: TWO_TRIANGLES,
          thickness: THICKNESS,
        },);
        for (
          const index of arrays.indices
            .slice(TRIANGLE_PRISM_INDICES,)
        ) {
          expect(index,).toBeGreaterThan(TRIANGLE_PRISM_VERTICES - 1,);
          expect(index,).toBeLessThan(TRIANGLE_PRISM_VERTICES * 2,);
        }
      },
    },),

    it({
      name: 'rejects rims beyond Uint16 vertex indexing',
      fn: async function rejectsOversizedRims(): Promise<void> {
        /**
         * Vertex count forcing the merged total past 65535.
         */
        const ringSize = 11_000;
        /**
         * One enormous polygon whose prism alone exceeds Uint16 indexing.
         */
        const oversized: PaneCell = Array.from(
          { length: ringSize, },
          function circleVertex(
            _ignored: unknown,
            index: number,
          ): PanePoint {
            /**
             * Angle of this ring vertex.
             */
            const angle = (index / ringSize) * Math.PI
              * 2;
            return {
              x: Math.cos(angle,),
              y: Math.sin(angle,),
            };
          },
        );
        expect(function buildOversized(): void {
          rimMeshArrays({
            cells: [oversized,],
            thickness: THICKNESS,
          },);
        },).toThrow(RangeError,);
      },
    },),
  ],
},);
