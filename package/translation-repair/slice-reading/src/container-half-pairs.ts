import type { ChunkPair, } from './chunk-document.ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  maskHtmlComments,
  maskLoneContainerTags,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Container half pairs
// ONE CONTAINER, TWO SLICES. `container-extents.ts` gives a container's opening
// tag to the first block inside it and its closing tag to the last, so a
// disclosure block whose blocks fall in different slices puts `<details>` at
// the head of one slice and `</details>` at the foot of another. Each slice
// reads and validates on its own (class nine masks the lone tag), and nothing
// between the slicer and the page assembly knew which two slices were the two
// halves of one element. XingZ607 (2026-09-18) is what that costs: the
// admission left two opening halves unfilled on a split verdict about the
// summary alone, the closing halves shipped, the strict grammar refused the
// page at the first unmatched closing tag, and the guard, unable to blame one
// slice for two, withdrew all 88 of the lane's replacements.
//
// This reader names the pairs, from the SOURCE text of every slice in document
// order: the original is the one document whose containers are known to be
// whole, and a translation carries each lone tag as an atom the floor holds it
// to. The admission admits both halves together and each assembly withholds a
// half whose partner ships nothing.

/**
 One half of a container, with the slice that owns it.

 @example
 ```ts
 const half: ContainerHalf = { sliceIndex: 4, position: 4, insertion: true, name: 'details', };
 ```
 */
export type ContainerHalf = {
  /**
   Global slice index the replacements and the admission name.
   */
  readonly sliceIndex: number;

  /**
   Position in the prepared slice order, which the admission keys on.
   */
  readonly position: number;

  /**
   Whether the archive has no text for this slice, so shipping nothing here
   leaves the tag off the page.
   */
  readonly insertion: boolean;

  /**
   Element name as the source writes it.
   */
  readonly name: string;
};

/**
 Two halves of one container, in different slices.

 @example
 ```ts
 const pair: ContainerHalfPair = { open, close, };
 ```
 */
export type ContainerHalfPair = {
  /**
   Slice owning the opening tag.
   */
  readonly open: ContainerHalf;

  /**
   Slice owning the closing tag.
   */
  readonly close: ContainerHalf;
};

/**
 One lone tag as the mask reports it, without its text.
 */
type LoneTag = {
  /**
   Whether the tag opens or closes its element.
   */
  readonly kind: 'open' | 'close';

  /**
   Element name as written.
   */
  readonly name: string;
};

/**
 Lone container tags of one slice, in document order, comments masked first
 as every slice reader masks them.

 @param slice - prepared slice whose source is read

 @returns Lone tags with the slice they belong to

 @example
 ```ts
 const tags = loneTagsOf({ slice, },);
 ```
 */
function loneTagsOf(
  { slice, }: { readonly slice: ChunkPair; },
): readonly LoneTag[] {
  /**
   Source text with its comments blanked, so a tag inside a comment is not
   read as structure.
   */
  const { masked, } = maskHtmlComments({
    text: slice.source
      .text,
  },);
  /**
   Tags with no partner inside this slice.
   */
  const { tags, } = maskLoneContainerTags({ text: masked, },);
  return tags;
}

/**
 Pairs every container's opening half with its closing half across slices.

 A closing tag pairs with the innermost open tag of the same name, as the
 document grammar pairs them; a closing tag with no open partner, or an open
 tag never closed, belongs to no pair and is left to the parse.

 @param slices - prepared slices in document order

 @returns Pairs in the order their closing halves occur

 @example
 ```ts
 const pairs = containerHalfPairs({ slices: prepared.slices, },);
 ```
 */
export function containerHalfPairs(
  { slices, }: { readonly slices: readonly ChunkPair[]; },
): readonly ContainerHalfPair[] {
  /**
   Open halves not yet closed, innermost last.
   */
  const open: ContainerHalf[] = [];
  /**
   Pairs found so far.
   */
  const pairs: ContainerHalfPair[] = [];
  for (const [position, slice,] of slices.entries()) {
    /**
     This slice as a half, whichever tag it turns out to own.
     */
    const half = {
      sliceIndex: slice.target
        .sliceIndex,
      position,
      insertion: isInsertionChunk(slice.target,),
    };
    for (const tag of loneTagsOf({ slice, },)) {
      if (tag.kind === 'open') {
        open.push({
          ...half,
          name: tag.name,
        },);
        continue;
      }
      /**
       Innermost open half of the same name, or -1.
       */
      const partner = open.findLastIndex(function sameName(candidate,): boolean {
        return candidate.name === tag.name;
      },);
      if (partner === (-1))
        continue;
      /**
       The opening half this tag closes.
       */
      const [opening,] = open.splice(
        partner,
        1,
      );
      pairs.push({
        open: nonNullishOrThrow(opening,),
        close: {
          ...half,
          name: tag.name,
        },
      },);
    }
  }
  return pairs;
}

/**
 Whether both halves of a pair are source-only slices.

 The assembly's lone-half withholding and the pass's insertion admission
 each kept their own copy of this test (audit area six, 2026-09-28).

 @param pair - container halves in two slices

 @returns True when the archive carries neither half

 @example
 ```ts
 const inserted = containerHalfPairs({ slices, },).filter(function inserted(pair,): boolean {
   return bothHalvesInserted({ pair, },);
 },);
 ```
 */
export function bothHalvesInserted({ pair, }: { readonly pair: ContainerHalfPair; },): boolean {
  return pair.open
    .insertion
    && pair.close
      .insertion;
}

//endregion Container half pairs
