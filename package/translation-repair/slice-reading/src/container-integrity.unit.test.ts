/**
 Tests for the check that every container tag is owned whole by the block and
 range that reach it.

 A dissolved container leaves its opening and closing tags belonging to no
 block, and every range here is minted from block offsets, so a boundary could
 fall between an opener and its closer while satisfying every node-level rule.
 `container-extents.ts` fixes that by handing each tag to the block beside it;
 this check asks whether that still happened, and is expected to fire only on
 a regression in how extents or ranges are derived.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  makeInsertionChunk,
  parseDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ContainerSpan,
  DocumentNode,
} from '@monochromatic-dev/translation-repair-document-structure';
import {
  assertContainerIntegrity,
  type ChunkPair,
  ContainerIntegrityError,
} from '../dist/final/node/index.mjs';

/**
 Translation packaging two of its four blocks inside a disclosure element, so
 the element's tags are handed to the outer two of those blocks.
 */
const TARGET_TEXT = `The cat sleeps on the windowsill.

<details>

She watches the birds outside.

Later she eats.

</details>

She purrs at nothing in particular.
`;

/**
 Parsed fixture, read once so blocks and containers describe the same parse.
 */
const TARGET = parseDocument({ text: TARGET_TEXT, },);

/**
 Blocks of that translation, each already owning any tag it carries.
 */
const TARGET_NODES = TARGET.nodes;

/**
 Container spans the parse reported, which is the whole point of the fixture.
 */
const TARGET_CONTAINERS = TARGET.containers;

/**
 Single container of the fixture, which every case is stated against.

 @returns Its two tag spans

 @throws {@link Error} when the fixture parsed no container, which would make
 every case here pass for the wrong reason

 @example
 ```ts
 const container = onlyContainer();
 ```
 */
function onlyContainer(): ContainerSpan {
  const [container,] = TARGET_CONTAINERS;
  if ((container === undefined) || (TARGET_CONTAINERS.length !== 1))
    throw new Error(`fixture reported ${String(TARGET_CONTAINERS.length,)} containers, expected one`,);
  return container;
}

/**
 Finds the block that owns one offset, which is how each case names a slice
 boundary without hard-coding a number the fixture could drift away from.

 @param offset - absolute offset the wanted block covers

 @returns Block covering that offset

 @throws {@link Error} when no block covers it

 @example
 ```ts
 const node = blockAt({ offset: container.openerStartOffset, },);
 ```
 */
function blockAt({ offset, }: { readonly offset: number; },): DocumentNode {
  /**
   First block whose range holds the offset.
   */
  const found = TARGET_NODES.find(function holds(node,): boolean {
    return (node.startOffset <= offset)
      && (node.endOffset > offset);
  },);
  if (found === undefined)
    throw new Error(`fixture has no block covering offset ${String(offset,)}`,);
  return found;
}

/**
 Builds one pair whose target side covers the given range.

 @param startOffset - absolute start of range assembly would replace

 @param endOffset - absolute exclusive end

 @returns Pair carrying that span

 @example
 ```ts
 const pair = rangeOf({ startOffset: 0, endOffset: 20, },);
 ```
 */
function rangeOf(
  {
    startOffset,
    endOffset,
  }: {
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
      nodes: TARGET_NODES.filter(function inRange(node,): boolean {
        return (node.startOffset >= startOffset)
          && (node.endOffset <= endOffset);
      },),
      startOffset,
      endOffset,
      text: TARGET_TEXT.slice(
        startOffset,
        endOffset,
      ),
    },
  };
}

/**
 Rebuilds the fixture's blocks as they looked BEFORE tags were handed out, so
 one case can state the regression this check exists to catch.

 @returns Blocks whose edges stop at the container's tags instead of covering them

 @example
 ```ts
 const orphaning = blocksWithoutTags();
 ```
 */
function blocksWithoutTags(): readonly DocumentNode[] {
  /**
   Container whose tags are about to be taken back off the blocks.
   */
  const container = onlyContainer();
  return TARGET_NODES.map(function shrink(node,): DocumentNode {
    return {
      ...node,
      startOffset: (node.startOffset === container.openerStartOffset)
        ? container.openerEndOffset
        : node.startOffset,
      endOffset: (node.endOffset === container.closerEndOffset)
        ? container.closerStartOffset
        : node.endOffset,
    };
  },);
}

/**
 Translation whose container is a bare fragment, which carries no element name.
 */
const FRAGMENT_TEXT = 'The cat naps.\n\n<>\n\nShe dreams.\n\n</>\n\nShe wakes.\n';

/**
 Parsed fragment fixture, read once so its blocks and container describe the same parse.
 */
const FRAGMENT = parseDocument({ text: FRAGMENT_TEXT, },);

