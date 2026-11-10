/**
 Stroke expansion refuses geometry it cannot offset.

 `offsetPolygon` intersects consecutive offset edges, so two collinear edges have
 no meeting point and a duplicated vertex has no edge direction at all. Both
 cases divided by zero, turned a vertex into `NaN` or `Infinity`, and wrote the
 corrupted ring into the shipped font with no diagnostic.
 */
/* oxlint-disable test-import/require-eventual-artifact -- this package ships src/ (package.json `files`) and builds font binaries rather than a JS artifact, so the modules beside this test are the shipped ones; see doc/planning/oxlint-test-import-eventual-artifact.md, section "Packages that ship source". */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { offsetPolygon, } from './expand-stroke.ts';

await describe({
  name: '',
  children: [
    describe({
      name: offsetPolygon.name,
      children: [
        it({
          name: 'offsets a square outward by the requested distance',
          fn: async () => {
            /**
             Offset the expansion moves every edge by.
             */
            const offset = 2;

            expect(offsetPolygon({
              vertices: [
                [0, 0,],
                [10, 0,],
                [10, 10,],
                [0, 10,],
              ],
              offset,
            },),).toStrictEqual([
              [10 + offset, -offset,],
              [10 + offset, 10 + offset,],
              [-offset, 10 + offset,],
              [-offset, -offset,],
            ],);
          },
        },),

        it({
          name: 'throws when two consecutive edges are collinear',
          fn: async () => {
            expect(function callExpand(): void {
              offsetPolygon({
                vertices: [
                  [0, 0,],
                  [10, 0,],
                  [20, 0,],
                  [20, 10,],
                ],
                offset: 2,
              },);
            },).toThrow('collinear',);
          },
        },),

        it({
          name: 'throws when two consecutive vertices coincide',
          fn: async () => {
            expect(function callExpand(): void {
              offsetPolygon({
                vertices: [
                  [0, 0,],
                  [0, 0,],
                  [10, 0,],
                  [10, 10,],
                ],
                offset: 2,
              },);
            },).toThrow('zero-length edge',);
          },
        },),
      ],
    },),
  ],
},);
