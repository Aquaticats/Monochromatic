/**
 Tests for the translate lane's per-slice wordings.

 WHAT THESE PIN is the fix this file was written for: a stage that heard NO
 TRANSLATOR keeps the archive's wording as its output text, and passing that
 through as a decision states that the lane examined the passage and chose the
 archive. It did not. Nobody answered. Every lane comparison run before this
 counted those slices as agreement with the archive, which is the window
 trial's lost-judge defect wearing the producing stage's clothes.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import { LaneSliceCoverageError, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  translateLaneWordings,
  type TranslateSliceRecord,
} from '../dist/final/node/index.mjs';

/**
 Builds one prepared slice pair.

 @param index - global slice index both sides carry

 @param target - archive translation of this slice

 @returns Pair shaped as preparation produces

 @example
 ```ts
 const pair = pairOf({ index: 0, target: 'The cat naps.', },);
 ```
 */
function pairOf(
  {
    index,
    target,
  }: {
    readonly index: number;
    readonly target: string;
  },
): {
  readonly source: {
    readonly sliceIndex: number;
    readonly nodes: readonly never[];
    readonly startOffset: number;
    readonly endOffset: number;
    readonly text: string;
  };
  readonly target: {
    readonly sliceIndex: number;
    readonly nodes: readonly never[];
    readonly startOffset: number;
    readonly endOffset: number;
    readonly text: string;
  };
} {
  return {
    source: {
      sliceIndex: index,
      nodes: [],
      startOffset: 0,
      endOffset: 1,
      text: 'source of the nap',
    },
    target: {
      sliceIndex: index,
      nodes: [],
      startOffset: 0,
      endOffset: target.length,
      text: target,
    },
  };
}

/**
 Builds one settled record with the parts this builder reads.

 @param sliceIndex - slice this record settles

 @param outputText - what the driver accepted for assembly, which for an
 unheard slice is the archive's own wording rather than anything produced

 @param heardTranslators - how many voices the producing stage actually heard,
 which is the whole question here

 @returns Record shaped as the driver settles one

 @example
 ```ts
 const record = recordFor({ sliceIndex: 0, outputText: 'The cat naps.', heardTranslators: 2, },);
 ```
 */
function recordFor(
  {
    sliceIndex,
    outputText,
    heardTranslators,
  }: {
    readonly sliceIndex: number;
    readonly outputText: string;
    readonly heardTranslators: number;
  },
): TranslateSliceRecord {
  return {
    kind: 'translate-slice',
    schemaVersion: 1,
    sliceIndex,
    outputText,
    changed: false,
    disposition: 'stage-result',
    findings: [],
    stageResult: {
      text: outputText,
      origin: 'fresh',
      decision: 'judged',
      voteWeight: 1,
      ballots: [],
      heardTranslators,
      candidateCount: heardTranslators,
      slate: [],
      perCandidate: [],
      findings: [],
    },
  } as unknown as TranslateSliceRecord;
}

/**
 Two prepared slices, both of which the archive already translates.
 */
const CAT_SLICES = [
  pairOf({
    index: 0,
    target: 'The cat sleeps on the sill.',
  },),
  pairOf({
    index: 1,
    target: 'The bowl is full.',
  },),
];

