/**
 Tests for the two claims a delivery ledger makes about a document.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  buildSliceDelivery,
  type LaneSliceText,
  type SliceDeliveryRecord,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  assertDeliveryAgreesWithDocument,
  DeliveryInvariantError,
} from '../dist/final/node/index.mjs';

/**
 Archive translation both lanes would write into.
 */
const ARCHIVE = 'The cat sleeps.\n\nShe purrs.\n';

/**
 Where the first paragraph ends, which is also the boundary an anchor names.
 */
const FIRST_END = 'The cat sleeps.'.length;

/**
 Where the second paragraph begins.
 */
const SECOND_START = ARCHIVE.indexOf('She purrs.',);

/**
 Where it ends.
 */
const SECOND_END = SECOND_START + 'She purrs.'.length;

/**
 Builds a pair whose target side covers a span of {@link ARCHIVE}.

 @param sliceIndex - position of this slice

 @param startOffset - absolute start

 @param endOffset - absolute exclusive end

 @param sourceText - original this slice renders

 @returns Pair covering that span

 @example
 ```ts
 const pair = spanAt({ sliceIndex: 0, startOffset: 0, endOffset: 15, sourceText: '猫猫在睡觉。', },);
 ```
 */
function spanAt(
  {
    sliceIndex,
    startOffset,
    endOffset,
    sourceText,
  }: {
    readonly sliceIndex: number;
    readonly startOffset: number;
    readonly endOffset: number;
    readonly sourceText: string;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: sourceText,
    },
    target: {
      sliceIndex,
      nodes: [],
      startOffset,
      endOffset,
      text: ARCHIVE.slice(
        startOffset,
        endOffset,
      ),
    },
  };
}

/**
 Builds a pair whose target side names a boundary and covers nothing.

 @param sliceIndex - position of this slice

 @param offset - boundary the translation belongs at

 @param sourceText - original with no translation in the archive

 @returns Pair anchored at that boundary

 @example
 ```ts
 const pair = anchorAt({ sliceIndex: 1, offset: 15, sourceText: '她伸了个懒腰。', },);
 ```
 */
function anchorAt(
  {
    sliceIndex,
    offset,
    sourceText,
  }: {
    readonly sliceIndex: number;
    readonly offset: number;
    readonly sourceText: string;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: sourceText,
    },
    target: makeInsertionChunk({
      sliceIndex,
      offset,
    },),
  };
}

/**
 Builds a ledger the way a lane driver does.

 @param slices - preparation the lane ran over

 @param wordings - what it decided per slice

 @param shipped - slices its document carries a change for

 @returns One row per prepared slice

 @example
 ```ts
 const ledger = ledgerFor({ slices, wordings, shipped: [0,], },);
 ```
 */
function ledgerFor(
  {
    slices,
    wordings,
    shipped,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly wordings: readonly LaneSliceText[];
    readonly shipped: readonly number[];
  },
): readonly SliceDeliveryRecord[] {
  return buildSliceDelivery({
    slices,
    wordings,
    changedSliceIndices: shipped,
    withdrawnSliceIndices: [],
    blocked: false,
  },);
}

/**
 Two slices covering the archive, the first rewritten and the second left.
 */
const REWRITTEN_SLICES: readonly ChunkPair[] = [
  spanAt({
    sliceIndex: 0,
    startOffset: 0,
    endOffset: FIRST_END,
    sourceText: '猫猫在睡觉。',
  },),
  spanAt({
    sliceIndex: 1,
    startOffset: SECOND_START,
    endOffset: SECOND_END,
    sourceText: '她在呼噜。',
  },),
];

/**
 What that lane decided for them.
 */
const REWRITTEN_WORDINGS: readonly LaneSliceText[] = [
  {
    sliceIndex: 0,
    incumbentKind: 'present',
    incumbentText: 'The cat sleeps.',
    outcome: {
      kind: 'decided',
      acceptedText: 'The cat dozes.',
    },
  },
  {
    sliceIndex: 1,
    incumbentKind: 'present',
    incumbentText: 'She purrs.',
    outcome: {
      kind: 'decided',
      acceptedText: 'She purrs.',
    },
  },
];

