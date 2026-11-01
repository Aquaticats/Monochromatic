/**
 Tests for which slices the window trial buys.

 WHAT THESE PIN are the two ways this draw can quietly ruin the measurement it
 feeds. Buying a slice twice, which relocation candidates invite because they
 are adjacencies, spends quota twice and counts one model's answer twice.
 Drawing controls badly, or not at all, leaves a general context-induced
 conservatism indistinguishable from the window working on relocations, and
 the judge-quality bench measured the roster declining on any archive
 imperfection, so that is a
 live possibility.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { DocumentDisplacement, } from '@monochromatic-dev/translation-repair-artifact';
import {
  CONTROL_CLASS,
  controlSlices,
  flaggedSlices,
  RELOCATION_CLASSES,
} from '../../dist/final/node/index.mjs';

/**
 Builds a screen reading with the flags a case wants.

 @param sliceCount - slices the document has

 @param relocation - adjacent pairs, as high and low index

 @param untranslated - slices with a negligible translation

 @param targetOnly - slices whose original is negligible

 @param otherImbalances - everything else the screen flagged

 @returns Reading shaped like one `classifyDisplacement` returns

 @example
 ```ts
 const reading = readingFor({ sliceCount: 6, relocation: [[1, 2]], },);
 ```
 */
function readingFor(
  {
    sliceCount,
    relocation = [],
    untranslated = [],
    targetOnly = [],
    otherImbalances = [],
  }: {
    readonly sliceCount: number;
    readonly relocation?: readonly (readonly [number, number])[];
    readonly untranslated?: readonly number[];
    readonly targetOnly?: readonly number[];
    readonly otherImbalances?: readonly number[];
  },
): DocumentDisplacement {
  return {
    baseline: 2.86,
    baselineFrom: 'document',
    slices: Array.from(
      { length: sliceCount, },
      function toSlice() {
        return {
          sliceClass: 'translated',
          ratio: 2.8,
          residual: 0,
        };
      },
    ),
    untranslated,
    targetOnly,
    relocationCandidates: relocation.map(function toCandidate([high, low,],) {
      return {
        high,
        low,
        surplus: 400,
      };
    },),
    otherImbalances,
  } as unknown as DocumentDisplacement;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: flaggedSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'BUYS A SLICE ONCE even when two relocation candidates share it, which they do by '
            + 'construction since candidates are adjacencies: buying it twice would spend quota twice '
            + 'and count one model\'s answer twice in the tally',
          fn: async () => {
            /**
             Slice 2 is the low end of one candidate and the high end of the next.
             */
            const flagged = flaggedSlices({
              entryId: 'Mittens',
              displacement: readingFor({
                sliceCount: 5,
                relocation: [[1,
                  2,], [2,
                    3,],],
              },),
            },);
            expect(flagged.length,).toBe(3,);
            expect(flagged.map(function toIndex(slice,) {
              return slice.sliceIndex;
            },),).toEqual([1,
              2,
              3,],);
          },
        },),
        it({
          name: 'gives a slice flagged two ways a RELOCATION label, since that is the class the relocation finding is '
            + 'about and the one the window is expected to move; dropping multiply-flagged slices '
            + 'would discard exactly the ambiguous cases the trial exists to resolve',
          fn: async () => {
            const flagged = flaggedSlices({
              entryId: 'Mittens',
              displacement: readingFor({
                sliceCount: 4,
                relocation: [[0,
                  1,],],
                untranslated: [1,],
              },),
            },);

            /**
             Slice 1, which the screen flagged both ways.
             */
            const both = flagged.find(function isOne(slice,) {
              return slice.sliceIndex === 1;
            },);
            expect(both?.sliceClass,).toBe(RELOCATION_CLASSES.low,);
            // And it appears once, not twice.
            expect(flagged.length,).toBe(2,);
          },
        },),
        it({
          name: 'SEPARATES THE TWO ENDS of a relocation, because the window can only reach one of '
            + 'them: at the high end the archive carries English the original does not account for '
            + 'and the neighbouring Chinese is where it could have come from, while at the low end '
            + 'the archive is MISSING English and neighbouring Chinese cannot say where it went. '
            + 'Pooled, the low ends would dilute the high ones with a population the treatment cannot '
            + 'touch, and a real effect would read as a weaker one',
          fn: async () => {
            const flagged = flaggedSlices({
              entryId: 'Mittens',
              displacement: readingFor({
                sliceCount: 4,
                relocation: [[1,
                  2,],],
              },),
            },);
            expect(flagged.map(function toClass(slice,) {
              return slice.sliceClass;
            },),).toEqual([RELOCATION_CLASSES.high,
              RELOCATION_CLASSES.low,],);
          },
        },),
        it({
          name: 'returns slices in document order, so a run that dies part way through has bought a '
            + 'contiguous prefix rather than a scatter nobody can characterise',
          fn: async () => {
            const flagged = flaggedSlices({
              entryId: 'Mittens',
              displacement: readingFor({
                sliceCount: 9,
                otherImbalances: [7,
                  2,],
                untranslated: [5,],
              },),
            },);
            expect(flagged.map(function toIndex(slice,) {
              return slice.sliceIndex;
            },),).toEqual([2,
              5,
              7,],);
          },
        },),
      ],
    },),

    describe({
      name: controlSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'draws only slices the screen left ALONE, since a control that was itself flagged '
            + 'measures nothing: the comparison is flagged against unflagged',
          fn: async () => {
            const controls = controlSlices({
              entryId: 'Mittens',
              displacement: readingFor({
                sliceCount: 6,
                untranslated: [0,
                  1,
                  2,],
              },),
              wanted: 3,
            },);
            for (const control of controls) {
              expect([0,
                1,
                2,].includes(control.sliceIndex,),).toBe(false,);
            }
            expect(controls.every(function labelled(control,) {
              return control.sliceClass === CONTROL_CLASS;
            },),).toBe(true,);
          },
        },),
        it({
          name: 'SPREADS THE DRAW rather than taking the front, because slices early in a document '
            + 'carry the opening and several entries begin with a heading and a stub that a judge '
            + 'reads differently from body prose',
          fn: async () => {
            const controls = controlSlices({
              entryId: 'Mittens',
              displacement: readingFor({ sliceCount: 12, },),
              wanted: 3,
            },);
            expect(controls.map(function toIndex(control,) {
              return control.sliceIndex;
            },),).toEqual([2,
              6,
              10,],);
          },
        },),
        it({
          name: 'draws the MIDDLE slice when asked for one, not the first: at a single control the '
            + 'stride is the whole document, so a draw taken at the start of every stride would make '
            + 'every control in the corpus an opening, and the class exists to detect a general '
            + 'context conservatism rather than an opening effect',
          fn: async () => {
            const controls = controlSlices({
              entryId: 'Mittens',
              displacement: readingFor({ sliceCount: 9, },),
              wanted: 1,
            },);
            expect(controls.map(function toIndex(control,) {
              return control.sliceIndex;
            },),).toEqual([4,],);
          },
        },),
        it({
          name: 'returns what it can when the entry has fewer unflagged slices than asked for, rather '
            + 'than refusing: a short document still contributes the controls it has',
          fn: async () => {
            const controls = controlSlices({
              entryId: 'Mittens',
              displacement: readingFor({
                sliceCount: 3,
                untranslated: [0,
                  1,],
              },),
              wanted: 5,
            },);
            expect(controls.length,).toBe(1,);
            expect(controls[0]?.sliceIndex,).toBe(2,);
          },
        },),
        it({
          name: 'draws NOTHING from a document the screen flagged entirely, rather than falling back '
            + 'to a flagged slice, since a control that is not a control is worse than one fewer',
          fn: async () => {
            expect(controlSlices({
              entryId: 'Mittens',
              displacement: readingFor({
                sliceCount: 2,
                untranslated: [0,
                  1,],
              },),
              wanted: 2,
            },).length,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
