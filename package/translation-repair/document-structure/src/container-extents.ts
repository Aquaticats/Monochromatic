import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  type DocumentNode,
  hashContent,
} from './document-node.ts';
import type { ContainerSpan, } from './unwrap-container.ts';

//region Container extents
// A block's extent used to be its mdast position and nothing else, and that is
// the whole defect: a container's opening and closing tags belong to no
// node, so every range minted from a first node's start and a last node's end
// drew its boundary straight through them. TWO independent sites mint such
// ranges, section chunking in `chunk-document.ts` and slice carving in
// `slice-pair.ts`, and a repair at either one leaves the other cutting tags.
// Measured on the corpus: eight refused entries are cut by the slice site and
// three (`SevenBird`, `mikaela_khara`, `XingZ60`) by the chunk site, whose
// closing tag lands in the gap between two chunks.
//
// So the correction lands where the package first writes an extent rather than
// at either consumer. A block carrying a container's opening tag OWNS that
// tag, and a block carrying its closing tag owns that one. Every consumer
// reading a start and an end inherits the correction without ever learning
// that containers exist.
//
// This also DISSOLVES the atomicity requirement rather than implementing it.
// When a container's blocks fall in different slices, the first slice carries
// the opening tag inside its own text and the last carries the closing tag
// inside its own, so each tag is wholly held by some slice and neither is half
// held. Keeping a container's blocks together stops being necessary.
//
// Safe because it was measured first, not because it reads safe: across both
// sides of the whole corpus there are 53 containers and not one whose tag
// region overlaps any block node, so widening cannot reach a neighbour's text.

/**
 Half-open range a block owns within its document.

 @example
 ```ts
 const extent: BlockExtent = { startOffset: 370, endOffset: 408, };
 ```
 */
export type BlockExtent = {
  /**
   Absolute start offset of owned range.
   */
  readonly startOffset: number;

  /**
   Absolute end offset (exclusive) of owned range.
   */
  readonly endOffset: number;
};

/**
 Which block indices a container's tags attach to.

 @example
 ```ts
 const bound: ContainerBound = { first: 1, last: 2, openerStartOffset: 370, closerEndOffset: 4045, };
 ```
 */
type ContainerBound = {
  /**
   Index of first block lying inside container, which takes opening tag.
   */
  readonly first: number;

  /**
   Index of last block lying inside container, which takes closing tag.
   */
  readonly last: number;

  /**
   Offset opening tag begins at.
   */
  readonly openerStartOffset: number;

  /**
   Offset closing tag ends at.
   */
  readonly closerEndOffset: number;
};

/**
 Locates blocks a container's tags attach to, by containment rather than by
 order, so nesting needs no separate handling: an inner container's bound is
 computed against the same unwidened extents as its outer one.

 Returns nothing for a container holding no blocks at all. Its tags then keep
 belonging to no block, which is safe on its own terms: no slice can reach
 them either, so assembly copies the region through unedited.

 @param extents - unwidened block extents in document order

 @param container - container whose tags need an owner

 @returns Bound naming owning block indices, or nothing when container is empty

 @example
 ```ts
 const bound = interiorBound({ extents, container, },);
 ```
 */
function interiorBound(
  {
    extents,
    container,
  }: {
    readonly extents: readonly BlockExtent[];
    readonly container: ContainerSpan;
  },
): readonly ContainerBound[] {
  /**
   Indices of every block lying wholly between this container's two tags.
   */
  const inside = extents
    .flatMap(function toInsideIndex(
      extent,
      index,
    ): readonly number[] {
      return ((extent.startOffset >= container.openerEndOffset)
        && (extent.endOffset <= container.closerStartOffset))
        ? [index,]
        : [];
    },);

  /**
   First such block, absent when container holds no blocks.
   */
  const [first,] = inside;

  /**
   Last such block, absent for the same reason.
   */
  const last = inside.at(-1,);
  if ((first === undefined) || (last === undefined))
    return [];
  return [
    {
      first,
      last,
      openerStartOffset: container.openerStartOffset,
      closerEndOffset: container.closerEndOffset,
    },
  ];
}

