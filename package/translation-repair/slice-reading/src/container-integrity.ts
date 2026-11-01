import type { ChunkPair, } from './chunk-document.ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  ContainerSpan,
  DocumentNode,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Container integrity
// What this guards changed once the defect it was built for was fixed at its
// origin, so read the current shape rather than the history.
//
// `flattenContainers` promotes a disclosure element's children to top-level
// blocks so both language sides expose comparable structure. That USED to leave
// the element's own opening and closing tags belonging to no block at all, and
// since every range in this package is minted from block offsets, a slice range
// could hold one tag and stop short of the other. Assembly replaces a range and
// copies the rest through, so such a range deleted the opener, kept the closer,
// and produced a page that lost the element's contents and carried markup
// closing nothing. Measured once on a corpus entry: 4388 characters became 680.
//
// `container-extents.ts` fixed that at the origin: the block carrying a
// container's opening tag now owns it, and the block carrying its closing tag
// owns that one. A tag is therefore inside some block, a range is a union of
// whole blocks, and a range that holds half a tag cannot be minted.
//
// SO THE RULE IS NO LONGER "BOTH TAGS OR NEITHER". That rule described a world
// where tags floated between blocks, and enforcing it now would refuse the
// ordinary healthy case: a container whose blocks fall in different slices puts
// its opening tag in the first slice's own text and its closing tag in the
// last's, each lane sees its tag and reproduces it, and the page stays
// balanced. A lane that drops one is a CANDIDATE fault, caught at the slice
// floor since 2026-09-07 (`mask-container-tags.ts` carries a lone tag as an
// atom) and again where the page grammar is read, not a preparation fault.
//
// What is checked instead is the property the origin fix establishes: every tag
// of a container that holds blocks lies wholly inside one of them, and no slice
// range ends part way through a tag. Both fire only on a regression in how
// extents or ranges are derived, which is exactly what they are for.

/**
 Raised when a container tag is not wholly owned by the block or range that
 reaches it.

 @example
 ```ts
 throw new ContainerIntegrityError({ message: 'no block owns the opening tag of details', },);
 ```
 */
export class ContainerIntegrityError extends Error {
  /**
   Builds the failure naming which element is not owned whole.

   @param message - which tag of which element is unowned or cut

   @example
   ```ts
   throw new ContainerIntegrityError({ message: 'slice 1 cuts the closing tag of details', },);
   ```
   */
  public constructor({ message, }: { readonly message: string; },) {
    super(message,);
    this.name = 'ContainerIntegrityError';
  }
}

/**
 Half-open range, named so the two questions asked of it read plainly.
 */
type Range = {
  /**
   Absolute inclusive start.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive end.
   */
  readonly endOffset: number;
};

/**
 Whether an outer range holds an inner one whole.

 @param outer - range doing the holding

 @param inner - range that must fit inside

 @returns Whether every character of inner lies in outer

 @example
 ```ts
 if (covers({ outer: span, inner: opener, },)) { }
 ```
 */
function covers(
  {
    outer,
    inner,
  }: {
    readonly outer: Range;
    readonly inner: Range;
  },
): boolean {
  return (inner.startOffset >= outer.startOffset)
    && (inner.endOffset <= outer.endOffset);
}

/**
 Whether two ranges share any character at all.

 ASKED BESIDE {@link covers} rather than instead of it, because the two differ
 exactly where a range ends part way through a tag. That range neither holds
 the tag nor leaves it alone, and assembly would write over half of it.

 @param outer - range doing the overlapping

 @param inner - range being overlapped

 @returns Whether any character is common to both

 @example
 ```ts
 if (touches({ outer: span, inner: closer, },)) { }
 ```
 */
function touches(
  {
    outer,
    inner,
  }: {
    readonly outer: Range;
    readonly inner: Range;
  },
): boolean {
  return (inner.startOffset < outer.endOffset)
    && (inner.endOffset > outer.startOffset);
}

/**
 Names a container in a form a reader can find on the page.

 @param container - container being reported

 @returns Element name in angle brackets, or a neutral phrase when unnamed

 @example
 ```ts
 const named = nameOf({ container, },);
 ```
 */
function nameOf({ container, }: { readonly container: ContainerSpan; },): string {
  return container.name === ''
    ? 'a fragment'
    : `<${container.name}>`;
}

/**
 One container tag, named so a message can say which half is at fault.
 */
type LabelledTag = {
  /**
   Which half of the element this is, in a word a message can use.
   */
  readonly label: string;

  /**
   Range that tag occupies.
   */
  readonly range: Range;
};

/**
 Reads both tags of a container as ranges, in document order.

 @param container - container whose tags are wanted

 @returns Opening tag range then closing tag range, each labelled

 @example
 ```ts
 for (const tag of tagsOf({ container, },)) { }
 ```
 */