await describe({
  name: assertContainerIntegrity.name,
  children: [
    it({
      name: 'ACCEPTS a container whose blocks fall in DIFFERENT slices, which the earlier rule '
        + 'refused: each tag rides inside its own slice text, so both lanes see the tag they must '
        + 'reproduce and the page stays balanced',
      fn: async () => {
        /** Container whose two tags this pair of slices will separate. */
        const container = onlyContainer();

        /** Block owning the opening tag, which ends the first slice. */
        const opening = blockAt({ offset: container.openerStartOffset, },);

        /** Block owning the closing tag, which starts the second. */
        const closing = blockAt({ offset: container.closerStartOffset, },);
        expect(function checkSplitContainer(): void {
          assertContainerIntegrity({
            slices: [
              rangeOf({
                startOffset: 0,
                endOffset: opening.endOffset,
              },),
              rangeOf({
                startOffset: closing.startOffset,
                endOffset: TARGET_TEXT.length,
              },),
            ],
            containers: TARGET_CONTAINERS,
            blocks: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'ACCEPTS a range holding BOTH tags, which is the ordinary healthy case: the slice text '
        + 'carries them, so a candidate dropping them is refused later by the page grammar rather '
        + 'than here',
      fn: async () => {
        expect(function checkWholeContainer(): void {
          assertContainerIntegrity({
            slices: [
              rangeOf({
                startOffset: 0,
                endOffset: TARGET_TEXT.length,
              },),
            ],
            containers: TARGET_CONTAINERS,
            blocks: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'REFUSES blocks that stop at a container tag instead of covering it, which is the '
        + 'regression the widening exists to prevent: with tags in no block, a range boundary can '
        + 'fall between an opener and its closer and assembly deletes one of them',
      fn: async () => {
        expect(function checkOrphanedTags(): void {
          assertContainerIntegrity({
            slices: [
              rangeOf({
                startOffset: 0,
                endOffset: TARGET_TEXT.length,
              },),
            ],
            containers: TARGET_CONTAINERS,
            blocks: blocksWithoutTags(),
          },);
        },).toThrow(ContainerIntegrityError,);
      },
    },),
    it({
      name: 'REFUSES a range ending PART WAY THROUGH a tag, which neither holds it nor leaves it '
        + 'alone: assembly would write over half of it and leave the rest',
      fn: async () => {
        /** Container whose opening tag the range will cut. */
        const container = onlyContainer();
        expect(function checkHalfTag(): void {
          assertContainerIntegrity({
            slices: [
              rangeOf({
                startOffset: 0,
                endOffset: container.openerStartOffset + 2,
              },),
            ],
            containers: TARGET_CONTAINERS,
            blocks: TARGET_NODES,
          },);
        },).toThrow(ContainerIntegrityError,);
      },
    },),
    it({
      name: 'ACCEPTS a range holding NEITHER tag, so a slice landing wholly outside a container is '
        + 'left alone rather than refused for being near one',
      fn: async () => {
        /** Container the range will stay clear of. */
        const container = onlyContainer();
        expect(function checkOutsideContainer(): void {
          assertContainerIntegrity({
            slices: [
              rangeOf({
                startOffset: 0,
                endOffset: container.openerStartOffset,
              },),
            ],
            containers: TARGET_CONTAINERS,
            blocks: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'says nothing about a page carrying NO container, so the check costs nothing on the '
        + 'pages it was not written for',
      fn: async () => {
        expect(function checkNoContainers(): void {
          assertContainerIntegrity({
            slices: [
              rangeOf({
                startOffset: 0,
                endOffset: TARGET_TEXT.length,
              },),
            ],
            containers: [],
            blocks: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'says nothing about an ANCHOR, which replaces no range and so can cut no tag: where an '
        + 'insertion lands inside a container is a placement question answered elsewhere',
      fn: async () => {
        expect(function checkAnchor(): void {
          assertContainerIntegrity({
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
                  offset: nonNullishOrThrow(onlyContainer()
                    .openerEndOffset,),
                },),
              },
            ],
            containers: TARGET_CONTAINERS,
            blocks: TARGET_NODES,
          },);
        },).not.toThrow();
      },
    },),
    it({
      name: 'NAMES A FRAGMENT AS "a fragment" where no block owns its opening tag, since a bare fragment tag has no '
        + 'element name to say',
      fn: async () => {
        /**
         Container the fixture's one fragment parsed to.
         */
        const fragment = nonNullishOrThrow(FRAGMENT.containers[0],);
        /**
         Blocks of the fragment page with the one holding the opening tag stopping short of it.
         */
        const orphaned = FRAGMENT.nodes
          .map(function shrink(node,): DocumentNode {
            return {
              ...node,
              startOffset: (node.startOffset === fragment.openerStartOffset)
                ? fragment.openerEndOffset
                : node.startOffset,
            };
          },);
        /**
         What the check refused with.
         */
        const refusal = caught(function checkOrphanedFragment(): unknown {
          return assertContainerIntegrity({
            slices: [],
            containers: FRAGMENT.containers,
            blocks: orphaned,
          },);
        },);
        expect(refusal,).toBeInstanceOf(ContainerIntegrityError,);
        expect(String(refusal,),).toBe(
          'ContainerIntegrityError: no block owns the opening tag of a fragment, though the container holds blocks: '
            + 'extents were derived without widening onto container tags, so a range boundary can fall between '
            + 'this tag and its partner',
        );
      },
    },),
    it({
      name: 'REFUSES a block that starts part way through an opening tag, naming the element and the half '
        + 'tag the block would carry',
      fn: async () => {
        /**
         Container whose opening tag the block will start inside.
         */
        const container = onlyContainer();
        /**
         Blocks of the fixture with the one owning the opening tag starting two characters into it.
         */
        const halfOpened = TARGET_NODES.map(function cutInto(node,): DocumentNode {
          return (node.startOffset === container.openerStartOffset)
            ? {
              ...node,
              startOffset: container.openerStartOffset + 2,
            }
            : node;
        },);
        /**
         What the check refused with.
         */
        const refusal = caught(function checkHalfOpened(): unknown {
          return assertContainerIntegrity({
            slices: [],
            containers: TARGET_CONTAINERS,
            blocks: halfOpened,
          },);
        },);
        expect(refusal,).toBeInstanceOf(ContainerIntegrityError,);
        expect(String(refusal,),).toBe(
          'ContainerIntegrityError: a block covers part of the opening tag of <details> without covering all of it, '
            + 'so every range minted from that block would carry half a tag',
        );
      },
    },),
  ],
},);