await describe({
  name: assertDeliveryAgreesWithDocument.name,
  children: [
    it({
      name: 'accepts a ledger whose shipped rows write the document the lane returned, which is the '
        + 'ordinary case and the positive control every refusal case in this file rests on',
      fn: async () => {
        expect(function checkAgreement(): void {
          assertDeliveryAgreesWithDocument({
            ledger: ledgerFor({
              slices: REWRITTEN_SLICES,
              wordings: REWRITTEN_WORDINGS,
              shipped: [0,],
            },),
            slices: REWRITTEN_SLICES,
            incumbentText: ARCHIVE,
            documentText: 'The cat dozes.\n\nShe purrs.\n',
            changedSliceIndices: [0,],
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'REFUSES a document carrying wording the ledger does not, which is the claim that spans two '
        + 'derivations: a row`s text is what the lane DECIDED and the document is what assembly wrote, '
        + 'and nothing until now made those two say so',
      fn: async () => {
        expect(function checkAgreement(): void {
          assertDeliveryAgreesWithDocument({
            ledger: ledgerFor({
              slices: REWRITTEN_SLICES,
              wordings: REWRITTEN_WORDINGS,
              shipped: [0,],
            },),
            slices: REWRITTEN_SLICES,
            incumbentText: ARCHIVE,
            documentText: 'The cat naps.\n\nShe purrs.\n',
            changedSliceIndices: [0,],
          },);
        },).toThrow(DeliveryInvariantError,);
      },
    },),
    it({
      name: 'REFUSES a result naming a slice the ledger ships nothing for, since a ledger joined to '
        + 'another run reads as a correct one row by row',
      fn: async () => {
        expect(function checkAgreement(): void {
          assertDeliveryAgreesWithDocument({
            ledger: ledgerFor({
              slices: REWRITTEN_SLICES,
              wordings: REWRITTEN_WORDINGS,
              shipped: [0,],
            },),
            slices: REWRITTEN_SLICES,
            incumbentText: ARCHIVE,
            documentText: 'The cat dozes.\n\nShe purrs.\n',
            changedSliceIndices: [
              0,
              1,
            ],
          },);
        },).toThrow(DeliveryInvariantError,);
      },
    },),
    it({
      name: 'REFUSES a ledger shipping a slice the result does not name, the same disagreement from the '
        + 'other side',
      fn: async () => {
        /** Lane that decided a wording for both slices. */
        const wordings: readonly LaneSliceText[] = [
          {
            sliceIndex: 0,
            incumbentKind: 'present',
            incumbentText: 'The cat sleeps.',
            outcome: {
              kind: 'decided',
              acceptedText: 'The cat dozes.',
            },
          },
          {
            sliceIndex: 1,
            incumbentKind: 'present',
            incumbentText: 'She purrs.',
            outcome: {
              kind: 'decided',
              acceptedText: 'She rumbles.',
            },
          },
        ];
        expect(function checkAgreement(): void {
          assertDeliveryAgreesWithDocument({
            ledger: ledgerFor({
              slices: REWRITTEN_SLICES,
              wordings,
              shipped: [
                0,
                1,
              ],
            },),
            slices: REWRITTEN_SLICES,
            incumbentText: ARCHIVE,
            documentText: 'The cat dozes.\n\nShe rumbles.\n',
            changedSliceIndices: [0,],
          },);
        },).toThrow(DeliveryInvariantError,);
      },
    },),
    it({
      name: 'accepts an ANCHORED insertion, whose row carries the rendering and not the blank lines '
        + 'around it. Assembly composes those, so they belong to no slice, and a check comparing row '
        + 'text against the document directly would refuse a document nothing is wrong with',
      fn: async () => {
        /** Archive slices with a boundary between them that has no translation. */
        const slices: readonly ChunkPair[] = [
          spanAt({
            sliceIndex: 0,
            startOffset: 0,
            endOffset: FIRST_END,
            sourceText: '猫猫在睡觉。',
          },),
          anchorAt({
            sliceIndex: 1,
            offset: FIRST_END,
            sourceText: '她伸了个懒腰。',
          },),
          spanAt({
            sliceIndex: 2,
            startOffset: SECOND_START,
            endOffset: SECOND_END,
            sourceText: '她在呼噜。',
          },),
        ];

        /** Lane that translated only the slice with nothing to keep. */
        const wordings: readonly LaneSliceText[] = [
          {
            sliceIndex: 0,
            incumbentKind: 'present',
            incumbentText: 'The cat sleeps.',
            outcome: {
              kind: 'decided',
              acceptedText: 'The cat sleeps.',
            },
          },
          {
            sliceIndex: 1,
            // The anchor, where the archive holds no wording at all. Stamping
            // it `present` is what the ledger's own consistency check refuses,
            // since the prepared chunk says otherwise.
            incumbentKind: 'absent',
            incumbentText: '',
            outcome: {
              kind: 'decided',
              acceptedText: 'She stretches.',
            },
          },
          {
            sliceIndex: 2,
            incumbentKind: 'present',
            incumbentText: 'She purrs.',
            outcome: {
              kind: 'decided',
              acceptedText: 'She purrs.',
            },
          },
        ];
        expect(function checkAgreement(): void {
          assertDeliveryAgreesWithDocument({
            ledger: ledgerFor({
              slices,
              wordings,
              shipped: [1,],
            },),
            slices,
            incumbentText: ARCHIVE,
            documentText: 'The cat sleeps.\n\nShe stretches.\n\nShe purrs.\n',
            changedSliceIndices: [1,],
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'WORDS its refusal from the fault alone, as a marked class the boundary may print',
      fn: async () => {
        const error = new DeliveryInvariantError({
          fault: {
            kind: 'unclaimed',
            indices: [
              2,
              5,
            ],
          },
        },);
        expect(error.message,).toBe(
          'ledger ships slices 2, 5 that the result does not name as changed, so one of the two describes '
            + 'another run',
        );
        expect(error.messageNamesOnly,).toBe(true,);
      },
    },),
  ],
},);
