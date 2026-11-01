/**
 Tests for which side of the window carries which text.

 WHY THIS FILE EXISTS. The fidelity judge is shown two passages either side of
 a slice: the ORIGINAL, which says what the neighbour is about, and the
 ARCHIVE ENGLISH, which is the half that shows a relocation, because the
 Chinese says each thing once in its own place while the English says it next
 door. One record carries both so they provably come from one slice position.

 WHAT WAS MEASURED. On 2026-08-25, swapping the two sides failed no test in
 this package. A judge would then be shown the English under the heading that
 promises the Chinese, which is indistinguishable from an archive that moved a
 passage: exactly the reading the window exists to support, arriving inverted.

 THE SLICE INDICES ARE NOT POSITIONS in the fixture, which pins the second
 half of the same contract: the map is keyed by the index each slice was
 STAMPED with, while the window is read by POSITION in the list.

 Fixtures carry parsed nodes because heading-boundary context uses source
 structure as well as text and stamped indices.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { sliceNeighbourContexts, } from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Builds one prepared slice pair.

 @param sliceIndex - index this slice was stamped with

 @param source - original wording of the passage

 @param target - archive English of the same passage

 @returns Pair shaped as preparation returns one

 @example
 ```ts
 const slice = pairOf({ sliceIndex: 10, source: '猫睡了。', target: 'The cat slept.', },);
 ```
 */
function pairOf(
  {
    sliceIndex,
    source,
    target,
  }: {
    readonly sliceIndex: number;
    readonly source: string;
    readonly target: string;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      startOffset: 0,
      endOffset: source.length,
      text: source,
      nodes: parseDocument({ text: source, },).nodes,
    },
    target: {
      sliceIndex,
      startOffset: 0,
      endOffset: target.length,
      text: target,
      nodes: parseDocument({ text: target, },).nodes,
    },
  };
}

/**
 Three slices, stamped from ten so no index equals its own position.
 */
const SLICES = [
  pairOf({
    sliceIndex: 10,
    source: '小猫在窗台上睡到中午。',
    target: 'Mittens slept on the sill until noon.',
  },),
  pairOf({
    sliceIndex: 11,
    source: '她的哥哥给她带来一根羽毛。',
    target: 'Her brother brought her a feather.',
  },),
  pairOf({
    sliceIndex: 12,
    source: '白胡子数着外面的鸟。',
    target: 'Whiskers counted the birds outside.',
  },),
];

//endregion Fixtures

await describe({
  name: sliceNeighbourContexts.name,
  children: [
    it({
      name: 'KEEPS the original on the source side and the archive English on the incumbent side, since '
        + 'a judge shown them the other way round reads an archive that moved a passage, which is the '
        + 'very finding this window exists to let it make',
      fn: async () => {
        /**
         Window of the middle slice, which has a neighbour each way.
         */
        const beside = sliceNeighbourContexts({ slices: SLICES, },)
          .get(11,);

        expect(beside?.sourceText,).toBe('小猫在窗台上睡到中午。\n\n白胡子数着外面的鸟。',);
        expect(beside?.incumbentText,).toBe(
          'Mittens slept on the sill until noon.\n\nWhiskers counted the birds outside.',
        );
      },
    },),
    it({
      name: 'READS the window by position while keying it by the stamped index, so a slice at the end '
        + 'of a document carries the one neighbour it has rather than an empty window that would report '
        + 'a measured null',
      fn: async () => {
        /**
         Window of the first slice, which has no neighbour before it.
         */
        const beside = sliceNeighbourContexts({ slices: SLICES, },)
          .get(10,);

        expect(beside?.sourceText,).toBe('她的哥哥给她带来一根羽毛。',);
        expect(beside?.incumbentText,).toBe('Her brother brought her a feather.',);
      },
    },),
  ],
},);
