/**
 Tests for the driver that runs the editor calibration's slices: that it
 returns their rounds in sample order whatever order they finish in, numbers
 each progress line by position, and admits no more slices than the overlap.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type BenchSlice,
  driveEditorCalibrate,
  type SliceRounds,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/rejection-of.test-fixture.ts';
import {
  benchSliceOf,
  unjudgedRounds,
} from './editor-calibrate-rounds.test-fixture.ts';

/**
 Three slices of one invented entry, in sample order.
 */
const SAMPLE: readonly BenchSlice[] = [
  benchSliceOf({ entryId: 'mittens', index: 0, },),
  benchSliceOf({ entryId: 'mittens', index: 1, },),
  benchSliceOf({ entryId: 'mittens', index: 2, },),
];

/**
 What each of the three slices produced, told apart by who shipped it.
 */
const ROUNDS: readonly SliceRounds[] = [
  unjudgedRounds({ shippers: [SEAT_HYPER_VISION,], refineAsked: true, },),
  unjudgedRounds({ shippers: [SEAT_HYPER_TEXT_BEDROCK,], refineAsked: true, },),
  unjudgedRounds({ shippers: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,], refineAsked: true, },),
];

/**
 Progress line the slice at a position prints, from `sliceProgressLine`.

 @param position - where the slice sits in the sample, counted from zero

 @returns Whole line

 @example
 ```ts
 const line = progressLineAt({ position: 0, },);
 ```
 */
function progressLineAt({ position, }: { readonly position: number; },): string {
  return `  slice ${String(position + 1,)} of 3 (mittens chunk ${String(position,)}): `
    + '0 editor rounds, 0 refiner rounds, 1 editor shipping';
}

/**
 Rounds the slice produced, which the fixture fixes by its position.

 @param slice - slice the driver handed over

 @returns Rounds scripted for that position

 @throws Error when the slice is not one of the sample's

 @example
 ```ts
 const rounds = roundsOf({ slice: SAMPLE[0], },);
 ```
 */
function roundsOf({ slice, }: { readonly slice: BenchSlice; },): SliceRounds {
  return nonNullishOrThrow(ROUNDS[slice.index],);
}

/**
 Runs the sample with a given overlap and counts how many slices were in
 flight at once.

 @param overlap - slices the driver may admit at once

 @returns Most slices running at one moment

 @example
 ```ts
 const widest = await widestAdmitted({ overlap: 1, },);
 ```
 */
async function widestAdmitted({ overlap, }: { readonly overlap: number; },): Promise<number> {
  /**
   Slices running right now.
   */
  let running = 0;

  /**
   Most slices running at one moment so far.
   */
  let widest = 0;

  await driveEditorCalibrate({
    sample: SAMPLE,
    overlap,
    runSlice: async function counting({ slice, },): Promise<SliceRounds> {
      running += 1;
      widest = Math.max(
        widest,
        running,
      );
      // Yields once so a driver that admitted every slice would have all of them running.
      await Promise.resolve();
      running -= 1;
      return roundsOf({ slice, },);
    },
  },);

  return widest;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: driveEditorCalibrate.name,
      concurrency: 1,
      children: [
        it({
          name: 'RETURNS the rounds in sample order and numbers each line by position when the first slice finishes last',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Opens when the last slice has finished, which holds the first back.
             */
            const lastFinished = Promise.withResolvers<undefined>();

            /**
             What the driver returned.
             */
            const perSlice = await driveEditorCalibrate({
              sample: SAMPLE,
              overlap: 3,
              runSlice: async function finishingInReverse({ slice, },): Promise<SliceRounds> {
                if (slice.index === 0)
                  await lastFinished.promise;
                if (slice.index === 2)
                  lastFinished.resolve(undefined,);
                return roundsOf({ slice, },);
              },
            },);

            expect(perSlice,).toEqual(ROUNDS,);
            expect(printed.lines,).toEqual([
              progressLineAt({ position: 1, },),
              progressLineAt({ position: 2, },),
              progressLineAt({ position: 0, },),
            ],);
          },
        },),

        it({
          name: 'REFUSES with the first slice\'s refusal in sample order when two slices are refused and the later '
            + 'one is refused first',
          fn: async (ctx,) => {
            using _printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Opened by the last slice as it is refused, which the first slice
             waits for.
             */
            const lastRefused = Promise.withResolvers<undefined>();

            /**
             What the driver refused with.
             */
            const refusal = await rejectionOf({
              promise: driveEditorCalibrate({
                sample: SAMPLE,
                overlap: 3,
                runSlice: async function refusingInReverse({ slice, },): Promise<SliceRounds> {
                  if (slice.index === 2) {
                    lastRefused.resolve(undefined,);
                    throw new Error('slice 2 was refused',);
                  }
                  if (slice.index === 0) {
                    await lastRefused.promise;
                    throw new Error('slice 0 was refused',);
                  }
                  return roundsOf({ slice, },);
                },
              },),
            },);

            expect(String(refusal,),).toBe('Error: slice 0 was refused',);
          },
        },),

        it({
          name: 'ADMITS one slice at a time when the overlap is one, and prints the lines in sample order',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            /**
             Most slices the driver had running at once.
             */
            const widest = await widestAdmitted({ overlap: 1, },);

            expect(widest,).toBe(1,);
            expect(printed.lines,).toEqual([
              progressLineAt({ position: 0, },),
              progressLineAt({ position: 1, },),
              progressLineAt({ position: 2, },),
            ],);
          },
        },),

        it({
          name: 'ADMITS two slices at once and no third when the overlap is two over three slices',
          fn: async (ctx,) => {
            using _printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            expect(await widestAdmitted({ overlap: 2, },),).toBe(2,);
          },
        },),

        it({
          name: 'RETURNS no rounds and prints no line when the sample is empty',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            /**
             What the driver returned for no slices.
             */
            const perSlice = await driveEditorCalibrate({
              sample: [],
              overlap: 4,
              runSlice: function neverCalled({ slice, },): Promise<SliceRounds> {
                return Promise.reject(new Error(`no slice was expected, got ${slice.entryId}`,),);
              },
            },);

            expect(perSlice,).toEqual([],);
            expect(printed.lines,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
