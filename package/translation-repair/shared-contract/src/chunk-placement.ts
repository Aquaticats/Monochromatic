import type { DocumentNode, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Chunk placement
// What a chunk POINTS AT, which until now was always existing text and soon
// will not be. A source section with no translation at all has to name the
// place its translation belongs rather than a span of the target document, so
// the target side of a pair becomes either CONTENT, which is text that is
// there, or an INSERTION, which is a boundary where text is not.
//
// SPLIT FROM `chunk-document.ts` on purpose. That file is about carving a
// document into sections and pairing two documents' sections; this is about
// what one side of such a pair can BE, which every later stage reads and few
// of them construct.
//
// THE DISCRIMINANT IS A FIELD, not a shape. An insertion carries no nodes and
// no text, so `nodes.length === 0` would identify one today, and the
// constructors here cannot produce an empty content chunk: `chunkByHeadings`
// builds every group from at least one node and `runToChunk` throws on an
// empty run. But the exported type is structural, so any caller can fabricate
// an empty content chunk, and under a structural test that fabrication
// silently BECOMES an insertion. A field says what a value is rather than
// letting its emptiness decide.
//
// WRITTEN BEFORE ANYTHING PRODUCED AN INSERTION, deliberately. The producers
// landed last, on 2026-08-23 (`chunk-insertion.ts` at section scale,
// `group-source-anchor.ts` at block scale), once assembly, the lanes and the
// caches could each consume one.

/**
 Where a chunk sits in its document, shared by both kinds.

 @example
 ```ts
 const position: ChunkPosition = { sliceIndex: 0, startOffset: 0, endOffset: 12, };
 ```
 */
type ChunkPosition = {
  /**
   Position of this chunk within its document, from zero.
   */
  readonly sliceIndex: number;

  /**
   Absolute start of the half-open document range this chunk represents.

   For CONTENT this is the first node's start. For an INSERTION it equals
   {@link ChunkPosition.endOffset} and names the boundary new text is written
   at, so the range holds no existing text.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive end of that half-open range.

   For CONTENT this is the last node's end. For an INSERTION it equals
   {@link ChunkPosition.startOffset}.
   */
  readonly endOffset: number;
};

/**
 Existing document text, named by the range it covers.

 `text` is the owning document sliced from `startOffset` to `endOffset`, so
 inter-node blank lines inside the chunk are preserved.

 WHAT ASSEMBLY READS is that range and that text; the constructors here emit
 at least one node per chunk, but nothing downstream requires it, and the
 layout guard in `placement-layout.ts` accepts an empty `nodes` on a content
 chunk whose offsets and text agree. A test fixture naming a span directly is
 a legitimate value of this type rather than a malformed one.

 THE DISCRIMINANT IS OPTIONAL HERE and required on {@link InsertionChunk},
 which is what lets every existing construction site stay as it is while an
 insertion still cannot be passed where content is required: under
 `exactOptionalPropertyTypes` a `kind` of `'insertion'` satisfies neither
 `'content'` nor absence. Making it required is a later tightening; no
 chunk is ever
 serialized, so an absent discriminant cannot reach a reader as data.

 @example
 ```ts
 const chunk: ContentChunk = { sliceIndex: 0, nodes, startOffset: 0, endOffset: 12, text: '## 简介\n', };
 ```
 */
export type ContentChunk = ChunkPosition & {
  /**
   Names this as existing content, and may be omitted.
   */
  readonly kind?: 'content';

  /**
   Nodes of this chunk in source order;
   every document node belongs to exactly one chunk.
   */
  readonly nodes: readonly DocumentNode[];

  /**
   Text this chunk covers, sliced from the owning document.
   */
  readonly text: string;
};

/**
 A boundary where a translation belongs and none exists.

 What it carries is a place: an insertion at offset `p` covers `[p, p)`, and
 its text is written between everything ending at `p` and everything starting
 there.

 EMPTINESS IS A CONSTRUCTOR PROMISE, not a type one. `no-optional-escape`
 refuses the empty tuple and empty-string literal that would state it in the
 type, so `nodes` and `text` stay wide and {@link makeInsertionChunk} is what
 keeps them empty. A hand-built insertion carrying nodes or text is therefore
 expressible; `assertPlacementLayout` refuses it at assembly, which is the
 last point before anything is written, and every lane trusts the
 discriminant rather than re-deriving it.

 @example
 ```ts
 const anchor: InsertionChunk = makeInsertionChunk({ sliceIndex: 4, offset: 1_280, },);
 ```
 */
export type InsertionChunk = ChunkPosition & {
  /**
   Names this as a place rather than as content.

   THE ONLY FIELD THAT SAYS SO. An earlier draft narrowed `nodes` to an empty
   tuple and `text` to the empty string, which `no-optional-escape` refuses
   and is right to: a zero-length container is absence spelled as a value,
   and the rule asks for a distinct non-empty domain value instead. This is
   that value, and {@link makeInsertionChunk} is what keeps the other two
   fields empty.
   */
  readonly kind: 'insertion';

  /**
   Nodes this covers, which {@link makeInsertionChunk} leaves empty: an
   insertion names a boundary, and a boundary has no nodes.
   */
  readonly nodes: readonly DocumentNode[];

  /**
   Text this covers, which {@link makeInsertionChunk} leaves empty for the
   same reason.
   */
  readonly text: string;
};

/**
 One side of a pair: existing content, or the place content is missing from.

 Both members carry `sliceIndex`, `nodes`, `text` and both offsets, so a
 reader that only needs the text of a side needs no narrowing. A consumer
 that would be WRONG about an insertion asks for {@link ContentChunk}
 instead, and the compiler stops the union there.

 @example
 ```ts
 const target: DocumentChunk = isInsertionChunk(side,) ? side : side;
 ```
 */
export type DocumentChunk = ContentChunk | InsertionChunk;

/**
 Builds the anchor for a translation that has nowhere to go yet.

 ONE OFFSET, NOT TWO, which is what makes the empty range unfalsifiable: a
 caller cannot hand in a start and an end that disagree. What this cannot
 check is whether the offset lies inside the document it names, since the
 document is not here; `assertPlacementLayout` validates bounds and ordering at assembly,
 where the target text is in hand and every placement is visible at once.

 @param sliceIndex - position this anchor holds among the slices

 @param offset - boundary in the target document new text is written at

 @returns Anchor covering nothing at that boundary

 @example
 ```ts
 const anchor = makeInsertionChunk({ sliceIndex: 4, offset: 1_280, },);
 ```
 */
export function makeInsertionChunk(
  {
    sliceIndex,
    offset,
  }: {
    readonly sliceIndex: number;
    readonly offset: number;
  },
): InsertionChunk {
  return {
    kind: 'insertion',
    sliceIndex,
    nodes: [],
    startOffset: offset,
    endOffset: offset,
    text: '',
  };
}

/**
 Reports whether a chunk names a place rather than covering text.

 TAKES ITS PARAMETER POSITIONALLY, which is the one exception the repo's
 destructured-parameter rule cannot absorb: a type predicate narrows a NAMED
 parameter, and a destructured object has no name to narrow.

 @param chunk - either side of a prepared pair

 @returns Whether this is an insertion anchor

 @example
 ```ts
 const missing = isInsertionChunk(pair.target,);
 ```
 */
export function isInsertionChunk(chunk: DocumentChunk,): chunk is InsertionChunk {
  return chunk.kind === 'insertion';
}

//endregion Chunk placement
