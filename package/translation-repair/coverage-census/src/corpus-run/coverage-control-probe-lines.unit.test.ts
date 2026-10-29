/**
 Tests for what the coverage control probe prints: the line before the
 roster is asked and the reading after it, over control results a case
 writes, with every count noun shown in the singular and the plural and every
 closing sentence reached.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  controlLines,
  type CoverageControlResult,
  offeringLine,
} from '../../dist/final/node/index.mjs';

/**
 One case damaged and re-asked, which the lines only count.
 */
const DAMAGED = {
  where: 'Mittens section 0',
  before: 'carried',
  after: 'absent',
  absentBefore: 0,
  absentAfter: 3,
  decoy: 'carried',
  absentAfterDecoy: 0,
  decoyAt: 12,
  removedSpans: 1,
  removedChars: 20,
} as const;

/**
 One case that could not be damaged because the roster never called it
 covered.
 */
const NOT_CARRIED = {
  where: 'Tabby section 1',
  reason: 'not-carried',
  verdict: 'absent',
  absent: 3,
  offeredSpans: 0,
} as const;

/**
 One case that could not be damaged because the cut would leave nothing.
 */
const LEFT_NOTHING = {
  where: 'Whiskers section 2',
  reason: 'cut-left-nothing',
  verdict: 'carried',
  absent: 0,
  offeredSpans: 1,
} as const;

/**
 A result nothing was damaged in, which a case changes in the fields it is
 about.

 @param changes - fields to set

 @returns Result

 @example
 ```ts
 const control = resultWith({ changes: { held: true, }, },);
 ```
 */
function resultWith(
  { changes, }: {
    readonly changes: {
      readonly held?: boolean;
      readonly sawAbsenceOnTarget?: number;
      readonly sawAbsenceOnDecoy?: number;
      readonly decoysTaken?: number;
      readonly rows?: CoverageControlResult['rows'];
      readonly refusals?: CoverageControlResult['refusals'];
    };
  },
): CoverageControlResult {
  return {
    held: false,
    sawAbsenceOnTarget: 0,
    sawAbsenceOnDecoy: 0,
    decoysTaken: 0,
    rows: [],
    refusals: [],
    ...changes,
  };
}

/**
 Closing sentence of a control that held.
 */
const HELD_SENTENCE = 'The roster voted absence once the rendering it pointed at was gone, so an absence '
  + 'vote is reachable and a run that produced none is reporting the corpus rather '
  + 'than the instrument.';

/**
 Closing sentence of a control that damaged nothing.
 */
const NOTHING_DAMAGED_SENTENCE = 'NOTHING WAS DAMAGED on this entry, so it says nothing either way about whether a '
  + 'deleted rendering is noticed. Read the refusals this probe printed instead: they are what the '
  + 'roster says about this page as it stands.';

/**
 Closing sentence of a control that took no decoy cut.
 */
const NO_DECOY_SENTENCE = 'NO DECOY CUT COULD BE TAKEN on any damaged case, so whether the absence votes follow the '
  + 'passage or any cut of the same size was never asked; the targeted cuts printed here show '
  + 'only whether an absence vote is reachable.';

/**
 Closing sentence of a control whose roster did not vote absence.
 */
const NOT_HELD_SENTENCE = 'THE ROSTER DID NOT VOTE ABSENCE even with the rendering it pointed at deleted, or it '
  + 'voted absence on an unrelated cut of the same size. Either way its coverage readings '
  + 'here are a property of the wire rather than of the translation.';

/**
 Warning printed when the roster declined an undamaged passage.
 */
const UNDAMAGED_ABSENCE = 'THE ROSTER DECLINED TO CALL THOSE PASSAGES COVERED WITH NO DAMAGE DONE, which is an '
  + 'absence reading on standing corpus text rather than on a page this probe cut.';