/**
 Widens block extents so each container's tags fall inside a block.

 Nesting composes without a special case, because a block takes the SMALLEST
 opening offset among containers that open at it and the LARGEST closing
 offset among those that close at it. Where an outer container's opener
 region abuts an inner one's, the two regions tile and the minimum swallows
 both.

 @param extents - unwidened block extents in document order

 @param containers - container spans in the same offset frame as extents

 @returns Extents in the same order, each widened over tags it owns

 @example
 ```ts
 const owned = widenExtentsToContainers({ extents, containers, },);
 ```
 */
export function widenExtentsToContainers(
  {
    extents,
    containers,
  }: {
    readonly extents: readonly BlockExtent[];
    readonly containers: readonly ContainerSpan[];
  },
): readonly BlockExtent[] {
  /**
   Every container that found an owning block, paired with those indices.
   */
  const bounds = containers.flatMap(function toBound(container,): readonly ContainerBound[] {
    return interiorBound({
      extents,
      container,
    },);
  },);
  return extents.map(function widenOne(
    extent,
    index,
  ): BlockExtent {
    /**
     Opening offsets of every container this block opens.
     */
    const opens = bounds
      .filter(function opensHere(bound,): boolean {
        return bound.first === index;
      },)
      .map(function toOpener(bound,): number {
        return bound.openerStartOffset;
      },);

    /**
     Closing offsets of every container this block closes.
     */
    const closes = bounds
      .filter(function closesHere(bound,): boolean {
        return bound.last === index;
      },)
      .map(function toCloser(bound,): number {
        return bound.closerEndOffset;
      },);
    return {
      startOffset: Math.min(
        extent.startOffset,
        ...opens,
      ),
      endOffset: Math.max(
        extent.endOffset,
        ...closes,
      ),
    };
  },);
}

/**
 Rewrites document nodes so each owns the container tags it carries.

 Text and hash are recomputed rather than carried over, because a node's text
 is defined as the exact slice its offsets name and that invariant has to
 survive the widening rather than be excused from it.

 @param nodes - document nodes carrying absolute offsets

 @param text - full document source both offsets and slices index

 @param containers - container spans in absolute offsets

 @returns Nodes in the same order, each spanning tags it owns

 @throws {@link Error} when widening returns a different node count than it received

 @example
 ```ts
 const owning = widenNodesToContainers({ nodes, text, containers, },);
 ```
 */
export function widenNodesToContainers(
  {
    nodes,
    text,
    containers,
  }: {
    readonly nodes: readonly DocumentNode[];
    readonly text: string;
    readonly containers: readonly ContainerSpan[];
  },
): readonly DocumentNode[] {
  /**
   Widened extents positionally matching the nodes they came from.
   */
  const widened = widenExtentsToContainers({
    extents: nodes.map(function toExtent(node,): BlockExtent {
      return {
        startOffset: node.startOffset,
        endOffset: node.endOffset,
      };
    },),
    containers,
  },);
  return nodes.map(function toOwningNode(
    node,
    index,
  ): DocumentNode {
    /**
     This node's widened extent, positional by construction.
     */
    const extent = nonNullishOrThrow(widened[index],);
    if ((extent.startOffset === node.startOffset)
      && (extent.endOffset === node.endOffset))
      return node;

    /**
     Exact slice the widened extent names, keeping text and offsets in step.
     */
    const owned = text.slice(
      extent.startOffset,
      extent.endOffset,
    );
    return {
      ...node,
      text: owned,
      startOffset: extent.startOffset,
      endOffset: extent.endOffset,
      contentHash: hashContent({ content: owned, },),
    };
  },);
}

//endregion Container extents
