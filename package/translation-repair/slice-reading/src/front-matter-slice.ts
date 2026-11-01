import type {
  ChunkPair,
  SliceSyntax,
} from './chunk-document.ts';
import {
  type ContentChunk,
  makeInsertionChunk,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { FrontMatterBlock, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Front matter slice
// Metadata is visible localized page content, but Markdown parsing deliberately
// excludes it from document nodes. This factory gives it one explicit syntax-
// bearing slice without pretending YAML keys are Markdown blocks.

/**
 Result of aligning optional front matter.

 @example
 ```ts
 const result: FrontMatterSliceResult = { kind: 'none', };
 ```
 */
export type FrontMatterSliceResult = {
  readonly kind: 'none';
} | {
  readonly kind: 'paired';
  readonly slice: ChunkPair;
};

/**
 The span one document's metadata fills: its exact bytes, from its opening
 fence, which a leading byte order mark does not move into the slice.

 @param block - metadata of one document

 @returns Content chunk over exactly those bytes

 @example
 ```ts
 const chunk = metadataChunk({ block, },);
 ```
 */
function metadataChunk({ block, }: { readonly block: FrontMatterBlock; },): ContentChunk {
  /**
   Exact metadata bytes and where the opening fence stands.
   */
  const {
    raw,
    startOffset,
  } = block;
  return {
    kind: 'content',
    sliceIndex: 0,
    nodes: [],
    startOffset,
    endOffset: startOffset + raw.length,
    text: raw,
  };
}

/**
 Creates front-matter slice when both documents declare one.

 The spans start where each block's opening fence stands, so a byte order mark
 opening a page stays outside the slice and no rewrite of the metadata
 reaches it.

 @param source - original front matter

 @param target - translation front matter

 @param targetInsertionOffset - where metadata is inserted when the translation
 has none: after the byte order mark its page opens with, zero where it opens
 with none

 @returns Tagged syntax-bearing pair,
 insertion pair for source-only metadata,
 or no localized slice

 @example
 ```ts
 const pair = frontMatterSlice({ source, target, });
 ```
 */
export function frontMatterSlice(
  {
    source,
    target,
    targetInsertionOffset = 0,
  }: {
    readonly source?: FrontMatterBlock;
    readonly target?: FrontMatterBlock;
    readonly targetInsertionOffset?: number;
  },
): FrontMatterSliceResult {
  if (source === undefined)
    return { kind: 'none', };

  return {
    kind: 'paired',
    slice: {
      syntax: 'front-matter',
      source: metadataChunk({ block: source, },),
      target: (target === undefined)
        ? makeInsertionChunk({
          sliceIndex: 0,
          offset: targetInsertionOffset,
        },)
        : metadataChunk({ block: target, },),
    },
  };
}

/**
 Names syntax-bearing metadata slices in prepared order.

 @param slices - prepared document slices

 @returns Set of front matter slice indexes

 @example
 ```ts
 const indexes = frontMatterSliceIndexes({ slices, });
 ```
 */
export function frontMatterSliceIndexes(
  { slices, }: { readonly slices: readonly ChunkPair[]; },
): ReadonlySet<number> {
  return new Set(slices
    .filter(function hasSyntax(slice,): boolean {
      return slice.syntax === ('front-matter' satisfies SliceSyntax);
    },)
    .map(function toIndex(slice,): number {
      return slice.target
        .sliceIndex;
    },),);
}

//endregion Front matter slice
