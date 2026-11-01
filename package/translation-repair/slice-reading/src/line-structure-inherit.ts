import { isLineStructured, } from './line-structure.ts';

//region Line structure inheritance
// Decides the line-structure fact ONCE PER CHUNK and hands it to every slice
// carved from that chunk.
//
// WHY THE UNIT IS THE CHUNK. `isLineStructured` reads the median length of a
// slice's blank-line separated blocks, and refuses to answer at all below five
// blocks, because under that a stanza is indistinguishable from a short
// paragraph. Subdivision routinely leaves fewer. Asking the predicate about each
// slice therefore does not ask a harder question, it asks an ANSWERABLE question
// of an unanswerable input, and gets false every time.
//
// MEASURED on `Toka_ls`, the entry whose editor fabricated three lines: its
// verse chunk trips the predicate at 21 blocks, median 22, then subdivides into
// seven slices of which ONE still trips it. Four of the remaining six sit at
// medians 20, 22, 23 and 29, inside the same verse range, and fail only for want
// of a fifth block. Deciding per slice dropped the instruction on six sevenths
// of the verse it was written for, and the resulting run still added five lines.
//
// Subdivision is the pipeline's own choice. It cannot change whether the
// original is verse, so it must not change what the editor is told about it.

/**
 One aligned chunk and the slices carved from it.

 @example
 ```ts
 const chunk: ChunkGovernance = { sourceText, slices: [{ index: 0, sourceText, },], };
 ```
 */
export type ChunkGovernance = Readonly<{
  /**
   Original-side text of the WHOLE chunk, before subdivision.
   */
  sourceText: string;

  /**
   Every slice carved from this chunk, with its own original-side text.
   */
  slices: readonly ChunkSlice[];
}>;

/**
 One slice carved from a chunk.

 @example
 ```ts
 const slice: ChunkSlice = { index: 4, sourceText, };
 ```
 */
export type ChunkSlice = Readonly<{
  /**
   Global index, matching what the artifact records as `sliceIndex`.
   */
  index: number;

  /**
   Original-side text of this slice alone.
   */
  sourceText: string;
}>;

/**
 Reports which slices the line-structure rule governs.

 A UNION of two readings, not a replacement of one by the other. The predicate
 only ever answers false when it cannot tell, so a true from either the slice
 or its enclosing chunk is evidence, and neither false is evidence against.

 Taking the chunk's answer ALONE loses ground, which is measurable rather than
 hypothetical: across the 92 entries at the pinned corpus commit, chunk-only
 governance covers 195 slices against the slice-only 55, but four entries go
 BACKWARDS, `interrgned` from 5 governed slices to 1 and three others from 1 to
 0. Those are stanzas sitting inside a section whose prose dominates the
 chunk's median, so the slice trips and the chunk does not. The union covers
 both shapes and cannot lose to either.

 @param chunks - aligned chunks paired with the slices carved from each

 @returns Global indices of slices whose own original, or whose enclosing
 chunk's original, is line-structured

 @example
 ```ts
 const governed = governedSliceIndices({ chunks, },);
 ```
 */
export function governedSliceIndices(
  { chunks, }: { readonly chunks: readonly ChunkGovernance[]; },
): ReadonlySet<number> {
  return new Set(
    chunks.flatMap(function governedOf(chunk,): readonly number[] {
      /**
       Whether the whole chunk reads as line-structured.
       */
      const chunkGoverns = isLineStructured({ text: chunk.sourceText, },);

      return chunk.slices
        .filter(function isGoverned(slice,): boolean {
          return chunkGoverns
            || isLineStructured({ text: slice.sourceText, },);
        },)
        .map(function toIndex(slice,): number {
          return slice.index;
        },);
    },),
  );
}

//endregion Line structure inheritance
