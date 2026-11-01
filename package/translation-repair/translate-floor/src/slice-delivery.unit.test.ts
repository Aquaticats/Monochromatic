/**
 Tests for the per-slice delivery ledger.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  buildSliceDelivery,
  type LaneSliceText,
  SliceDeliveryError,
  type SliceDeliveryFault,
} from '../dist/final/node/index.mjs';

/**
 Archive wording of each fixture slice, in document order.
 */
const INCUMBENTS = [
  'The cat sleeps.',
  'The cat eats.',
  'The cat watches the birds.',
] as const;

/**
 Original wording of each fixture slice.
 */
const SOURCES = [
  '猫猫在睡觉。',
  '猫猫在吃饭。',
  '猫猫在看鸟。',
] as const;

/**
 Prepared slice pairs shaped as the preparation produces them.
 */
function preparedSlices(): readonly {
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
}[] {
  return INCUMBENTS.map(function toSlice(
    incumbentText,
    sliceIndex,
  ) {
    /**
     Original of this slice, present for every fixture index.
     */
    const sourceText = SOURCES[sliceIndex] ?? '';
    return {
      source: {
        sliceIndex,
        nodes: [],
        startOffset: 0,
        endOffset: sourceText.length,
        text: sourceText,
      },
      target: {
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
 Lane wordings for the fixture slices, with the decisions a case needs.

 @param decided - accepted wording keyed by slice index; a slice absent from
 the map is one the lane never reached

 @returns Wordings in document order

 @example
 ```ts
 const wordings = laneWordings({ decided: new Map([[0, 'The cat naps.',],],), },);
 ```
 */
function laneWordings(
  { decided, }: { readonly decided: ReadonlyMap<number, string>; },
): readonly LaneSliceText[] {
  return INCUMBENTS.map(function toWording(
    incumbentText,
    sliceIndex,
  ): LaneSliceText {
    /**
     What this case says the lane decided here.
     */
    const accepted = decided.get(sliceIndex,);
    return {
      sliceIndex,
      incumbentKind: 'present',
      incumbentText,
      outcome: (accepted === undefined)
        ? { kind: 'not-evaluated', }
        : {
          kind: 'decided',
          acceptedText: accepted,
        },
    };
  },);
}

/**
 Wordings where every slice was examined and left exactly as it was.

 @returns Map from slice index to the archive's own wording

 @example
 ```ts
 const wordings = laneWordings({ decided: everySliceUnchanged(), },);
 ```
 */
function everySliceUnchanged(): ReadonlyMap<number, string> {
  return new Map(INCUMBENTS.map(function toEntry(
    incumbentText,
    sliceIndex,
  ): readonly [number, string,] {
    return [
      sliceIndex,
      incumbentText,
    ];
  },),);
}

/**
 Fixture slices whose middle one is a place rather than existing text.

 @returns Prepared pairs with an anchor at index one

 @example
 ```ts
 const slices = anchoredSlices();
 ```
 */
function anchoredSlices(): readonly ChunkPair[] {
  return preparedSlices().map(function toAnchored(
    slice,
    sliceIndex,
  ): ChunkPair {
    return (sliceIndex === 1)
      ? {
        source: slice.source,
        target: makeInsertionChunk({
          sliceIndex,
          offset: 0,
        },),
      }
      : slice;
  },);
}

/**
 Lane wordings for {@link anchoredSlices}, whose anchor holds no archive
 wording to agree with.

 @param anchorNotApplicable - whether the lane had no work to do at the anchor,
 as against having tried there and produced nothing

 @returns Wordings in document order

 @example
 ```ts
 const wordings = anchoredWordings({ anchorNotApplicable: false, },);
 ```
 */
function anchoredWordings(
  { anchorNotApplicable, }: { readonly anchorNotApplicable: boolean; },
): readonly LaneSliceText[] {
  return [
    {
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: INCUMBENTS[0],
      outcome: {
        kind: 'decided',
        acceptedText: INCUMBENTS[0],
      },
    },
    {
      sliceIndex: 1,
      incumbentKind: 'absent',
      incumbentText: '',
      // The two honest things a lane can say about a passage the archive never
      // translated: it tried and produced nothing, or its work does not apply
      // here at all. Deciding the blank was a third, and it said the lane chose
      // the wording it found, which at an anchor is no wording.
      outcome: anchorNotApplicable
        ? { kind: 'not-applicable', }
        : { kind: 'unfilled', },
    },
    {
      sliceIndex: 2,
      incumbentKind: 'present',
      incumbentText: INCUMBENTS[2],
      outcome: {
        kind: 'decided',
        acceptedText: INCUMBENTS[2],
      },
    },
  ];
}

/**
 Reads the fault a ledger build's refusal names, after checking it is a
 delivery refusal at all.

 THE FAULT RATHER THAN A FRAGMENT OF THE MESSAGE, since every check here throws
 the same class and a fragment can match more than one sentence.

 @param build - call that should refuse

 @returns Fault the refusal carries

 @example
 ```ts
 const fault = deliveryFault({ build: function build() { buildSliceDelivery({ ... },); }, },);
 ```
 */
function deliveryFault({ build, }: { readonly build: () => void; },): SliceDeliveryFault {
  /**
   What the build threw.
   */
  const refusal = caught(build,);
  expect(refusal,).toBeInstanceOf(SliceDeliveryError,);
  return (refusal as SliceDeliveryError).fault;
}

await describe({
  name: buildSliceDelivery.name,
  children: [
    it({
      name: 'CARRIES the assembly guard\'s trimmed text on a shipped row beside the untrimmed decision, since '
        + 'the document carries the decision with an orphan definition cut and the row says what the document '
        + 'carries (the twentieth hakureico pass of 2026-09-09 stopped at the reassembly invariant otherwise)',
      fn: async () => {
        /**
         What the judges chose, two notes behind the sentence.
         */
        const decided = 'The cat is asleep.\n\n[^1]: A cat note.\n\n[^2]: Nothing points here.';

        /**
         What the document carries after the guard's trim.
         */
        const carried = 'The cat is asleep.\n\n[^1]: A cat note.';
        const ledger = buildSliceDelivery({
          slices: preparedSlices(),
          wordings: laneWordings({ decided: new Map([[0, decided,],],), },),
          changedSliceIndices: [0,],
          withdrawnSliceIndices: [],
          trimmedReplacements: [{ sliceIndex: 0, replacementText: carried, },],
          blocked: false,
        },);
        expect(ledger[0]?.shippedText,).toBe(carried,);
        expect(ledger[0]?.delivery,).toEqual({ kind: 'replacement-shipped', },);
        expect(ledger[0]?.outcome,).toEqual({ kind: 'decided', acceptedText: decided, },);
      },
    },),
    it({
      name: 'REFUSES a trimmed replacement naming a slice the document does not ship',
      fn: async () => {
        expect(deliveryFault({
          build: function trimsUnshipped() {
            buildSliceDelivery({
              slices: preparedSlices(),
              wordings: laneWordings({ decided: new Map([[0, 'The cat is asleep.',],],), },),
              changedSliceIndices: [0,],
              withdrawnSliceIndices: [],
              trimmedReplacements: [{ sliceIndex: 1, replacementText: 'The cat eats well.', },],
              blocked: false,
            },);
          },
        },),).toEqual({
          kind: 'trim-names-unshipped',
          sliceIndex: 1,
        },);
      },
    },),
    it({
      name: 'REFUSES two trimmed replacements naming one slice, which keying them by slice silently forgave: the '
        + 'ledger would report the later text as what the document carries, and nothing here knows which the '
        + 'assembly wrote',
      fn: async () => {
        /**
         What trimsOneSliceTwice raised, read for its class and its sentence.
         */
        const refusal = caught(function trimsOneSliceTwice() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({ decided: new Map([[0, 'The cat is asleep.',],],), },),
            changedSliceIndices: [0,],
            withdrawnSliceIndices: [],
            trimmedReplacements: [
              { sliceIndex: 0, replacementText: 'The cat is asleep', },
              { sliceIndex: 0, replacementText: 'The cat dozes', },
            ],
            blocked: false,
          },);
        },);

        expect(refusal,).toBeInstanceOf(SliceDeliveryError,);
        expect(String(refusal,),).toBe(
          'SliceDeliveryError: slice 0 carries more than one trimmed replacement, so which text the document '
            + 'holds there cannot be read',
        );
      },
    },),
    it({
      name: 'reads a slice the archive never translated as a GAP THAT REMAINS, whether the lane tried '
        + 'and could not fill it or had no work to do there at all. Both neighbours read falsely: one says '
        + 'the document carries the archive`s own wording, of which there is none, and the other says '
        + 'nobody looked',
      fn: async () => {
        /** Ledger over an anchor the lane reached and could not fill. */
        const undecided = buildSliceDelivery({
          slices: anchoredSlices(),
          wordings: anchoredWordings({ anchorNotApplicable: false, },),
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          blocked: false,
        },);
        expect(undecided[1]?.delivery
          .kind,).toBe('gap-remains',);

        /** Same anchor, where this lane had nothing to work on at all. */
        const agreed = buildSliceDelivery({
          slices: anchoredSlices(),
          wordings: anchoredWordings({ anchorNotApplicable: true, },),
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          blocked: false,
        },);
        expect(agreed[1]?.delivery
          .kind,).toBe('gap-remains',);
        // Every content slice still reads exactly as it did: this changes what
        // an ANCHOR means and nothing else.
        expect(agreed[0]?.delivery
          .kind,).toBe('incumbent-retained',);
        expect(agreed[2]?.delivery
          .kind,).toBe('incumbent-retained',);
      },
    },),
    it({
      name: 'names every fate a slice can meet in one pass: a shipped replacement, one the assembly '
        + 'guard took back, a slice the lane examined and left alone, and the source beside each, which '
        + 'is the field a grader cannot recover from an artifact at all',
      fn: async () => {
        /** Ledger over one shipped slice, one withdrawn, one left alone. */
        const ledger = buildSliceDelivery({
          slices: preparedSlices(),
          wordings: laneWordings({
            decided: new Map([
              [0, 'The cat is asleep.',],
              [1, 'The cat is eating.',],
              [2, INCUMBENTS[2],],
            ],),
          },),
          changedSliceIndices: [0,],
          withdrawnSliceIndices: [1,],
          blocked: false,
        },);
        expect(ledger.map(function toShipment(record,): string {
          return record.delivery
            .kind;
        },),).toEqual([
          'replacement-shipped',
          'replacement-withdrawn',
          'incumbent-retained',
        ],);
        expect(ledger.map(function toShipped(record,): string {
          return record.shippedText;
        },),).toEqual([
          'The cat is asleep.',
          INCUMBENTS[1],
          INCUMBENTS[2],
        ],);
        expect(ledger.map(function toSource(record,): string {
          return record.sourceText;
        },),).toEqual([...SOURCES,],);
        expect(ledger[1]?.delivery,).toEqual({
          kind: 'replacement-withdrawn',
          reason: 'assembly-integrity',
        },);
      },
    },),
    it({
      name: 'records a slice the lane never reached as NOT EVALUATED rather than as one it left alone. '
        + 'Both carry the archive wording, and only one of them means anybody looked: the repair lane '
        + 'stops at the earliest dominance crossing, so the slices after it were never examined',
      fn: async () => {
        /** Ledger over a lane that stopped after its first slice. */
        const ledger = buildSliceDelivery({
          slices: preparedSlices(),
          wordings: laneWordings({
            decided: new Map([[0, INCUMBENTS[0],],],),
          },),
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          blocked: true,
        },);
        // WHAT THE LANE DID, which is the axis that separates these slices.
        expect(ledger.map(function toOutcome(record,): string {
          return record.outcome
            .kind;
        },),).toEqual([
          'decided',
          'not-evaluated',
          'not-evaluated',
        ],);
        // And what the DOCUMENT carries, which is the same for all three: the
        // archive's own wording, whether anyone looked at it or not. One word
        // could not hold both of these, which is why there are two.
        expect(ledger.map(function toDelivery(record,): string {
          return record.delivery
            .kind;
        },),).toEqual([
          'incumbent-retained',
          'incumbent-retained',
          'incumbent-retained',
        ],);
      },
    },),
    it({
      name: 'calls a decided slice on a BLOCKED run withdrawn, naming the block rather than the assembly '
        + 'guard. That exit never reaches assembly: it returns the archive whatever any slice decided, '
        + 'so the two withdrawals are different events and a reader counting integrity damage would '
        + 'otherwise count a refusal as one',
      fn: async () => {
        /** Ledger over a blocked run whose first slice had decided a repair. */
        const ledger = buildSliceDelivery({
          slices: preparedSlices(),
          wordings: laneWordings({
            decided: new Map([[0, 'The cat is asleep.',],],),
          },),
          changedSliceIndices: [],
          withdrawnSliceIndices: [],
          blocked: true,
        },);
        expect(ledger[0]?.delivery,).toEqual({
          kind: 'replacement-withdrawn',
          reason: 'blocked-non-translation',
        },);
        expect(ledger[0]?.shippedText,).toBe(INCUMBENTS[0],);
        expect(ledger[0]?.outcome,).toEqual({
          kind: 'decided',
          acceptedText: 'The cat is asleep.',
        },);
      },
    },),
    it({
      name: 'REFUSES a decided slice that an unblocked run names as neither shipped nor withdrawn, which '
        + 'is the state where nothing says what the document carries there',
      fn: async () => {
        /**
         What unstatedSlice raised, read for its class and the fault it names.
         */
        const refusalOfUnstatedSlice = caught(function unstatedSlice() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({
              decided: new Map([
                [0, 'The cat is asleep.',],
                [1, INCUMBENTS[1],],
                [2, INCUMBENTS[2],],
              ],),
            },),
            changedSliceIndices: [],
            withdrawnSliceIndices: [],
            blocked: false,
          },);
        },);

        expect(refusalOfUnstatedSlice,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfUnstatedSlice as SliceDeliveryError).fault,).toEqual({
          kind: 'decided-unstated',
          sliceIndex: 0,
        },);
      },
    },),
    it({
      name: 'REFUSES a shipped slice whose decision is the archive wording, and a slice named as shipped '
        + 'that the lane never reached: each says the document carries a change nobody made',
      fn: async () => {
        /**
         What shippedWithoutChange raised, read for its class and the fault it names.
         */
        const refusalOfShippedWithoutChange = caught(function shippedWithoutChange() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({ decided: everySliceUnchanged(), },),
            changedSliceIndices: [0,],
            withdrawnSliceIndices: [],
            blocked: false,
          },);
        },);

        expect(refusalOfShippedWithoutChange,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfShippedWithoutChange as SliceDeliveryError).fault,).toEqual({
          kind: 'shipped-archive-wording',
          sliceIndex: 0,
        },);
        /**
         What shippedWithoutDecision raised, read for its class and the fault it names.
         */
        const refusalOfShippedWithoutDecision = caught(function shippedWithoutDecision() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({
              decided: new Map([
                [1, INCUMBENTS[1],],
                [2, INCUMBENTS[2],],
              ],),
            },),
            changedSliceIndices: [0,],
            withdrawnSliceIndices: [],
            blocked: false,
          },);
        },);

        expect(refusalOfShippedWithoutDecision,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfShippedWithoutDecision as SliceDeliveryError).fault,).toEqual({
          kind: 'named-without-decision',
          sliceIndex: 0,
          set: 'shipped',
        },);
        // THE OTHER SET, named on a slice the lane never reached: the refusal
        // names which set did it.
        expect(deliveryFault({
          build: function withdrawnWithoutDecision() {
            buildSliceDelivery({
              slices: preparedSlices(),
              wordings: laneWordings({
                decided: new Map([
                  [0, INCUMBENTS[0],],
                  [2, INCUMBENTS[2],],
                ],),
              },),
              changedSliceIndices: [],
              withdrawnSliceIndices: [1,],
              blocked: false,
            },);
          },
        },),).toEqual({
          kind: 'named-without-decision',
          sliceIndex: 1,
          set: 'withdrawn',
        },);
      },
    },),
    it({
      name: 'REFUSES reports built from a different preparation: a short wording list, an index outside '
        + 'the prepared slices, and archive wording the two sides disagree about. Each would join one '
        + 'lane`s slice against another`s while the two name different passages',
      fn: async () => {
        /**
         What shortWordings raised, read for its class and the fault it names.
         */
        const refusalOfShortWordings = caught(function shortWordings() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({ decided: everySliceUnchanged(), },)
              .slice(
              0,
              2,
              ),
            changedSliceIndices: [],
            withdrawnSliceIndices: [],
            blocked: false,
          },);
        },);

        expect(refusalOfShortWordings,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfShortWordings as SliceDeliveryError).fault,).toEqual({
          kind: 'wording-count',
          wordings: 2,
          slices: 3,
        },);
        /**
         What outOfRangeIndex raised, read for its class and the fault it names.
         */
        const refusalOfOutOfRangeIndex = caught(function outOfRangeIndex() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({ decided: everySliceUnchanged(), },),
            changedSliceIndices: [7,],
            withdrawnSliceIndices: [],
            blocked: false,
          },);
        },);

        expect(refusalOfOutOfRangeIndex,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfOutOfRangeIndex as SliceDeliveryError).fault,).toEqual({
          kind: 'set-names-unproduced',
          sliceIndex: 7,
          sliceCount: 3,
        },);

        /** Wordings whose archive text was taken from another document. */
        const drifted = laneWordings({ decided: everySliceUnchanged(), },)
          .map(function toDrifted(
            wording,
            position,
          ): LaneSliceText {
            return (position === 1)
              ? {
                ...wording,
                incumbentText: 'The cat dines.',
                outcome: {
                  kind: 'decided',
                  acceptedText: 'The cat dines.',
                },
              }
              : wording;
          },);
        expect(deliveryFault({
          build: function driftedIncumbent() {
            buildSliceDelivery({
              slices: preparedSlices(),
              wordings: drifted,
              changedSliceIndices: [],
              withdrawnSliceIndices: [],
              blocked: false,
            },);
          },
        },),).toEqual({
          kind: 'archive-wording-differs',
          sliceIndex: 1,
        },);

        /** Wordings whose second names the third slice. */
        const shifted = laneWordings({ decided: everySliceUnchanged(), },)
          .map(function toShifted(
            wording,
            position,
          ): LaneSliceText {
            return (position === 1)
              ? {
                ...wording,
                sliceIndex: 2,
              }
              : wording;
          },);
        expect(deliveryFault({
          build: function shiftedIndex() {
            buildSliceDelivery({
              slices: preparedSlices(),
              wordings: shifted,
              changedSliceIndices: [],
              withdrawnSliceIndices: [],
              blocked: false,
            },);
          },
        },),).toEqual({
          kind: 'wording-index-differs',
          position: 1,
          sliceIndex: 1,
          wordingIndex: 2,
        },);
      },
    },),
    it({
      name: 'REFUSES a record that calls the archive present at an anchor, whose prepared chunk names a place and '
        + 'carries no wording, and words it plainly: the lane record and the preparation disagree about whether '
        + 'the archive translated that passage at all',
      fn: async () => {
        /**
         Anchored wordings whose anchor claims archive wording, with the same
         empty text the anchor carries, so only the kind disagrees.
         */
        const claimed = anchoredWordings({ anchorNotApplicable: false, },)
          .map(function toClaimed(
            wording,
            position,
          ): LaneSliceText {
            return (position === 1)
              ? {
                ...wording,
                incumbentKind: 'present',
              }
              : wording;
          },);

        /**
         What the build raised.
         */
        const refusal = caught(function claimsArchiveAtAnchor() {
          buildSliceDelivery({
            slices: anchoredSlices(),
            wordings: claimed,
            changedSliceIndices: [],
            withdrawnSliceIndices: [],
            blocked: false,
          },);
        },);
        expect(refusal,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusal as SliceDeliveryError).fault,).toEqual({
          kind: 'incumbent-kind-differs',
          sliceIndex: 1,
          recorded: 'present',
        },);
        expect((refusal as SliceDeliveryError).message,).toBe(
          'slice 1\'s lane record says archive wording is present there, and its prepared chunk says the opposite',
        );
      },
    },),

    it({
      name: 'REFUSES an index set that names one slice twice, which building a '
        + 'set out of it silently forgave: a lane counting one change as two '
        + 'has two derivations disagreeing about its own document, and neither '
        + 'the count nor the ledger showed it',
      fn: async () => {
        /**
         What shippedTwice raised, read for its class and the fault it names.
         */
        const refusalOfShippedTwice = caught(function shippedTwice() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({
              decided: new Map([[
                0,
                'The cat naps.',
              ],],),
            },),
            changedSliceIndices: [
              0,
              0,
            ],
            withdrawnSliceIndices: [],
            blocked: false,
          },);
        },);

        expect(refusalOfShippedTwice,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfShippedTwice as SliceDeliveryError).fault,).toEqual({
          kind: 'set-repeats',
          set: 'shipped',
          named: 2,
          distinct: 1,
        },);
        /**
         What withdrawnTwice raised, read for its class and the fault it names.
         */
        const refusalOfWithdrawnTwice = caught(function withdrawnTwice() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({
              decided: new Map([[
                1,
                'The cat dines.',
              ],],),
            },),
            changedSliceIndices: [],
            withdrawnSliceIndices: [
              1,
              1,
            ],
            blocked: false,
          },);
        },);

        expect(refusalOfWithdrawnTwice,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfWithdrawnTwice as SliceDeliveryError).fault,).toEqual({
          kind: 'set-repeats',
          set: 'withdrawn',
          named: 2,
          distinct: 1,
        },);
      },
    },),

    it({
      name: 'REFUSES a slice named as both shipped and withdrawn rather than '
        + 'letting the branch order answer it, which reported a change assembly '
        + 'had taken back as one the document carries',
      fn: async () => {
        /**
         What shippedAndWithdrawn raised, read for its class and the fault it names.
         */
        const refusalOfShippedAndWithdrawn = caught(function shippedAndWithdrawn() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({
              decided: new Map([[
                2,
                'The cat studies the birds.',
              ],],),
            },),
            changedSliceIndices: [2,],
            withdrawnSliceIndices: [2,],
            blocked: false,
          },);
        },);

        expect(refusalOfShippedAndWithdrawn,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfShippedAndWithdrawn as SliceDeliveryError).fault,).toEqual({
          kind: 'both-shipped-and-withdrawn',
          sliceIndex: 2,
        },);
      },
    },),

    it({
      name: 'REFUSES a withdrawal of a slice the lane left at the archive '
        + 'wording, since assembly cannot take back a replacement nobody wrote '
        + 'and the row would read as a lane overruled rather than one that left '
        + 'the slice alone',
      fn: async () => {
        /**
         What withdrewNothing raised, read for its class and the fault it names.
         */
        const refusalOfWithdrewNothing = caught(function withdrewNothing() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({ decided: everySliceUnchanged(), },),
            changedSliceIndices: [],
            withdrawnSliceIndices: [1,],
            blocked: false,
          },);
        },);

        expect(refusalOfWithdrewNothing,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfWithdrewNothing as SliceDeliveryError).fault,).toEqual({
          kind: 'withdrawn-archive-wording',
          sliceIndex: 1,
        },);
      },
    },),

    it({
      name: 'REFUSES a slice named as withdrawn BY ASSEMBLY on a blocked run, because that exit returns '
        + 'the archive without assembling anything: the withdrawal it reports is the block itself, and '
        + 'those are the two events a reader counting integrity damage has to tell apart',
      fn: async () => {
        /**
         What withdrewWhileBlocked raised, read for its class and the fault it names.
         */
        const refusalOfWithdrewWhileBlocked = caught(function withdrewWhileBlocked() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({
              decided: new Map([[0, 'The cat is asleep.',],],),
            },),
            changedSliceIndices: [],
            withdrawnSliceIndices: [0,],
            blocked: true,
          },);
        },);

        expect(refusalOfWithdrewWhileBlocked,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfWithdrewWhileBlocked as SliceDeliveryError).fault,).toEqual({
          kind: 'withdrawn-on-blocked',
          sliceIndex: 0,
        },);
      },
    },),

    it({
      name: 'REFUSES a shipped slice on a BLOCKED run, because that exit returns the archive document '
        + 'whatever any slice decided: a shipped index there names a replacement no reader can have '
        + 'seen, and the ledger would report the run delivering work it explicitly refused to deliver',
      fn: async () => {
        /**
         What shippedWhileBlocked raised, read for its class and the fault it names.
         */
        const refusalOfShippedWhileBlocked = caught(function shippedWhileBlocked() {
          buildSliceDelivery({
            slices: preparedSlices(),
            wordings: laneWordings({
              decided: new Map([[0, 'The cat is asleep.',],],),
            },),
            changedSliceIndices: [0,],
            withdrawnSliceIndices: [],
            blocked: true,
          },);
        },);

        expect(refusalOfShippedWhileBlocked,).toBeInstanceOf(SliceDeliveryError,);
        expect((refusalOfShippedWhileBlocked as SliceDeliveryError).fault,).toEqual({
          kind: 'shipped-on-blocked',
          sliceIndex: 0,
        },);
      },
    },),
    it({
      name: 'WORDS its refusal from the fault alone, as a marked class the boundary may print',
      fn: async () => {
        const error = new SliceDeliveryError({
          fault: {
            kind: 'wording-index-differs',
            position: 1,
            sliceIndex: 1,
            wordingIndex: 2,
          },
        },);
        expect(error.message,).toBe('slice at position 1 is indexed 1 and its wording names slice 2',);
        expect(error.messageNamesOnly,).toBe(true,);
      },
    },),
  ],
},);