await describe({
  name: translateLaneWordings.name,
  children: [
    it({
      name:
        'reports a slice whose stage heard NO TRANSLATOR as the archive standing by default rather than '
          + 'as a decision, because the record carries the incumbent as its output text and passing that '
          + 'through says the lane examined the passage and kept it, which is what a lost voice looks like '
          + 'when nothing separates the two',
      fn: async () => {
        /**
         Wordings where the first slice was translated and the second was met
         with silence.
         */
        const wordings = translateLaneWordings({
          slices: CAT_SLICES,
          unfilledChunkIndices: [],
          settled: [
            recordFor({
              sliceIndex: 0,
              outputText: 'The cat is asleep on the windowsill.',
              heardTranslators: 2,
            },),
            recordFor({
              sliceIndex: 1,
              outputText: 'The bowl is full.',
              heardTranslators: 0,
            },),
          ],
        },);
        expect(wordings,).toHaveLength(2,);
        expect(wordings[0]?.outcome,).toEqual({
          kind: 'decided',
          acceptedText: 'The cat is asleep on the windowsill.',
        },);

        // The unheard slice carries NO WORDING AT ALL. `incumbent-fallback`
        // has no accepted text by construction, so no consumer can read one
        // off it and call it a choice.
        expect(wordings[1]?.outcome,).toEqual({ kind: 'incumbent-fallback', },);
      },
    },),
    it({
      name:
        'still reports the archive wording for the unheard slice, on the incumbent side where it belongs, '
          + 'since that is what the assembled document carries there and a comparison needs it to say what '
          + 'stood',
      fn: async () => {
        /**
         Wordings where nobody was heard for either slice.
         */
        const wordings = translateLaneWordings({
          slices: CAT_SLICES,
          unfilledChunkIndices: [],
          settled: [
            recordFor({
              sliceIndex: 0,
              outputText: 'The cat sleeps on the sill.',
              heardTranslators: 0,
            },),
            recordFor({
              sliceIndex: 1,
              outputText: 'The bowl is full.',
              heardTranslators: 0,
            },),
          ],
        },);
        expect(wordings.map(function toIncumbent(one,): string {
          return one.incumbentText;
        },),).toEqual([
          'The cat sleeps on the sill.',
          'The bowl is full.',
        ],);

        // READ OFF THE PREPARATION, not off the records: a resumed run can hold
        // cache values written under an earlier preparation of the same entry.
        expect(wordings.map(function toIncumbentKind(one,): string {
          return one.incumbentKind;
        },),).toEqual(['present', 'present',],);
      },
    },),
    it({
      name:
        'keeps an unfilled passage separate from an unheard one, because only one of them has anything to '
          + 'fall back on: the archive translates the unheard slice and has never translated the unfilled one',
      fn: async () => {
        /**
         One translated slice and one place the archive leaves empty.
         */
        const anchored = [
          CAT_SLICES[0] ?? pairOf({
            index: 0,
            target: 'The cat sleeps on the sill.',
          },),
          {
            source: {
              sliceIndex: 1,
              nodes: [],
              startOffset: 0,
              endOffset: 0,
              text: 'source of the bowl',
            },
            target: makeInsertionChunk({
              sliceIndex: 1,
              offset: 0,
            },),
          },
        ];

        /**
         Wordings where the heard slice went unheard and the anchor went
         unfilled.
         */
        const wordings = translateLaneWordings({
          slices: anchored,
          unfilledChunkIndices: [1,],
          settled: [
            recordFor({
              sliceIndex: 0,
              outputText: 'The cat sleeps on the sill.',
              heardTranslators: 0,
            },),
          ],
        },);
        expect(wordings.map(function toOutcome(one,): string {
          return one.outcome
            .kind;
        },),).toEqual([
          'incumbent-fallback',
          'unfilled',
        ],);
        expect(wordings.map(function toIncumbentKind(one,): string {
          return one.incumbentKind;
        },),).toEqual(['present', 'absent',],);
      },
    },),
    it({
      name:
        'MARKS AN UNFILLED PASSAGE THE SELECT BENCH NEVER BOUGHT A CANDIDATE FOR, so the outcome tells a skip for '
          + 'want of a bench from a panel that declined, and marks only the unfilled slices it is asked about',
      fn: async () => {
        /**
         Two places the archive leaves empty, and a translated slice before them.
         */
        const anchored = [
          CAT_SLICES[0] ?? pairOf({
            index: 0,
            target: 'The cat sleeps on the sill.',
          },),
          ...[1, 2,].map(function insertionAt(sliceIndex,) {
            return {
              source: {
                sliceIndex,
                nodes: [],
                startOffset: 0,
                endOffset: 0,
                text: `source of the bowl ${String(sliceIndex,)}`,
              },
              target: makeInsertionChunk({
                sliceIndex,
                offset: 0,
              },),
            };
          },),
        ];
        /**
         Wordings with slice 1 marked and slice 2 an ordinary decline.
         */
        const wordings = translateLaneWordings({
          slices: anchored,
          unfilledChunkIndices: [1, 2,],
          belowFloorChunkIndices: [1, 0,],
          settled: [
            recordFor({
              sliceIndex: 0,
              outputText: 'The cat sleeps on the sill.',
              heardTranslators: 2,
            },),
          ],
        },);
        expect(wordings.map(function toOutcome(one,) {
          return one.outcome;
        },),).toEqual([
          {
            kind: 'decided',
            acceptedText: 'The cat sleeps on the sill.',
          },
          {
            kind: 'unfilled',
            benchBelowFloor: true,
          },
          { kind: 'unfilled', },
        ],);
      },
    },),
    it({
      name:
        'REFUSES a prepared slice no record and no unfilled index accounts for, since this lane visits every '
          + 'slice by contract and a gap it never named is a lost slice rather than an early stop',
      fn: async () => {
        /**
         Failure the builder raised.
         */
        let caught: unknown;
        try {
          translateLaneWordings({
            slices: CAT_SLICES,
            unfilledChunkIndices: [],
            settled: [
              recordFor({
                sliceIndex: 0,
                outputText: 'The cat is asleep on the windowsill.',
                heardTranslators: 2,
              },),
            ],
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(LaneSliceCoverageError,);
        expect(String(caught,),).toContain('1',);
      },
    },),
  ],
},);
