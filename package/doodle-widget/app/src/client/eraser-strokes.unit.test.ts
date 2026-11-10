/**
 Tests for stroke erasure, and for the iteration shape the eraser walks points with.

 The behavioral suites pin segment splitting and the fast-drag segment check so
 the iteration shape those cases accompany cannot quietly change what the eraser
 erases. The last suite reads the module's own source, because the review
 standard's `for...of` requirement is otherwise invisible to execution: a classic
 index loop behaves identically, so only a source assertion can hold the shape the
 standard asks for.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  eraseStrokesAt,
  getStrokes,
  getStrokeWidth,
  type NormalizedPoint,
  setStrokes,
  type StrokeData,
} from '../../dist/final/node/index.mjs';

//region Constants

/**
 Canvas edge in CSS pixels the cases draw on.

 Square, so pixel and normalized coordinates share one divisor.
 */
const CANVAS_PX = 1_000;

/**
 Eraser radius in CSS pixels, which the module takes from the active stroke width.
 */
const ERASER_RADIUS_PX = getStrokeWidth();

/**
 Distance between adjacent stroke points in the splitting and crossing cases.

 Ten eraser radii, so a point that must survive sits unambiguously outside the
 eraser's reach rather than just past its edge.
 */
const POINT_SPACING_PX = 10 * ERASER_RADIUS_PX;

/**
 Stroke color recorded on the fixtures, checked to survive a split.
 */
const FIXTURE_COLOR = '#123456';

/**
 Stroke width recorded on the fixtures, checked to survive a split.
 */
const FIXTURE_WIDTH_PX = 3;

/**
 How many points the splitting case records before the eraser's run of them.
 */
const POINTS_BEFORE_ERASED_RUN = 4;

/**
 How many points the eraser's run in the splitting case takes.

 Two: the reached point, and its successor, because the segment starting on an
 erased point starts on the eraser itself.
 */
const ERASED_RUN_LENGTH = 2;

/**
 How many points the splitting case records after the eraser's run of them.
 */
const POINTS_AFTER_ERASED_RUN = 2;

/**
 Step of the first point the splitting case's eraser takes with it.
 */
const ERASED_FIRST_STEP = POINTS_BEFORE_ERASED_RUN + 1;

/**
 Step of the last point the splitting case's eraser takes with it.
 */
const ERASED_LAST_STEP = (ERASED_FIRST_STEP + ERASED_RUN_LENGTH) - 1;

/**
 Steps of every point the splitting case records, from the leftmost to the rightmost.
 */
const SPLIT_STEPS: readonly number[] = Array.from(
  {
    length: POINTS_BEFORE_ERASED_RUN + ERASED_RUN_LENGTH + POINTS_AFTER_ERASED_RUN,
  },
  function toStep(_unused: unknown, index: number,) {
    return index + 1;
  },
);

/**
 Path of the module whose iteration shape the last suite holds.
 */
const ERASER_SOURCE_PATH = join(
  import.meta.dirname,
  'eraser-strokes.ts',
);

/**
 Header of a C-style counted loop with a mutable counter binding.

 A justified classic loop would mean updating this guard, which is the point:
 the guard fails until someone states why the index form is required.
 */
const CLASSIC_INDEX_LOOP_HEADER = 'for (let';

//endregion Constants

//region Helpers

/**
 Converts a pixel coordinate into the normalized space the widget stores.

 @param px - coordinate in CSS pixels along either axis

 @returns coordinate in [0..1] for a {@link CANVAS_PX} edge
 */
function normalized(px: number,): number {
  return px / CANVAS_PX;
}

/**
 Builds a normalized point from pixel coordinates.

 @param px - x coordinate in CSS pixels

 @param py - y coordinate in CSS pixels

 @returns the point the stroke list stores
 */
function pixelPoint({ px, py, }: {
  readonly px: number;
  readonly py: number;
},): NormalizedPoint {
  return [normalized(px,), normalized(py,),];
}

/**
 Builds the stroke fixture every case records against.

 @param points - points in normalized space

 @returns stroke carrying {@link FIXTURE_COLOR} and {@link FIXTURE_WIDTH_PX}
 */
function fixtureStroke({ points, }: {
  readonly points: readonly NormalizedPoint[];
},): StrokeData {
  return {
    points,
    color: FIXTURE_COLOR,
    width: FIXTURE_WIDTH_PX,
  };
}

/**
 Places one point of the splitting case's horizontal stroke.

 @param step - how many {@link POINT_SPACING_PX} from the left canvas edge

 @returns that point, on the canvas's vertical middle
 */
function splitPointAtStep({ step, }: {
  readonly step: number;
},): NormalizedPoint {
  return pixelPoint({
    px: POINT_SPACING_PX * step,
    py: CANVAS_PX / 2,
  },);
}

//endregion Helpers

