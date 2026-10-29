/**
 Tests for the repair lane's per-slice wordings.

 WHAT THESE PIN is that this lane's silence at a passage the archive never
 translated is reported as silence. It mends existing English, so where there
 is none it has no work to do and no opinion to record; its settled outcome
 carries the empty string for want of anything else, and passing that through
 as a decision said the lane chose the wording it found. A lane comparison
 then read that against a translate lane that had actually filled the passage
 and reported the two lanes choosing DIFFERENT wordings.

 The second thing they pin is the intersection. The blocked exit settles a
 prefix and stops, so an anchor before the crossing was reached and one after
 it was not, and naming every anchor in the preparation would report the lane
 as having visited slices it never got to.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import { LaneSliceCoverageError, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  repairLaneWordings,
  RepairUnheardError,
  type RepairVoiceRecord,
} from '../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Archive wording of each content slice, by index; the odd ones are anchors.
 */
const INCUMBENTS: Readonly<Record<number, string>> = {
  0: 'The cat sleeps on the sill.',
  2: 'The bowl is full.',
};

/**
 Four prepared slices: content, anchor, content, anchor.

 @returns Pairs shaped as preparation produces them

 @example
 ```ts
 const slices = alternatingSlices();
 ```
 */
function alternatingSlices(): readonly ChunkPair[] {
  return [
    0,
    1,
    2,
    3,
  ].map(function toPair(sliceIndex,): ChunkPair {
    /**
     Archive wording here, absent at every anchor.
     */
    const incumbentText = INCUMBENTS[sliceIndex];
    return {
      source: {
        sliceIndex,
        nodes: [],
        startOffset: 0,
        endOffset: 1,
        text: `source of slice ${String(sliceIndex,)}`,
      },
      target: (incumbentText === undefined)
        ? makeInsertionChunk({
          sliceIndex,
          offset: 0,
        },)
        : {
          sliceIndex,
          nodes: [],
          startOffset: 0,
          endOffset: incumbentText.length,
          text: incumbentText,
        },
    };
  },);
}

/**
 One settled slice a critic answered on, which is what makes it a decision.

 WRITTEN OUT rather than defaulted, because the field it fills is exactly what
 separates a slice the lane examined from one it heard nobody about, and a
 fixture that left it implicit would stop testing the distinction the moment
 the default changed.

 @param sliceIndex - slice this settled

 @param repairedText - wording the lane settled on

 @returns Outcome carrying one heard critic and no refinement

 @example
 ```ts
 const outcome = heard({ sliceIndex: 0, repairedText: 'The cat naps.', },);
 ```
 */
function heard(
  {
    sliceIndex,
    repairedText,
  }: {
    readonly sliceIndex: number;
    readonly repairedText: string;
  },
): RepairVoiceRecord {
  return {
    sliceIndex,
    repairedText,
    changed: false,
    heardCriticIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
    refined: false,
  };
}

/**
 One settled slice no stage spoke about, which leaves the archive standing.

 @param sliceIndex - slice this settled

 @param repairedText - wording the lane settled on, which for a silent slice
 has to be the archive's own

 @returns Outcome carrying no heard critic and no refinement

 @example
 ```ts
 const outcome = unheard({ sliceIndex: 0, repairedText: ARCHIVE_NAP, },);
 ```
 */
function unheard(
  {
    sliceIndex,
    repairedText,
  }: {
    readonly sliceIndex: number;
    readonly repairedText: string;
  },
): RepairVoiceRecord {
  return {
    sliceIndex,
    repairedText,
    changed: false,
    heardCriticIds: [],
    refined: false,
  };
}

