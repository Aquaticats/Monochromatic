/**
 * Tests for the shard prism mesh generator.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PaneCell, } from './fracture.ts';
import { prismFromPolygon, } from './prism.ts';

/**
 * Right-triangle cell fixture with centroid at (1, 1).
 */
const TRIANGLE: PaneCell = [
  {
    x: 0,
    y: 0,
  },
  {
    x: 3,
    y: 0,
  },
  {
    x: 0,
    y: 3,
  },
];

/**
 * Pane thickness shared across prism tests, meters.
 */
const THICKNESS = 0.01;

/**
 * Prism built once from the triangle fixture; the generator is pure so
 * sharing the result across read-only assertions is safe.
 */
const TRIANGLE_PRISM = prismFromPolygon({
  polygon: TRIANGLE,
  thickness: THICKNESS,
},);

await describe({
  name: prismFromPolygon.name,
  children: [
    it({
      name: 'sizes buffers as 6 vertices and 2 faces plus walls per ring vertex',
      fn: async function sizesBuffers(): Promise<void> {
        expect(TRIANGLE_PRISM.positions,).toHaveLength(TRIANGLE.length * 6 * 3,);
        expect(TRIANGLE_PRISM.normals,).toHaveLength(TRIANGLE.length * 6 * 3,);
        expect(TRIANGLE_PRISM.indices,).toHaveLength(
          ((TRIANGLE.length - 2) * 3 * 2) + (TRIANGLE.length * 6),
        );
      },
    },),

    it({
      name: 'recenters the polygon around its centroid pivot',
      fn: async function recentersAroundPivot(): Promise<void> {
        expect(TRIANGLE_PRISM.pivot.x,).toBeCloseTo(1, 10,);
        expect(TRIANGLE_PRISM.pivot.y,).toBeCloseTo(1, 10,);
        /**
         * Mean x of the front ring, which recentering must place at 0.
         */
        const meanX = (TRIANGLE_PRISM.positions[0] as number)
          + (TRIANGLE_PRISM.positions[3] as number)
          + (TRIANGLE_PRISM.positions[6] as number);
        expect(meanX / 3,).toBeCloseTo(0, 10,);
      },
    },),

    it({
      name: 'extrudes symmetrically to half thickness on both sides',
      fn: async function extrudesSymmetrically(): Promise<void> {
        /**
         * Every z coordinate in the position buffer.
         */
        const zs = Array.from(
          { length: TRIANGLE_PRISM.positions.length / 3, },
          function zAt(_ignored: unknown, index: number,): number {
            return TRIANGLE_PRISM.positions[(index * 3) + 2] as number;
          },
        );
        // Float32Array storage rounds 0.005; assert at float32 precision.
        expect(Math.max(...zs,),).toBeCloseTo(THICKNESS / 2, 6,);
        expect(Math.min(...zs,),).toBeCloseTo((-THICKNESS) / 2, 6,);
      },
    },),

    it({
      name: 'keeps every index inside the vertex range',
      fn: async function indexesStayInRange(): Promise<void> {
        for (const index of TRIANGLE_PRISM.indices)
          expect(index,).toBeLessThan(TRIANGLE_PRISM.positions.length / 3,);
      },
    },),

    it({
      name: 'emits unit-length normals everywhere',
      fn: async function normalsAreUnit(): Promise<void> {
        for (let vertex = 0; vertex < (TRIANGLE_PRISM.normals.length / 3); vertex++)
          expect(
            Math.hypot(
              TRIANGLE_PRISM.normals[vertex * 3] as number,
              TRIANGLE_PRISM.normals[(vertex * 3) + 1] as number,
              TRIANGLE_PRISM.normals[(vertex * 3) + 2] as number,
            ),
          ).toBeCloseTo(1, 6,);
      },
    },),

    it({
      name: 'rejects degenerate polygons',
      fn: async function rejectsDegenerate(): Promise<void> {
        expect(function buildDegenerate(): void {
          prismFromPolygon({
            polygon: [
              {
                x: 0,
                y: 0,
              },
              {
                x: 1,
                y: 0,
              },
            ],
            thickness: THICKNESS,
          },);
        },).toThrow(RangeError,);
      },
    },),
  ],
},);