await describe({
  name: '',
  // The widget keeps stroke state in one module-level slot, so the cases
  // arrange and read that same state and must not overlap.
  concurrency: 1,
  children: [
    describe({
      name: eraseStrokesAt.name,
      children: [
        //region Point erasure

        it({
          name: 'drops a one-point stroke the eraser reaches',
          fn: async () => {
            const point = pixelPoint({
              px: CANVAS_PX / 2,
              py: CANVAS_PX / 2,
            },);
            setStrokes([fixtureStroke({ points: [point,], },),],);

            const erased = eraseStrokesAt({
              point,
              cw: CANVAS_PX,
              ch: CANVAS_PX,
            },);

            expect(erased,).toBe(true,);
            expect(getStrokes(),).toHaveLength(0,);
          },
        },),

        it({
          name: 'keeps an untouched stroke and reports no change',
          fn: async () => {
            const stroke = fixtureStroke({
              points: [
                pixelPoint({
                  px: 0,
                  py: 0,
                },),
                pixelPoint({
                  px: POINT_SPACING_PX,
                  py: 0,
                },),
              ],
            },);
            setStrokes([stroke,],);

            const erased = eraseStrokesAt({
              point: pixelPoint({
                px: CANVAS_PX,
                py: CANVAS_PX,
              },),
              cw: CANVAS_PX,
              ch: CANVAS_PX,
            },);

            expect(erased,).toBe(false,);
            expect(getStrokes(),).toHaveLength(1,);
            expect(getStrokes()[0],).toBe(stroke,);
          },
        },),

        //endregion Point erasure

        //region Segment handling

        it({
          name: 'splits one stroke into the sub-strokes around the erased points',
          fn: async () => {
            const points = SPLIT_STEPS.map(function toSplitPoint(step: number,) {
              return splitPointAtStep({ step, },);
            },);
            setStrokes([fixtureStroke({ points, },),],);

            const erased = eraseStrokesAt({
              point: splitPointAtStep({ step: ERASED_FIRST_STEP, },),
              cw: CANVAS_PX,
              ch: CANVAS_PX,
            },);

            const firstErasedIndex = ERASED_FIRST_STEP - 1;
            expect(erased,).toBe(true,);
            expect(getStrokes(),).toEqual([
              fixtureStroke({ points: points.slice(0, firstErasedIndex,), },),
              fixtureStroke({ points: points.slice(ERASED_LAST_STEP,), },),
            ],);
          },
        },),

        it({
          name: 'erases a stroke the eraser crosses between two far-apart points',
          fn: async () => {
            setStrokes([
              fixtureStroke({
                points: [
                  pixelPoint({
                    px: POINT_SPACING_PX,
                    py: CANVAS_PX / 2,
                  },),
                  pixelPoint({
                    px: CANVAS_PX - POINT_SPACING_PX,
                    py: CANVAS_PX / 2,
                  },),
                ],
              },),
            ],);

            const erased = eraseStrokesAt({
              point: pixelPoint({
                px: CANVAS_PX / 2,
                py: POINT_SPACING_PX,
              },),
              previousPoint: pixelPoint({
                px: CANVAS_PX / 2,
                py: CANVAS_PX - POINT_SPACING_PX,
              },),
              cw: CANVAS_PX,
              ch: CANVAS_PX,
            },);

            // Neither point sits within the eraser's radius; only the stroke
            // segment between them meets the eraser's travel segment, which
            // erases the point closing it and drops the lone point left over.
            expect(erased,).toBe(true,);
            expect(getStrokes(),).toHaveLength(0,);
          },
        },),

        //endregion Segment handling

        //region Sparse point arrays

        it({
          name: 'skips a hole in the point array instead of matching a segment against nothing',
          fn: async () => {
            const holeyPoints: NormalizedPoint[] = [];
            holeyPoints[0] = pixelPoint({
              px: 0,
              py: 0,
            },);
            holeyPoints[2] = pixelPoint({
              px: POINT_SPACING_PX,
              py: 0,
            },);
            const stroke = fixtureStroke({ points: holeyPoints, },);
            setStrokes([stroke,],);

            const erased = eraseStrokesAt({
              point: pixelPoint({
                px: CANVAS_PX,
                py: CANVAS_PX,
              },),
              cw: CANVAS_PX,
              ch: CANVAS_PX,
            },);

            // Positive control for the fixture: the array really is sparse, so
            // the skipped iteration reads genuine absence rather than an
            // in-bounds element an assignment filled in.
            expect(1 in holeyPoints,).toBe(false,);
            expect(erased,).toBe(false,);
            expect(getStrokes()[0],).toBe(stroke,);
          },
        },),

        //endregion Sparse point arrays
      ],
    },),

    describe({
      name: 'point iteration',
      children: [
        it({
          name: 'walks stroke points with no classic index loop',
          fn: async () => {
            const source = await readFile(
              ERASER_SOURCE_PATH,
              'utf8',
            );

            expect(source,).not.toContain(CLASSIC_INDEX_LOOP_HEADER,);
          },
        },),
      ],
    },),
  ],
},);
