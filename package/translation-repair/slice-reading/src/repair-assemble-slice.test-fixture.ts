import type { ChunkPair, } from '../dist/final/node/index.mjs';

//region Repair assemble slice
// ONE PREPARED SLICE OVER A SPAN OF A DOCUMENT, matched by searching the
// document for the slice's own wording, since each wording is unique in it.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The repair-assemble-slice-match and
// repair-assemble-withdrawal tests kept their own copy of this builder; all
// now import it from here.

/**
 Which slice to prepare and the wording it holds.
 */
type SliceRequest = {
  /**
   Position of the slice in its preparation.
   */
  readonly sliceIndex: number;

  /**
   Wording the slice holds, found in the document by search.
   */
  readonly text: string;
};

/**
 Builds a function that prepares one slice at a time over a given document.

 @param targetText - document the built slices' offsets address

 @returns Builder preparing one slice over that document

 @throws {@link Error} from the builder when a slice's wording is not in the
 document, rather than addressing offset -1

 @example
 ```ts
 const sliceOf = sliceBuilderFor({ targetText: TARGET_TEXT, },);
 const slice = sliceOf({ sliceIndex: 0, text: FIRST_ARCHIVE, },);
 ```
 */
export function sliceBuilderFor(
  { targetText, }: { readonly targetText: string; },
): (request: SliceRequest,) => ChunkPair {
  // Prepares one slice over a span of the document, searching for its own
  // wording since each slice's wording is unique in it.
  return function sliceOf(
    {
      sliceIndex,
      text,
    }: SliceRequest,
  ): ChunkPair {
    /**
     Where this slice starts, found by search since each wording is unique here.
     */
    const startOffset = targetText.indexOf(text,);
    if (startOffset === (-1))
      throw new Error(`slice ${String(sliceIndex,)} wording is not in the document it is cut from`,);

    return {
      source: {
        sliceIndex,
        text: '小猫在窗台上打盹。',
        startOffset: 0,
        endOffset: 9,
        nodes: [],
      },
      target: {
        sliceIndex,
        text,
        startOffset,
        endOffset: startOffset + text.length,
        nodes: [],
      },
    };
  };
}

//endregion Repair assemble slice
