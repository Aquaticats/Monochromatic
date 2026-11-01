/**
 Tests for the repeat census's positive control: which lines a group stands
 on, whether it covers the control's lines, and which controls of the two
 readings failed, which the command turns into its exit. Fixtures are
 cat-themed invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ControlReading,
  controlReadingOf,
  coversAllLines,
  failedControlsOf,
  occupiedLinesOf,
} from '../../dist/final/node/index.mjs';

/**
 What follows a control's name where its reading found no control entry.
 */
const ABSENT_NOTE = '(the TianqiChen666 entry is absent from the census)';

/**
 Reading in which both controls held.
 */
const HELD: ControlReading = controlReadingOf({
  refrain: true,
  maximal: true,
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: occupiedLinesOf.name,
      children: [
        it({
          name: 'Lists the line of each occurrence in the order given, repeats kept',
          fn: async () => {
            expect(occupiedLinesOf({ occurrences: [{ line: 35, }, { line: 17, }, { line: 35, },], },),).toEqual([
              35,
              17,
              35,
            ],);
          },
        },),

        it({
          name: 'Lists no line for no occurrence',
          fn: async () => {
            expect(occupiedLinesOf({ occurrences: [], },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: coversAllLines.name,
      children: [
        it({
          name: 'Covers the lines when every one of them is held, whatever else is held',
          fn: async () => {
            expect(coversAllLines({ held: [37, 17, 99, 25,], lines: [17, 25, 37,], },),).toBe(true,);
          },
        },),

        it({
          name: 'Does not cover the lines when one of them is not held',
          fn: async () => {
            expect(coversAllLines({ held: [17, 25,], lines: [17, 25, 37,], },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: controlReadingOf.name,
      children: [
        it({
          name: 'Names the two controls as the census prints them',
          fn: async () => {
            expect(controlReadingOf({ refrain: true, maximal: false, },),).toEqual({
              refrainCoversLines17To37: true,
              maximalCoversLines35And55: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: failedControlsOf.name,
      children: [
        it({
          name: 'Names nothing when every control of both readings held',
          fn: async () => {
            expect(failedControlsOf({ readings: { lines: HELD, package: HELD, }, },),).toEqual([],);
          },
        },),

        it({
          name: 'Names the refrain control of the package reading alone when only it read false',
          fn: async () => {
            expect(failedControlsOf({
              readings: {
                lines: HELD,
                package: controlReadingOf({ refrain: false, maximal: true, },),
              },
            },),).toEqual(['package.refrainCoversLines17To37',],);
          },
        },),

        it({
          name: 'Names the maximal control of the line reading alone when only it read false',
          fn: async () => {
            expect(failedControlsOf({
              readings: {
                lines: controlReadingOf({ refrain: true, maximal: false, },),
                package: HELD,
              },
            },),).toEqual(['lines.maximalCoversLines35And55',],);
          },
        },),

        it({
          name: 'Names every control that read false, the line reading first and the refrain control before the maximal',
          fn: async () => {
            /** Reading in which both controls read false. */
            const neither = controlReadingOf({ refrain: false, maximal: false, },);

            expect(failedControlsOf({ readings: { lines: neither, package: neither, }, },),).toEqual([
              'lines.refrainCoversLines17To37',
              'lines.maximalCoversLines35And55',
              'package.refrainCoversLines17To37',
              'package.maximalCoversLines35And55',
            ],);
          },
        },),

        it({
          name: 'Names both controls of a reading that found no control entry, with why, and not the other reading',
          fn: async () => {
            expect(failedControlsOf({ readings: { lines: HELD, package: 'absent', }, },),).toEqual([
              `package.refrainCoversLines17To37 ${ABSENT_NOTE}`,
              `package.maximalCoversLines35And55 ${ABSENT_NOTE}`,
            ],);
          },
        },),

        it({
          name: 'Names all four controls when neither reading found the control entry',
          fn: async () => {
            expect(failedControlsOf({ readings: { lines: 'absent', package: 'absent', }, },),).toEqual([
              `lines.refrainCoversLines17To37 ${ABSENT_NOTE}`,
              `lines.maximalCoversLines35And55 ${ABSENT_NOTE}`,
              `package.refrainCoversLines17To37 ${ABSENT_NOTE}`,
              `package.maximalCoversLines35And55 ${ABSENT_NOTE}`,
            ],);
          },
        },),
      ],
    },),
  ],
},);
