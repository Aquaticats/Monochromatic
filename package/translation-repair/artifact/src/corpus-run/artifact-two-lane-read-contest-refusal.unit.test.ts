/**
 Tests for what the contest reader refuses, which is the reason it re-derives
 anything at all.

 SPLIT FROM THE ACCEPTANCE CASES on the line budget. A stored verdict is a
 claim about ballots the same record carries, and a stored contest is a claim
 about which slices the two lanes worded differently; each refusal here is one
 of those claims caught disagreeing with what it describes.

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
  type ArtifactComparisonRow,
  ArtifactParseError,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  parseLaneSelection,
  SLICE_SPELLED_KEYS,
} from '../../dist/final/node/index.mjs';
import { ARCHIVE_NAP, catRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-comparison-row.test-fixture.ts';
import { insertionRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/sole-lane-comparison-row.test-fixture.ts';

/**
 Wording the repair lane left.
 */
const REPAIR_NAP = 'The cat naps in the bookshop attic.';

/**
 Wording the translate lane left, differing from both.
 */
const TRANSLATE_NAP = 'The cat dozes in the attic of the bookshop.';

/**
 Path every message under test is built against.
 */
const SELECTION_PATH = 'CatEntry1.laneSelection';

/**
 Ballot backing the translate lane, in the recorded rather than typed form.
 */
const FOR_TRANSLATE = {
  choice: 'translate',
  unsupported: [],
  unsupportedRaw: [],
  dropped: [],
  droppedRaw: [],
  reason: 'reads more naturally and says the same thing',
};

/**
 Slice both lanes worded differently, which a contest may answer.
 */
const CONTESTED_ROW = catRow({
  sliceIndex: 0,
  repairText: REPAIR_NAP,
  translateText: TRANSLATE_NAP,
},);

/**
 Comparison whose one slice the two lanes worded differently.
 */
const ONE_CONTESTED: readonly ArtifactComparisonRow[] = [CONTESTED_ROW,];

/**
 Comparison whose second slice the two lanes worded identically, so a contest
 naming it is naming a slice with nothing to choose between.
 */
const ONE_CONTESTED_ONE_AGREED: readonly ArtifactComparisonRow[] = [
  CONTESTED_ROW,
  catRow({
    sliceIndex: 1,
    repairText: ARCHIVE_NAP,
    translateText: ARCHIVE_NAP,
  },),
];

/**
 Reads a selection whose one slice carries whatever a test hands it.

 @param slice - recorded slice, valid or not

 @param comparison - rows the contest is checked against

 @returns Nothing a caller uses; every case here expects a throw

 @example
 ```ts
 readOneSlice({ slice, comparison: ONE_CONTESTED, },);
 ```
 */
function readOneSlice(
  {
    slice,
    comparison,
  }: {
    readonly slice: unknown;
    readonly comparison: readonly ArtifactComparisonRow[];
  },
): void {
  parseLaneSelection({
    value: {
      kind: 'contested',
      slices: [slice,],
    },
    comparison,
    path: SELECTION_PATH,
    keys: SLICE_SPELLED_KEYS,
  },);
}

/**
 Generation that records a sole-lane verdict at an archive-absent insertion.
 */
const SOLE_LANE_GENERATION = 15;

/**
 Generation before a sole-lane verdict existed.
 */
const ROSTER_ONLY_GENERATION = 14;

/**
 Reads a selection whose one slice carries whatever a test hands it, in the
 generation the test names.

 @param slice - recorded slice, valid or not

 @param comparison - rows the contest is checked against

 @param generation - artifact generation the file records

 @returns Nothing a caller uses; every case here expects a throw or an answer

 @example
 ```ts
 readOneSliceInGeneration({ slice, comparison, generation: SOLE_LANE_GENERATION, },);
 ```
 */
function readOneSliceInGeneration(
  {
    slice,
    comparison,
    generation,
  }: {
    readonly slice: unknown;
    readonly comparison: readonly ArtifactComparisonRow[];
    readonly generation: 14 | 15;
  },
): void {
  parseLaneSelection({
    value: {
      kind: 'contested',
      slices: [slice,],
    },
    comparison,
    path: SELECTION_PATH,
    keys: SLICE_SPELLED_KEYS,
    generation,
  },);
}