function tagsOf(
  { container, }: { readonly container: ContainerSpan; },
): readonly LabelledTag[] {
  return [
    {
      label: 'opening',
      range: {
        startOffset: container.openerStartOffset,
        endOffset: container.openerEndOffset,
      },
    },
    {
      label: 'closing',
      range: {
        startOffset: container.closerStartOffset,
        endOffset: container.closerEndOffset,
      },
    },
  ];
}

/**
 Refuses a document whose container tags are owned by no block.

 A container holding no blocks is left alone deliberately: its tags belong to
 nothing, but no range can reach them either, so assembly copies the region
 through untouched.

 @param blocks - document blocks carrying absolute offsets

 @param containers - every container the parse dissolved, absolute offsets

 @throws {@link ContainerIntegrityError} when a tag of a container holding
 blocks is owned by no block, or is only partly covered by one

 @example
 ```ts
 assertTagsRideInBlocks({ blocks, containers, },);
 ```
 */
function assertTagsRideInBlocks(
  {
    blocks,
    containers,
  }: {
    readonly blocks: readonly DocumentNode[];
    readonly containers: readonly ContainerSpan[];
  },
): void {
  for (const container of containers) {
    /**
     Whether any block lies within this container's whole span, which is what
     separates a container with contents from an empty one.
     */
    const holdsBlocks = blocks.some(function isInside(block,): boolean {
      return (block.startOffset >= container.openerStartOffset)
        && (block.endOffset <= container.closerEndOffset);
    },);
    for (const tag of tagsOf({ container, },)) {
      /**
       Whether some block owns this tag whole.
       */
      const owned = blocks.some(function ownsTag(block,): boolean {
        return covers({
          outer: block,
          inner: tag.range,
        },);
      },);
      if (owned)
        continue;

      /**
       Whether some block covers part of this tag without owning it.
       */
      const cut = blocks.some(function cutsTag(block,): boolean {
        return touches({
          outer: block,
          inner: tag.range,
        },);
      },);
      if (cut)
        throw new ContainerIntegrityError({
          message: `a block covers part of the ${tag.label} tag of ${nameOf({ container, },)} without `
            + 'covering all of it, so every range minted from that block would carry half a tag',
        },);
      if (holdsBlocks)
        throw new ContainerIntegrityError({
          message: `no block owns the ${tag.label} tag of ${nameOf({ container, },)}, though the `
            + 'container holds blocks: extents were derived without widening onto container tags, '
            + 'so a range boundary can fall between this tag and its partner',
        },);
    }
  }
}

/**
 Refuses any slice whose range ends part way through a container tag.

 @param slices - prepared slice pairs

 @param containers - every container the parse dissolved, absolute offsets

 @throws {@link ContainerIntegrityError} when a range covers part of a tag

 @example
 ```ts
 assertNoSliceCutsATag({ slices, containers, },);
 ```
 */
function assertNoSliceCutsATag(
  {
    slices,
    containers,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly containers: readonly ContainerSpan[];
  },
): void {
  for (const slice of slices) {
    /**
     Target side of this pair, which is the side assembly writes over.
     */
    const span = slice.target;

    // An insertion names a place rather than a range, so it replaces nothing
    // and can cut no tag. Where it lands inside a container is a question about
    // placement, which `span-contiguity.ts` already answers.
    if (isInsertionChunk(span,))
      continue;
    for (const container of containers) {
      for (const tag of tagsOf({ container, },)) {
        /**
         Whether this range reaches the tag without owning all of it, which is
         the only way a range built from whole blocks could carry half a tag.
         */
        const cuts = (touches({
          outer: span,
          inner: tag.range,
        },))
          && (!covers({
            outer: span,
            inner: tag.range,
          },));
        if (cuts)
          throw new ContainerIntegrityError({
            message: `slice ${String(span.sliceIndex,)} ends part way through the ${tag.label} tag of `
              + `${nameOf({ container, },)}, so assembly would replace half of it and leave the `
              + 'rest beside text written without it',
          },);
      }
    }
  }
}

/**
 Refuses a prepared pair whose container tags are not owned whole.

 CALLED AT PREPARATION, where the parsed document still remembers which
 containers were dissolved. Nothing later can: the slices carry blocks and
 offsets, and a tag is neither.

 @param slices - prepared slice pairs

 @param containers - every container the parse dissolved, absolute offsets

 @param blocks - target document blocks carrying absolute offsets

 @throws {@link ContainerIntegrityError} when a tag is owned by no block, or a
 range ends part way through one

 @example
 ```ts
 assertContainerIntegrity({ slices, containers: doc.containers, blocks: doc.nodes, },);
 ```
 */
export function assertContainerIntegrity(
  {
    slices,
    containers,
    blocks,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly containers: readonly ContainerSpan[];
    readonly blocks: readonly DocumentNode[];
  },
): void {
  assertTagsRideInBlocks({
    blocks,
    containers,
  },);
  assertNoSliceCutsATag({
    slices,
    containers,
  },);
}

//endregion Container integrity