await describe({
  name: repairLaneWordings.name,
  children: [
    it({
      name:
        'reports a passage the archive never translated as NOT APPLICABLE rather than as a decision, '
          + 'because this lane repairs existing English and there is none there: its outcome carries the '
          + 'empty string for want of anything else, and calling that a decision credits the lane with '
          + 'choosing a wording where it had no opinion at all',
      fn: async () => {
        /**
         Wordings for a run that visited every slice.
         */
        const wordings = repairLaneWordings({
          slices: alternatingSlices(),
          undecided: 'refuse',
          outcomes: [
            heard({ sliceIndex: 0, repairedText: 'The cat is asleep on the windowsill.', },),
            heard({ sliceIndex: 1, repairedText: '', },),
            heard({ sliceIndex: 2, repairedText: 'The bowl is full.', },),
            heard({ sliceIndex: 3, repairedText: '', },),
          ],
        },);
        expect(wordings.map(function toOutcome(one,): string {
          return one.outcome
            .kind;
        },),).toEqual([
          'decided',
          'not-applicable',
          'decided',
          'not-applicable',
        ],);

        // And the archive side agrees, which is what makes the delivery a gap
        // that remains rather than wording being retained.
        expect(wordings.map(function toIncumbentKind(one,): string {
          return one.incumbentKind;
        },),).toEqual([
          'present',
          'absent',
          'present',
          'absent',
        ],);
      },
    },),
    it({
      name:
        'names only the anchors the run actually REACHED. The blocked exit settles a prefix and stops, so '
          + 'an anchor before the crossing is one this lane looked at and had no work for, and an anchor '
          + 'after it is one nobody got to: naming every anchor in the preparation would report slices as '
          + 'visited that never were',
      fn: async () => {
        /**
         Wordings for a run blocked after the third slice.
         */
        const wordings = repairLaneWordings({
          slices: alternatingSlices(),
          undecided: 'not-evaluated',
          outcomes: [
            heard({ sliceIndex: 0, repairedText: 'The cat is asleep on the windowsill.', },),
            heard({ sliceIndex: 1, repairedText: '', },),
            heard({ sliceIndex: 2, repairedText: 'The bowl is full.', },),
          ],
        },);
        expect(wordings.map(function toOutcome(one,): string {
          return one.outcome
            .kind;
        },),).toEqual([
          'decided',
          'not-applicable',
          'decided',
          'not-evaluated',
        ],);
      },
    },),
    it({
      name:
        'reports a slice NOBODY was heard about as the archive standing by default, not as a decision: '
          + 'with every critic lost and the naturalness lane silent, the lane settled the archive`s own '
          + 'wording because nothing else was available, and calling that a decision credits it with '
          + 'having examined a passage it never saw an answer about',
      fn: async () => {
        /**
         Wordings for a run whose second content slice heard nobody.
         */
        const wordings = repairLaneWordings({
          slices: alternatingSlices(),
          undecided: 'refuse',
          outcomes: [
            heard({ sliceIndex: 0, repairedText: 'The cat is asleep on the windowsill.', },),
            heard({ sliceIndex: 1, repairedText: '', },),
            unheard({ sliceIndex: 2, repairedText: 'The bowl is full.', },),
            heard({ sliceIndex: 3, repairedText: '', },),
          ],
        },);
        expect(wordings.map(function toOutcome(one,): string {
          return one.outcome
            .kind;
        },),).toEqual([
          'decided',
          'not-applicable',
          'incumbent-fallback',
          'not-applicable',
        ],);
      },
    },),
    it({
      name:
        'keeps a slice a critic answered on as a DECISION even where the answer changed nothing, since '
          + 'a lane that examined a passage and left it alone has chosen it: folding that together with a '
          + 'lost stage would make the two indistinguishable in exactly the direction that flatters the run',
      fn: async () => {
        /**
         Wordings where every content slice was examined and kept.
         */
        const wordings = repairLaneWordings({
          slices: alternatingSlices(),
          undecided: 'refuse',
          outcomes: [
            heard({ sliceIndex: 0, repairedText: 'The cat sleeps on the sill.', },),
            heard({ sliceIndex: 1, repairedText: '', },),
            heard({ sliceIndex: 2, repairedText: 'The bowl is full.', },),
            heard({ sliceIndex: 3, repairedText: '', },),
          ],
        },);
        expect(wordings[0]?.outcome
          .kind,).toBe('decided',);
        expect(wordings[2]?.outcome
          .kind,).toBe('decided',);
      },
    },),
    it({
      name:
        'REFUSES a slice nobody was heard about that carries a wording other than the archive`s, because '
          + 'something produced text no stage was recorded as having produced, and a row like that reaches '
          + 'a ledger, a comparison and a rate before anyone notices',
      fn: async () => {
        /**
         What silentSliceCarriesText raised, read for its class as well as its wording.
         */
        const refusalOfSilentSliceCarriesText = caught(function silentSliceCarriesText() {
          repairLaneWordings({
            slices: alternatingSlices(),
            undecided: 'refuse',
            outcomes: [
              heard({ sliceIndex: 0, repairedText: 'The cat is asleep on the windowsill.', },),
              heard({ sliceIndex: 1, repairedText: '', },),
              unheard({ sliceIndex: 2, repairedText: 'The bowl is overflowing.', },),
              heard({ sliceIndex: 3, repairedText: '', },),
            ],
          },);
        },);

        expect(refusalOfSilentSliceCarriesText,).toBeInstanceOf(RepairUnheardError,);
        expect((refusalOfSilentSliceCarriesText as Error).message,).toContain('slice 2 heard no critic',);
      },
    },),
    it({
      name:
        'reports a slice the NATURALNESS lane rewrote as a decision even where no critic answered, '
          + 'because that stage produces wordings too: reading only the critics would report a rewritten '
          + 'passage as the archive standing untouched',
      fn: async () => {
        /**
         Wordings where the refiner acted on a slice no critic raised.
         */
        const wordings = repairLaneWordings({
          slices: alternatingSlices(),
          undecided: 'refuse',
          outcomes: [
            heard({ sliceIndex: 0, repairedText: 'The cat is asleep on the windowsill.', },),
            heard({ sliceIndex: 1, repairedText: '', },),
            {
              ...unheard({ sliceIndex: 2, repairedText: 'The bowl is brimming.', },),
              refined: true,
              changed: true,
            },
            heard({ sliceIndex: 3, repairedText: '', },),
          ],
        },);
        expect(wordings[2]?.outcome
          .kind,).toBe('decided',);
      },
    },),
    it({
      name:
        'REFUSES a slice nobody was heard about that keeps the archive`s own wording and still CLAIMS a '
          + 'change, which the case beside this one cannot reach: there the wording gave the contradiction '
          + 'away, and here the text matches, so the claim is the only thing wrong with the row',
      fn: async () => {
        /**
         What silentSliceClaimsChange raised, read for its class as well as its wording.
         */
        const refusalOfSilentSliceClaimsChange = caught(function silentSliceClaimsChange() {
          repairLaneWordings({
            slices: alternatingSlices(),
            undecided: 'refuse',
            outcomes: [
              heard({ sliceIndex: 0, repairedText: 'The cat is asleep on the windowsill.', },),
              heard({ sliceIndex: 1, repairedText: '', },),
              {
                ...unheard({ sliceIndex: 2, repairedText: 'The bowl is full.', },),
                changed: true,
              },
              heard({ sliceIndex: 3, repairedText: '', },),
            ],
          },);
        },);

        expect(refusalOfSilentSliceClaimsChange,).toBeInstanceOf(RepairUnheardError,);
        expect((refusalOfSilentSliceClaimsChange as Error).message,).toContain(
          'slice 2 heard no critic and was never refined, and claims a change',
        );
      },
    },),
    it({
      name:
        'REFUSES an outcome at a slice preparation never made as a coverage fault, not as a silent '
          + 'slice that changed the archive, since there is no archive there for it to have kept',
      fn: async () => {
        /**
         What an outcome at an unmade slice raised.
         */
        const refusalOfUnmadeSlice = caught(function unmadeSlice() {
          repairLaneWordings({
            slices: alternatingSlices(),
            undecided: 'refuse',
            outcomes: [
              heard({ sliceIndex: 0, repairedText: 'The cat is asleep on the windowsill.', },),
              heard({ sliceIndex: 1, repairedText: '', },),
              heard({ sliceIndex: 2, repairedText: 'The bowl is full.', },),
              heard({ sliceIndex: 3, repairedText: '', },),
              unheard({ sliceIndex: 7, repairedText: 'The kitten waves from the fence.', },),
            ],
          },);
        },);

        expect(refusalOfUnmadeSlice,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(refusalOfUnmadeSlice,).not
          .toBeInstanceOf(RepairUnheardError,);
      },
    },),
  ],
},);
