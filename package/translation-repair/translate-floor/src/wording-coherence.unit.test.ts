/**
 Tests for the rule tying a lane's outcome to what the archive holds.

 WHAT THESE PIN is that the two axes of a wording are independent and not
 unconstrained. Three of their combinations describe a slice that cannot
 exist, every field of each is individually well formed, and no later join or
 count could detect one: a fallback onto wording that was never there reads
 downstream as a translation being kept.

 `buildLaneSliceTexts` refuses all three while building. This is the same rule
 at the boundaries that take wordings from a caller rather than making them.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assertWordingCoherent,
  type LaneSliceText,
  WordingCoherenceError,
} from '../dist/final/node/index.mjs';

/**
 Archive wording of the slice the coherent cases use.
 */
const ARCHIVE_NAP = 'The cat sleeps on the sill.';

await describe({
  name: assertWordingCoherent.name,
  children: [
    it({
      name:
        'accepts every combination a real slice can be in: a decision, an unexamined slice, silence where '
          + 'the archive has wording to stand on, and a missing passage where it has none',
      fn: async () => {
        /**
         Wordings a lane can legitimately report.
         */
        const coherent = [
          {
            sliceIndex: 0,
            incumbentKind: 'present',
            incumbentText: ARCHIVE_NAP,
            outcome: {
              kind: 'decided',
              acceptedText: 'The cat is asleep on the windowsill.',
            },
          },
          {
            sliceIndex: 1,
            incumbentKind: 'present',
            incumbentText: ARCHIVE_NAP,
            outcome: { kind: 'not-evaluated', },
          },
          {
            sliceIndex: 2,
            incumbentKind: 'present',
            incumbentText: ARCHIVE_NAP,
            outcome: { kind: 'incumbent-fallback', },
          },
          {
            sliceIndex: 3,
            incumbentKind: 'absent',
            incumbentText: '',
            outcome: { kind: 'unfilled', },
          },
          // An anchor nobody reached is still just an anchor nobody reached.
          {
            sliceIndex: 4,
            incumbentKind: 'absent',
            incumbentText: '',
            outcome: { kind: 'not-evaluated', },
          },
          // And an anchor a lane FILLED is the point of anchors existing.
          {
            sliceIndex: 5,
            incumbentKind: 'absent',
            incumbentText: '',
            outcome: {
              kind: 'decided',
              acceptedText: 'The cat has a bowl of its own.',
            },
          },
        ] satisfies readonly LaneSliceText[];
        for (const wording of coherent)
          assertWordingCoherent({ wording, },);
      },
    },),
    it({
      name:
        'REFUSES a fallback onto wording the archive does not hold, since `incumbent-fallback` says the '
          + 'archive`s own English stands here and there is none: the row would report a passage as covered '
          + 'by a translation that has never existed',
      fn: async () => {
        /**
         What fellBackOnNothing raised, read for its class as well as its wording.
         */
        const refusalOfFellBackOnNothing = caught(function fellBackOnNothing() {
          assertWordingCoherent({
            wording: {
              sliceIndex: 2,
              incumbentKind: 'absent',
              incumbentText: '',
              outcome: { kind: 'incumbent-fallback', },
            },
          },);
        },);

        expect(refusalOfFellBackOnNothing,).toBeInstanceOf(WordingCoherenceError,);
        expect((refusalOfFellBackOnNothing as Error).message,)
          .toBe('slice 2 reports the archive\'s wording standing by default, and the archive holds none',);
      },
    },),
    it({
      name:
        'REFUSES a missing passage the archive translates, the mirror of the same rule, because every count '
          + 'of untranslated passages would inherit a slice the archive covers',
      fn: async () => {
        /**
         What missingWhatExists raised, read for its class as well as its wording.
         */
        const refusalOfMissingWhatExists = caught(function missingWhatExists() {
          assertWordingCoherent({
            wording: {
              sliceIndex: 3,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              outcome: { kind: 'unfilled', },
            },
          },);
        },);

        expect(refusalOfMissingWhatExists,).toBeInstanceOf(WordingCoherenceError,);
        expect((refusalOfMissingWhatExists as Error).message,)
          .toBe('slice 3 reports a missing passage, and the archive holds wording for it',);
      },
    },),
    it({
      name:
        'REFUSES a lane reporting nothing to work on where the archive holds wording, since the only slice '
          + 'a lane can have no input at is one the archive never translated: declining one it did translate '
          + 'is a choice, and reporting it as a structural fact hides the choice',
      fn: async () => {
        /**
         What declinedWhatExists raised, read for its class as well as its wording.
         */
        const refusalOfDeclinedWhatExists = caught(function declinedWhatExists() {
          assertWordingCoherent({
            wording: {
              sliceIndex: 6,
              incumbentKind: 'present',
              incumbentText: ARCHIVE_NAP,
              outcome: { kind: 'not-applicable', },
            },
          },);
        },);

        expect(refusalOfDeclinedWhatExists,).toBeInstanceOf(WordingCoherenceError,);
        expect((refusalOfDeclinedWhatExists as Error).message,)
          .toBe('slice 6 reports this lane having nothing to work on, and the archive holds wording there',);
      },
    },),
    it({
      name:
        'REFUSES wording beside an archive reported as holding none, whatever the outcome, since a place '
          + 'the archive never translated carries no text by construction and wording there came from another '
          + 'preparation',
      fn: async () => {
        /**
         What wordingBesideNothing raised, read for its class as well as its wording.
         */
        const refusalOfWordingBesideNothing = caught(function wordingBesideNothing() {
          assertWordingCoherent({
            wording: {
              sliceIndex: 7,
              incumbentKind: 'absent',
              incumbentText: ARCHIVE_NAP,
              outcome: { kind: 'not-evaluated', },
            },
          },);
        },);

        expect(refusalOfWordingBesideNothing,).toBeInstanceOf(WordingCoherenceError,);
        expect((refusalOfWordingBesideNothing as Error).message,).toBe(
          'slice 7 says the archive holds no wording and carries some anyway, so the two sides of this row '
            + 'were built from different preparations',
        );
      },
    },),
    it({
      name:
        'REFUSES a decision of empty wording at a place the archive never translated, which fills nothing '
          + 'and leaves the passage exactly as it was, so recording it as a decision credits the lane with '
          + 'work indistinguishable from the gap it started with',
      fn: async () => {
        /**
         What decidedNothing raised, read for its class as well as its wording.
         */
        const refusalOfDecidedNothing = caught(function decidedNothing() {
          assertWordingCoherent({
            wording: {
              sliceIndex: 4,
              incumbentKind: 'absent',
              incumbentText: '',
              outcome: {
                kind: 'decided',
                acceptedText: '',
              },
            },
          },);
        },);

        expect(refusalOfDecidedNothing,).toBeInstanceOf(WordingCoherenceError,);
        expect((refusalOfDecidedNothing as Error).message,).toBe(
          'slice 4 reports a decision of empty wording where the archive holds none, so nothing was filled '
            + 'and nothing distinguishes it from a passage still missing',
        );
      },
    },),
    it({
      name:
        'REFUSES a decision whose wording shows a reader nothing at a place the archive never translated, as '
          + 'it refuses an empty one: spaces, a zero-width space, a Hangul filler, or both among spaces fill no '
          + 'passage either',
      fn: async () => {
        /**
         Refusal of a decision of the given wording at an untranslated place.

         @param acceptedText - wording the lane decided on

         @returns Refusal, class and message

         @example
         ```ts
         const refusal = refusalOf('\u{200B}',);
         ```
         */
        function refusalOf(acceptedText: string,): string {
          return String(caught(function decidedNothingVisible() {
            assertWordingCoherent({
              wording: {
                sliceIndex: 4,
                incumbentKind: 'absent',
                incumbentText: '',
                outcome: {
                  kind: 'decided',
                  acceptedText,
                },
              },
            },);
          },),);
        }
        /**
         The refusal an empty decision gets, whose wording its own case pins.
         */
        const empty = refusalOf('',);
        expect(empty.startsWith('WordingCoherenceError: slice 4 reports a decision of empty wording',),).toBe(true,);
        expect([
          '   ',
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
        ].map(refusalOf,),).toEqual([
          empty,
          empty,
          empty,
          empty,
        ],);
      },
    },),
    it({
      name:
        'still accepts a decision of empty wording where the archive HAS wording, since deleting a passage '
          + 'is a decision somebody took and the delivery ledger has to be able to say so',
      fn: async () => {
        assertWordingCoherent({
          wording: {
            sliceIndex: 5,
            incumbentKind: 'present',
            incumbentText: ARCHIVE_NAP,
            outcome: {
              kind: 'decided',
              acceptedText: '',
            },
          },
        },);
      },
    },),
  ],
},);
