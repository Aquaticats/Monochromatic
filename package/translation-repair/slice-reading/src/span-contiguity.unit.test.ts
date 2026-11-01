/**
 Tests for the check that a span carries every block it covers.

 The failure this exists for is silent by construction: a span's text is
 sliced from its own offsets, so a block missing from its node run is still
 inside the range, still agrees with the text byte for byte, and is replaced
 at assembly by a decision that never saw it.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  makeInsertionChunk,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  assertSpanContiguity,
  type ChunkPair,
  SpanContiguityError,
} from '../dist/final/node/index.mjs';

/**
 Translation whose three paragraphs are three blocks.
 */
const TARGET_TEXT = `The cat sleeps on the windowsill.

She watches the birds outside.

Later she eats.
`;

/**
 Blocks of that translation, in document order.
 */
const TARGET_NODES = parseDocument({ text: TARGET_TEXT, },).nodes;

/**
 Builds one pair whose target side spans the given blocks.

 @param nodes - blocks the span claims to carry

 @param startOffset - absolute start of the range

 @param endOffset - absolute exclusive end

 @returns Pair carrying that span

 @example
 ```ts
 const pair = spanOf({ nodes, startOffset: 0, endOffset: 20, },);
 ```
 */
function spanOf(
  {
    nodes,
    startOffset,
    endOffset,
  }: {
    readonly nodes: typeof TARGET_NODES;
    readonly startOffset: number;
    readonly endOffset: number;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: '猫猫在窗台上睡觉。',
    },
    target: {
      sliceIndex: 0,
      nodes,
      startOffset,
      endOffset,
      text: TARGET_TEXT.slice(
        startOffset,
        endOffset,
      ),
    },
  };
}

await describe({
  name: assertSpanContiguity.name,
  children: [
    it({
      name: 'accepts a span carrying every block between its offsets, which is what consecutive '
        + 'grouping produces and what every slice looks like today',
      fn: async () => {
        /** All three blocks under one span, which is the ordinary shape. */
        const [first,] = TARGET_NODES;

        /** Last block, for the closing offset. */
        const last = TARGET_NODES.at(-1,);
        if ((first === undefined) || (last === undefined))
          throw new Error('fixture parsed no blocks',);
        expect(function checkWholeRange(): void {
          assertSpanContiguity({
            slices: [
              spanOf({
                nodes: TARGET_NODES,
                startOffset: first.startOffset,
                endOffset: last.endOffset,
              },),
            ],
            targetNodes: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'REFUSES a span that SKIPS a block lying inside its own range, which is the trap a run '
        + 'built by filtering falls into: the offsets come from the first and last node kept, so the '
        + 'text contains the dropped block and agrees with the document byte for byte',
      fn: async () => {
        /** First block, whose start opens the range. */
        const [
          first,
          ,
          third,
        ] = TARGET_NODES;
        if ((first === undefined) || (third === undefined))
          throw new Error('fixture parsed fewer than three blocks',);
        expect(function checkFilteredRun(): void {
          assertSpanContiguity({
            slices: [
              spanOf({
                // The middle block is dropped from the run and still covered by
                // the range, which is exactly what nothing else can see.
                nodes: [
                  first,
                  third,
                ],
                startOffset: first.startOffset,
                endOffset: third.endOffset,
              },),
            ],
            targetNodes: TARGET_NODES,
          },);
        },).toThrow(SpanContiguityError,);
      },
    },),
    it({
      name: 'REFUSES a span whose nodes COUNT correctly and name a block from outside its range, since '
        + 'a count alone would pass a slice describing two different passages',
      fn: async () => {
        /** First block, which the range covers. */
        const [
          first,
          second,
          third,
        ] = TARGET_NODES;
        if ((first === undefined) || (second === undefined) || (third === undefined))
          throw new Error('fixture parsed fewer than three blocks',);
        expect(function checkForeignNode(): void {
          assertSpanContiguity({
            slices: [
              spanOf({
                nodes: [
                  first,
                  third,
                ],
                startOffset: first.startOffset,
                endOffset: second.endOffset,
              },),
            ],
            targetNodes: TARGET_NODES,
          },);
        },).toThrow(SpanContiguityError,);
      },
    },),
    it({
      name: 'says nothing about an ANCHOR, which covers no range and carries no blocks: the rule is '
        + 'about what a span claims, and a place claims nothing',
      fn: async () => {
        expect(function checkAnchor(): void {
          assertSpanContiguity({
            slices: [
              {
                source: {
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 0,
                  text: '猫猫也喜欢晒太阳。',
                },
                target: makeInsertionChunk({
                  sliceIndex: 0,
                  offset: TARGET_TEXT.length,
                },),
              },
            ],
            targetNodes: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'REFUSES a span whose range cuts through a block, which counting whole blocks alone could '
        + 'not see: the straddled block is not inside the range, so it was not counted, and a span '
        + 'carrying nothing across half a paragraph agreed with itself',
      fn: async () => {
        /** Second block, whose middle the range will stop in. */
        const [, middle,] = TARGET_NODES;
        if (middle === undefined)
          throw new Error('fixture parsed too few blocks',);

        /** Offset partway through it, which no node boundary sits at. */
        const cut = middle.startOffset + 4;
        expect(function checkCutRange(): void {
          assertSpanContiguity({
            slices: [
              spanOf({
                nodes: [],
                startOffset: middle.startOffset,
                endOffset: cut,
              },),
            ],
            targetNodes: TARGET_NODES,
          },);
        },).toThrow(SpanContiguityError,);
      },
    },),
    it({
      name: 'REFUSES an anchor strictly inside a block, since every legal place for one is a boundary '
        + 'between blocks and the layout check cannot see this: an empty span starts where it ends, so '
        + 'it never overlaps a neighbour however wrong its offset',
      fn: async () => {
        /** First block, whose middle the anchor will name. */
        const [first,] = TARGET_NODES;
        if (first === undefined)
          throw new Error('fixture parsed no blocks',);
        expect(function checkAnchorInsideBlock(): void {
          assertSpanContiguity({
            slices: [
              {
                source: {
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 0,
                  text: '猫猫在窗台上睡觉。',
                },
                target: makeInsertionChunk({
                  sliceIndex: 0,
                  offset: first.startOffset + 4,
                },),
              },
            ],
            targetNodes: TARGET_NODES,
          },);
        },).toThrow(SpanContiguityError,);
      },
    },),
    it({
      name: 'accepts an anchor at a block boundary, which is where every insertion the slicing builds '
        + 'sits: the start of the block it precedes, or the end of the last one',
      fn: async () => {
        /** Second block, whose start is an ordinary insertion point. */
        const [, middle,] = TARGET_NODES;

        /** Last block, whose end is where a trailing insertion goes. */
        const last = TARGET_NODES.at(-1,);
        if ((middle === undefined) || (last === undefined))
          throw new Error('fixture parsed too few blocks',);
        expect(function checkBoundaryAnchors(): void {
          assertSpanContiguity({
            slices: [
              {
                source: {
                  sliceIndex: 0,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 0,
                  text: '猫猫在窗台上睡觉。',
                },
                target: makeInsertionChunk({
                  sliceIndex: 0,
                  offset: middle.startOffset,
                },),
              },
              {
                source: {
                  sliceIndex: 1,
                  nodes: [],
                  startOffset: 0,
                  endOffset: 0,
                  text: '她在看鸟。',
                },
                target: makeInsertionChunk({
                  sliceIndex: 1,
                  offset: last.endOffset,
                },),
              },
            ],
            targetNodes: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
  ],
},);
