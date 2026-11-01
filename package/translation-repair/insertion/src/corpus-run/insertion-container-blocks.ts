import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ChunkPair,
  type ContainerHalfPair,
  containerHalfPairs,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  maskHtmlComments,
  maskLoneContainerTags,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { readSliceSkeleton, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Insertion container blocks
// WHAT A CONTAINER HOLDS, SIDE BY SIDE AND SLICE BY SLICE, for the block
// deficit (`insertion-container-deficit.ts`).
//
// READ OFF THE PARSE (ledger B68). The deficit counted runs of non-blank
// lines, and a list written tight on one side and loose on the other, or a
// fenced block with a blank line inside, counted differently on the two
// sides of one rendering: a container the archive renders whole admitted an
// absent passage, and one it renders short refused it. A block is a
// top-level node of the slice reading the floor uses (`readSliceSkeleton`),
// so a list is one block however its items are spaced.
//
// WHAT THAT GIVES UP, as a choice: a list the archive renders with two of
// its three items reads one block against one, and no deficit. A partial
// rendering is the roster's carried and partly-carried verdicts' business,
// not this signal's, which says only that the archive holds fewer blocks.
//
// A HALF THE PARSE REFUSES leaves the container uncounted, and the reading
// says which slice and why. Both documents parse whole, so a refused half is
// not expected; counting what the parser refused would be guessing.

/**
 Which part of a container a slice is.
 */
type ContainerRole = 'open' | 'close' | 'inside';

/**
 Which side of a slice is read.
 */
type Side = 'source' | 'target';

/**
 One container both of whose halves the archive carries, with the blocks
 each side writes inside it.
 */
export type CarriedContainer = {
  /**
   Halves in their slices.
   */
  readonly pair: ContainerHalfPair;

  /**
   Blocks the original writes inside the container, by prepared position,
   so a passage inside it costs what the container counted for it.
   */
  readonly sourceByPosition: ReadonlyMap<number, number>;

  /**
   Blocks the original writes inside the container.
   */
  readonly sourceBlocks: number;

  /**
   Blocks the archive writes inside it.
   */
  readonly targetBlocks: number;
};

/**
 One carried container as read: counted, or left uncounted because a half
 of one side would not parse.
 */
export type ContainerReading =
  | {
    readonly kind: 'counted';

    /**
     The container with its counts.
     */
    readonly container: CarriedContainer;
  }
    | {
      readonly kind: 'unread';

    /**
     Halves in their slices.
     */
      readonly pair: ContainerHalfPair;

    /**
     Side whose slice the parse refused.
     */
      readonly side: Side;

    /**
     Slice the parse refused.
     */
      readonly sliceIndex: number;

    /**
     The parser's own account.
     */
      readonly detail: string;
    };

/**
 One side's blocks inside a container, slice by slice, or the first slice
 the parse refused.
 */
type SideCount =
  | {
    readonly kind: 'counted';

    /**
     Blocks by prepared position.
     */
    readonly byPosition: ReadonlyMap<number, number>;
  }
    | {
      readonly kind: 'unread';

    /**
     Slice the parse refused.
     */
      readonly sliceIndex: number;

    /**
     The parser's own account.
     */
      readonly detail: string;
    };

/**
 Text of one side of one slice with comments and lone container tags
 blanked, cut to the part inside the container: after the opening tag in the
 opening half, before the closing tag in the closing half.

 @param text - side text as written

 @param name - element name of the container

 @param role - which half this slice is, or a whole slice inside

 @returns Masked text inside the container

 @example
 ```ts
 const inside = insideContainer({ text, name: 'details', role: 'open', },);
 ```
 */
function insideContainer(
  {
    text,
    name,
    role,
  }: {
    readonly text: string;
    readonly name: string;
    readonly role: ContainerRole;
  },
): string {
  /**
   Text with comments blanked, so a tag in a comment is not structure.
   */
  const { masked: uncommented, } = maskHtmlComments({ text, },);
  /**
   Lone tags of this side, and the text with them blanked.
   */
  const {
    masked,
    tags,
  } = maskLoneContainerTags({ text: uncommented, },);
  if (role === 'inside')
    return masked;
  /**
   Lone tags of the container's kind and name on this side, in order.
   */
  const ownTags = tags.filter(function ofContainer(candidate,): boolean {
    return (candidate.kind === role) && (candidate.name === name);
  },);

  // CUT AT THE TAG ITSELF, by the offset the mask read it at (ledger B67).
  // Searching for the tag's text found a whole element of the same name
  // beside the container instead, and counted its blocks as the container's.
  // The first lone opener and the last lone closer are the outermost where
  // two containers of one name nest.
  /**
   The container's own tag on this side, absent where this side does not
   write it.
   */
  const tag = (role === 'open') ? ownTags.at(0,) : ownTags.at(-1,);
  if (tag === undefined)
    return masked;
  if (role === 'open')
    return masked.slice(tag.endOffset,);
  return masked.slice(
    0,
    tag.startOffset,
  );
}

/**
 Role of the slice at one position inside a container.

 @param position - prepared position of the slice

 @param pair - the container's halves

 @returns Which part of the container the slice is

 @example
 ```ts
 const role = roleAt({ position: 3, pair, },); // 'inside'
 ```
 */
function roleAt(
  {
    position,
    pair,
  }: {
    readonly position: number;
    readonly pair: ContainerHalfPair;
  },
): ContainerRole {
  /**
   The container's halves.
   */
  const {
    open,
    close,
  } = pair;
  if (position === open.position)
    return 'open';
  return (position === close.position) ? 'close' : 'inside';
}

/**
 Text one side of a slice writes, empty where the archive has none.

 @param slice - prepared slice

 @param side - which side

 @returns That side's text

 @example
 ```ts
 const text = sideText({ slice, side: 'target', },);
 ```
 */
function sideText(
  {
    slice,
    side,
  }: {
    readonly slice: ChunkPair;
    readonly side: Side;
  },
): string {
  /**
   Both sides of the slice; the archive's writes nothing where it is an
   insertion.
   */
  const {
    source,
    target,
  } = slice;
  if (side === 'source')
    return source.text;
  return isInsertionChunk(target,) ? '' : target.text;
}

/**
 Counts the blocks one side writes inside a container, slice by slice, off
 the parse.

 @param slices - prepared slices in document order

 @param pair - the container's halves

 @param side - which side to read

 @returns Blocks by position, or the first slice the parse refused

 @example
 ```ts
 const count = countSide({ slices, pair, side: 'source', },);
 ```
 */
function countSide(
  {
    slices,
    pair,
    side,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly pair: ContainerHalfPair;
    readonly side: Side;
  },
): SideCount {
  /**
   The container's halves.
   */
  const {
    open,
    close,
  } = pair;
  /**
   Blocks read so far, by position.
   */
  const byPosition = new Map<number, number>();
  /**
   Slices from the opening half through the closing half.
   */
  const spanned = slices.slice(
    open.position,
    close.position + 1,
  );
  for (const [offset, slice,] of spanned.entries()) {
    /**
     Prepared position of this slice.
     */
    const position = open.position + offset;
    /**
     This slice's side, read the way the floor reads a slice.
     */
    const read = readSliceSkeleton({
      text: insideContainer({
        text: sideText({
          slice,
          side,
        },),
        name: open.name,
        role: roleAt({
          position,
          pair,
        },),
      },),
    },);
    if (read.kind === 'unparseable') {
      return {
        kind: 'unread',
        sliceIndex: slice.target
          .sliceIndex,
        detail: read.detail,
      };
    }
    byPosition.set(
      position,
      read.skeleton
        .blocks
        .length,
    );
  }
  return {
    kind: 'counted',
    byPosition,
  };
}

/**
 Sums a side's blocks.

 @param byPosition - blocks by position

 @returns Their total

 @example
 ```ts
 const total = totalOf({ byPosition, },);
 ```
 */
function totalOf({ byPosition, }: { readonly byPosition: ReadonlyMap<number, number>; },): number {
  return [...byPosition.values(),].reduce(
    function add(
      sum,
      blocks,
    ): number {
      return sum + blocks;
    },
    0,
  );
}

/**
 Reads every container both of whose halves the archive carries, with the
 blocks each side writes inside it.

 @param slices - prepared slices in document order

 @returns Carried containers in closing-half order, each counted or left
 uncounted with the slice the parse refused

 @example
 ```ts
 const readings = readCarriedContainers({ slices, },);
 ```
 */
export function readCarriedContainers(
  { slices, }: { readonly slices: readonly ChunkPair[]; },
): readonly ContainerReading[] {
  return containerHalfPairs({ slices, },)
    .filter(function bothCarried(pair,): boolean {
      /**
       The container's halves.
       */
      const {
        open,
        close,
      } = pair;
      return (!open.insertion) && (!close.insertion);
    },)
    .map(function counted(pair,): ContainerReading {
      /**
       The original's blocks inside the container.
       */
      const source = countSide({
        slices,
        pair,
        side: 'source',
      },);
      if (source.kind === 'unread') {
        return {
          kind: 'unread',
          pair,
          side: 'source',
          sliceIndex: source.sliceIndex,
          detail: source.detail,
        };
      }
      /**
       The archive's blocks inside it.
       */
      const target = countSide({
        slices,
        pair,
        side: 'target',
      },);
      if (target.kind === 'unread') {
        return {
          kind: 'unread',
          pair,
          side: 'target',
          sliceIndex: target.sliceIndex,
          detail: target.detail,
        };
      }
      return {
        kind: 'counted',
        container: {
          pair,
          sourceByPosition: source.byPosition,
          sourceBlocks: totalOf({ byPosition: source.byPosition, },),
          targetBlocks: totalOf({ byPosition: target.byPosition, },),
        },
      };
    },);
}

//endregion Insertion container blocks