await describe({
  name: `${parseLaneSelection.name} refusals`,
  children: [
    it({
      name:
        'REFUSES a verdict naming a lane its own ballots did not back, since a stored winner nobody '
          + 're-derives is a field that can quietly become a lie',
      fn: async () => {
        /**
         What readOneSlice refused with, read for class as well as wording.
         */
        const refusalOfReadOneSlice = caught(() => {
          readOneSlice({
            slice: {
              sliceIndex: 0,
              verdict: {
                kind: 'lane-won',
                lane: 'repair',
              },
              ballots: [
                FOR_TRANSLATE,
                FOR_TRANSLATE,
              ],
              usable: 2,
            },
            comparison: ONE_CONTESTED,
          },);
        },);

        expect(refusalOfReadOneSlice,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfReadOneSlice as Error).message,).toContain('lane-won:translate, which is what these ballots settle on, rather than lane-won:repair',);
      },
    },),
    it({
      name:
        'REFUSES a refusal its ballots contradict, so a slice recorded as shipping neither lane '
          + 'cannot carry a roster that chose one',
      fn: async () => {
        /**
         What readOneSlice refused with, read for class as well as wording.
         */
        const refusalOfReadOneSlice = caught(() => {
          readOneSlice({
            slice: {
              sliceIndex: 0,
              verdict: { kind: 'settled-neither', },
              ballots: [
                FOR_TRANSLATE,
                FOR_TRANSLATE,
              ],
              usable: 2,
            },
            comparison: ONE_CONTESTED,
          },);
        },);

        expect(refusalOfReadOneSlice,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfReadOneSlice as Error).message,).toContain('rather than settled-neither',);
      },
    },),
    it({
      name:
        'REFUSES a usable count the ballots do not support, since the quorum is measured against '
          + 'that number and a raised one turns an unheard roster into a verdict',
      fn: async () => {
        /**
         What readOneSlice refused with, read for class as well as wording.
         */
        const refusalOfReadOneSlice = caught(() => {
          readOneSlice({
            slice: {
              sliceIndex: 0,
              verdict: { kind: 'settled-neither', },
              ballots: [FOR_TRANSLATE,],
              usable: 2,
            },
            comparison: ONE_CONTESTED,
          },);
        },);

        expect(refusalOfReadOneSlice,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfReadOneSlice as Error).message,).toContain('1, which is how many ballots this slice carries, rather than 2',);
      },
    },),
    it({
      name:
        'REFUSES a contest answering a slice where the two lanes left the same wording, which has no '
          + 'question to ask and therefore no answer to record',
      fn: async () => {
        /**
         What parseLaneSelection refused with, read for class as well as wording.
         */
        const refusalOfParseLaneSelection = caught(() => {
          parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [
                {
                  sliceIndex: 1,
                  verdict: { kind: 'quorum-not-met', },
                  ballots: [],
                  usable: 0,
                },
              ],
            },
            comparison: ONE_CONTESTED_ONE_AGREED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
          },);
        },);

        expect(refusalOfParseLaneSelection,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfParseLaneSelection as Error).message,).toContain('slices [0], which are the ones where the two lanes differ, rather than [1]',);
      },
    },),
    it({
      name:
        'REFUSES a contest leaving an eligible slice unanswered, since a silent gap reads as a slice '
          + 'nobody had to decide rather than as one the record dropped',
      fn: async () => {
        /**
         What parseLaneSelection refused with, read for class as well as wording.
         */
        const refusalOfParseLaneSelection = caught(() => {
          parseLaneSelection({
            value: {
              kind: 'contested',
              slices: [],
            },
            comparison: ONE_CONTESTED,
            path: SELECTION_PATH,
            keys: SLICE_SPELLED_KEYS,
          },);
        },);

        expect(refusalOfParseLaneSelection,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfParseLaneSelection as Error).message,).toContain('slices [0], which are the ones where the two lanes differ, rather than []',);
      },
    },),
    it({
      name: 'REFUSES a won verdict that does not say which lane won, rather than reading the omission as a refusal',
      fn: async () => {
        /**
         What readOneSlice refused with, read for class as well as wording.
         */
        const refusalOfReadOneSlice = caught(() => {
          readOneSlice({
            slice: {
              sliceIndex: 0,
              verdict: { kind: 'lane-won', },
              ballots: [
                FOR_TRANSLATE,
                FOR_TRANSLATE,
              ],
              usable: 2,
            },
            comparison: ONE_CONTESTED,
          },);
        },);

        expect(refusalOfReadOneSlice,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfReadOneSlice as Error).message,).toContain(`${SELECTION_PATH}.slices[0].verdict`,);
      },
    },),
    it({
      name: 'REFUSES a ballot naming a candidate that is neither lane nor the refusal',
      fn: async () => {
        /**
         What readOneSlice refused with, read for class as well as wording.
         */
        const refusalOfReadOneSlice = caught(() => {
          readOneSlice({
            slice: {
              sliceIndex: 0,
              verdict: { kind: 'quorum-not-met', },
              ballots: [
                {
                  ...FOR_TRANSLATE,
                  choice: 'archive',
                },
              ],
              usable: 1,
            },
            comparison: ONE_CONTESTED,
          },);
        },);

        expect(refusalOfReadOneSlice,).toBeInstanceOf(ArtifactParseError,);
        expect((refusalOfReadOneSlice as Error).message,).toContain(`${SELECTION_PATH}.slices[0].ballots[0].choice`,);
      },
    },),
    it({
      name:
        'REFUSES A SOLE-LANE VERDICT in a generation that predates it, since that file asked the roster '
          + 'at every insertion and a sole-lane record in it has no roster verdict behind it',
      fn: async () => {
        const refusal = caught(function act(): void {
          readOneSliceInGeneration({
            slice: {
              sliceIndex: 0,
              verdict: {
                kind: 'sole-lane',
                lane: 'translate',
              },
              ballots: [],
              usable: 0,
            },
            comparison: [insertionRow({
              sliceIndex: 0,
              translateText: TRANSLATE_NAP,
            },),],
            generation: ROSTER_ONLY_GENERATION,
          },);
        },);
        expect(String(refusal,),).toBe(
          "ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].verdict.lane: expected no key here beyond kind.",
        );
      },
    },),
    it({
      name:
        'REFUSES A SOLE-LANE VERDICT on a slice both lanes worded, since a slice with two applicable lanes '
          + 'is a question for the roster and no lane alone applies to it',
      fn: async () => {
        const refusal = caught(function act(): void {
          readOneSliceInGeneration({
            slice: {
              sliceIndex: 0,
              verdict: {
                kind: 'sole-lane',
                lane: 'translate',
              },
              ballots: [],
              usable: 0,
            },
            comparison: ONE_CONTESTED,
            generation: SOLE_LANE_GENERATION,
          },);
        },);
        expect(String(refusal,),).toBe(
          "ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].verdict.lane: expected no key here beyond kind.",
        );
      },
    },),
    it({
      name:
        'REFUSES A ROSTER VERDICT at an archive-absent insertion in a generation that records sole-lane, '
          + 'since the one lane that applies needs no vote and the roster was never to be asked',
      fn: async () => {
        const refusal = caught(function act(): void {
          readOneSliceInGeneration({
            slice: {
              sliceIndex: 0,
              verdict: {
                kind: 'lane-won',
                lane: 'translate',
              },
              ballots: [],
              usable: 0,
            },
            comparison: [insertionRow({
              sliceIndex: 0,
              translateText: TRANSLATE_NAP,
            },),],
            generation: SOLE_LANE_GENERATION,
          },);
        },);
        expect(String(refusal,),).toBe(
          "ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].verdict: expected sole-lane:translate, which is the one lane the comparison leaves applicable, rather than lane-won:translate.",
        );
      },
    },),
    it({
      name:
        'REFUSES BALLOTS CARRIED BY A SOLE-LANE VERDICT, since a slice no roster judged holds none, '
          + 'and a stored ballot would be a vote nobody cast on that slice',
      fn: async () => {
        const refusal = caught(function act(): void {
          readOneSliceInGeneration({
            slice: {
              sliceIndex: 0,
              verdict: {
                kind: 'sole-lane',
                lane: 'translate',
              },
              ballots: [FOR_TRANSLATE,],
              usable: 1,
            },
            comparison: [insertionRow({
              sliceIndex: 0,
              translateText: TRANSLATE_NAP,
            },),],
            generation: SOLE_LANE_GENERATION,
          },);
        },);
        expect(String(refusal,),).toBe(
          "ArtifactParseError: artifact parse failed at CatEntry1.laneSelection.slices[0].ballots: expected no ballots, since no roster was asked about a slice one lane applies to.",
        );
      },
    },),
  ],
},);
