/**
 Tests for what a slice ships where the archive has no text and no wording was backed.

 The owner, 2026-10-23, chose "Source text + note": the page carries the original text of the passage,
 preceded by a note, and a later run replaces it once a wording is backed. The page-level cases are in
 `source-shipped-page.unit.test.ts`; these pin the three small subjects the page is read through.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  carriesText,
  carryingSource,
  markRuns,
  type TextCarryingReading,
  textOnPage,
  UNTRANSLATED_PASSAGE_NOTE,
  type WouldShipReading,
  type WouldShipSlice,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';

/**
 The silence of an anchor nobody filled.
 */
const SILENT_ANCHOR: WouldShipReading = {
  kind: 'nothing-ships',
  reason: 'lanes-agreed-on-no-wording',
  incumbentKind: 'absent',
};

/**
 Artifact whose translate rows record the original of slice 4.

 @param sourceText - original the row records

 @returns Artifact carrying only the rows the reading reads

 @example
 ```ts
 const artifact = recording({ sourceText: '猫\n', },);
 ```
 */
function recording({ sourceText, }: { readonly sourceText: string; },): WouldShipSource {
  return {
    comparison: [],
    consolidation: { kind: 'not-run', },
    laneSelection: { kind: 'pending-human-decision', },
    lanes: {
      translate: {
        delivery: [{
          sliceIndex: 4,
          sourceText,
        },],
      },
    },
  } as unknown as WouldShipSource;
}

/**
 One slice shipping its original.

 @param sliceIndex - slice index
 @param opensRun - whether the note stands before it

 @returns The slice

 @example
 ```ts
 const slice = shipping({ sliceIndex: 1, opensRun: true, },);
 ```
 */
function shipping({ sliceIndex, opensRun, }: { readonly sliceIndex: number; readonly opensRun: boolean; },): WouldShipSlice {
  return {
    sliceIndex,
    reading: {
      kind: 'source-ships',
      reason: 'lanes-agreed-on-no-wording',
      incumbentKind: 'absent',
      text: '猫\n',
      opensRun,
    },
  };
}

/**
 A reading that ships the original text of a slice.

 @param opensRun - whether the note stands before it

 @returns The reading

 @example
 ```ts
 const reading = originalOf({ opensRun: true, },);
 ```
 */
function originalOf({ opensRun, }: { readonly opensRun: boolean; },): TextCarryingReading {
  return {
    kind: 'source-ships',
    reason: 'lanes-agreed-on-no-wording',
    incumbentKind: 'absent',
    text: '猫\n',
    opensRun,
  };
}

/**
 Whether a slice opens a run of slices shipping their original, continues one, or ships something else.

 @param slice - slice read

 @returns Its role in a run

 @example
 ```ts
 const role = runRoleOf({ sliceIndex: 0, reading: SILENT_ANCHOR, },);
 ```
 */
function runRoleOf(slice: WouldShipSlice,): 'continues' | 'opens' | 'other' {
  /**
   What the slice ships.
   */
  const { reading, } = slice;
  if (reading.kind !== 'source-ships')
    return 'other';
  return reading.opensRun ? 'opens' : 'continues';
}