await describe({
  name: 'coverage-control-probe-lines',
  children: [
    describe({
      name: offeringLine.name,
      children: [
        it({
          name: 'SAYS case in the singular for one case',
          fn: async () => {
            expect(offeringLine({
              caseCount: 1,
              rosterSize: 8,
            },),).toBe('COVERAGE control offering 1 case to a roster of 8',);
          },
        },),
        it({
          name: 'SAYS cases in the plural for several',
          fn: async () => {
            expect(offeringLine({
              caseCount: 5,
              rosterSize: 8,
            },),).toBe('COVERAGE control offering 5 cases to a roster of 8',);
          },
        },),
      ],
    },),
    describe({
      name: controlLines.name,
      children: [
        it({
          name: 'SAYS HELD with one targeted cut and one decoy cut in the singular, then the sentence for a control that held',
          fn: async () => {
            expect(controlLines({ control: resultWith({ changes: {
              held: true,
              rows: [DAMAGED,],
              sawAbsenceOnTarget: 1,
              sawAbsenceOnDecoy: 1,
              decoysTaken: 1,
            }, },), },),).toEqual([
              'COVERAGE control HELD over 1 damaged case: absence votes appeared on 1 targeted cut '
                + 'and on 1 of 1 equally large cut taken elsewhere',
              'COVERAGE control 0 cases could not be damaged: 0 because the roster never called them covered, '
                + '0 because what the roster anchored on was the whole page',
              HELD_SENTENCE,
            ],);
          },
        },),
        it({
          name: 'SAYS DID NOT HOLD with several cuts in the plural, and the cases that could not be damaged by reason, and warns '
            + 'of the roster declining an undamaged passage before the sentence for a roster that voted no absence',
          fn: async () => {
            expect(controlLines({ control: resultWith({ changes: {
              rows: [DAMAGED, DAMAGED,],
              sawAbsenceOnTarget: 0,
              sawAbsenceOnDecoy: 0,
              decoysTaken: 2,
              refusals: [NOT_CARRIED, LEFT_NOTHING,],
            }, },), },),).toEqual([
              'COVERAGE control DID NOT HOLD over 2 damaged cases: absence votes appeared on 0 targeted cuts '
                + 'and on 0 of 2 equally large cuts taken elsewhere',
              'COVERAGE control 2 cases could not be damaged: 1 because the roster never called them covered, '
                + '1 because what the roster anchored on was the whole page',
              UNDAMAGED_ABSENCE,
              NOT_HELD_SENTENCE,
            ],);
          },
        },),
        it({
          name: 'SAYS nothing was damaged where no case could be, with one case that could not be, in the singular',
          fn: async () => {
            expect(controlLines({ control: resultWith({ changes: { refusals: [LEFT_NOTHING,], }, },), },),).toEqual([
              'COVERAGE control DID NOT HOLD over 0 damaged cases: absence votes appeared on 0 targeted cuts '
                + 'and on 0 of 0 equally large cuts taken elsewhere',
              'COVERAGE control 1 case could not be damaged: 0 because the roster never called them covered, '
                + '1 because what the roster anchored on was the whole page',
              NOTHING_DAMAGED_SENTENCE,
            ],);
          },
        },),
        it({
          name: 'SAYS no decoy cut could be taken where cases were damaged and none had room for one',
          fn: async () => {
            expect(controlLines({ control: resultWith({ changes: {
              rows: [DAMAGED,],
              sawAbsenceOnTarget: 1,
            }, },), },),).toEqual([
              'COVERAGE control DID NOT HOLD over 1 damaged case: absence votes appeared on 1 targeted cut '
                + 'and on 0 of 0 equally large cuts taken elsewhere',
              'COVERAGE control 0 cases could not be damaged: 0 because the roster never called them covered, '
                + '0 because what the roster anchored on was the whole page',
              NO_DECOY_SENTENCE,
            ],);
          },
        },),
        it({
          name: 'WARNS of the roster declining an undamaged passage before the sentence for a control that held',
          fn: async () => {
            expect(controlLines({ control: resultWith({ changes: {
              held: true,
              rows: [DAMAGED,],
              sawAbsenceOnTarget: 1,
              decoysTaken: 1,
              refusals: [NOT_CARRIED,],
            }, },), },),).toEqual([
              'COVERAGE control HELD over 1 damaged case: absence votes appeared on 1 targeted cut '
                + 'and on 0 of 1 equally large cut taken elsewhere',
              'COVERAGE control 1 case could not be damaged: 1 because the roster never called them covered, '
                + '0 because what the roster anchored on was the whole page',
              UNDAMAGED_ABSENCE,
              HELD_SENTENCE,
            ],);
          },
        },),
      ],
    },),
  ],
},);