await describe({
  name: '',
  children: [
    describe({
      name: carryingSource.name,
      children: [
        it({
          name: 'GIVES AN UNFILLED ANCHOR THE ORIGINAL THE ARTIFACT RECORDS FOR IT, keeping the silence it had as the reason',
          fn: async () => {
            expect(carryingSource({
              artifact: recording({ sourceText: '猫\n', },),
              sliceIndex: 4,
              reading: SILENT_ANCHOR,
            },),).toEqual({
              kind: 'source-ships',
              reason: 'lanes-agreed-on-no-wording',
              incumbentKind: 'absent',
              text: '猫\n',
              opensRun: true,
            },);
          },
        },),

        it({
          name: 'LEAVES A SILENT CONTENT SPAN SILENT, since the archive had wording there and the deciders removed it',
          fn: async () => {
            /**
             Silence over a span the archive holds wording for.
             */
            const removed: WouldShipReading = {
              ...SILENT_ANCHOR,
              incumbentKind: 'present',
            };
            expect(carryingSource({
              artifact: recording({ sourceText: '猫\n', },),
              sliceIndex: 4,
              reading: removed,
            },),).toBe(removed,);
          },
        },),

        it({
          name: 'LEAVES A SLICE WHOSE ORIGINAL RENDERS AS NOTHING SILENT, and so a slice the artifact has no row for, '
            + 'and an artifact with no lanes at all',
          fn: async () => {
            expect(carryingSource({
              artifact: recording({ sourceText: ' ​\n', },),
              sliceIndex: 4,
              reading: SILENT_ANCHOR,
            },),).toBe(SILENT_ANCHOR,);
            expect(carryingSource({
              artifact: recording({ sourceText: '猫\n', },),
              sliceIndex: 5,
              reading: SILENT_ANCHOR,
            },),).toBe(SILENT_ANCHOR,);
            expect(carryingSource({
              artifact: {
                comparison: [],
                consolidation: { kind: 'not-run', },
                laneSelection: { kind: 'pending-human-decision', },
              },
              sliceIndex: 4,
              reading: SILENT_ANCHOR,
            },),).toBe(SILENT_ANCHOR,);
          },
        },),

        it({
          name: 'PASSES A READING THAT CARRIES WORDING THROUGH UNCHANGED',
          fn: async () => {
            /**
             Wording a stage settled.
             */
            const wording: WouldShipReading = {
              kind: 'wording',
              text: 'A cat.\n',
              decidedBy: 'archive',
            };
            expect(carryingSource({
              artifact: recording({ sourceText: '猫\n', },),
              sliceIndex: 4,
              reading: wording,
            },),).toBe(wording,);
          },
        },),
      ],
    },),

    describe({
      name: markRuns.name,
      children: [
        it({
          name: 'KEEPS THE NOTE ON THE FIRST SLICE OF EACH CONTIGUOUS RUN and takes it off the slices after it',
          fn: async () => {
            /**
             Slices: a run of two, a wording, a run of one.
             */
            const slices: readonly WouldShipSlice[] = [
              shipping({ sliceIndex: 0, opensRun: true, },),
              shipping({ sliceIndex: 1, opensRun: true, },),
              {
                sliceIndex: 2,
                reading: {
                  kind: 'wording',
                  text: 'A cat.\n',
                  decidedBy: 'archive',
                },
              },
              shipping({ sliceIndex: 3, opensRun: true, },),
            ];
            expect(markRuns({ slices, },).map(runRoleOf,),).toEqual([
              'opens',
              'continues',
              'other',
              'opens',
            ],);
          },
        },),

        it({
          name: 'ENDS A RUN AT A SILENT SLICE TOO, so a gap between two slices that ship their original opens a second run',
          fn: async () => {
            /**
             Slices: one shipping, one silent, one shipping.
             */
            const slices: readonly WouldShipSlice[] = [
              shipping({ sliceIndex: 0, opensRun: true, },),
              {
                sliceIndex: 1,
                reading: SILENT_ANCHOR,
              },
              shipping({ sliceIndex: 2, opensRun: true, },),
            ];
            expect(markRuns({ slices, },).map(runRoleOf,),).toEqual([
              'opens',
              'other',
              'opens',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: textOnPage.name,
      children: [
        it({
          name: 'PUTS THE NOTE, A BLANK LINE AND THE ORIGINAL ON THE PAGE for a slice that opens a run, and the original '
            + 'alone for one that continues it, with no blank line of the original\'s own around it',
          fn: async () => {
            expect(textOnPage({ reading: originalOf({ opensRun: true, },), },),)
              .toBe(`${UNTRANSLATED_PASSAGE_NOTE}\n\n猫`,);
            expect(textOnPage({ reading: originalOf({ opensRun: false, },), },),).toBe('猫',);
          },
        },),

        it({
          name: 'PUTS WORDING ON THE PAGE AS IT IS and nothing for a silence',
          fn: async () => {
            expect(textOnPage({
              reading: {
                kind: 'wording',
                text: 'A cat.\n',
                decidedBy: 'archive',
              },
            },),).toBe('A cat.\n',);
            expect(carriesText(SILENT_ANCHOR,),).toBe(false,);
            /**
             A reading that ships the original.
             */
            const original = originalOf({ opensRun: false, },);
            expect(carriesText(original,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
