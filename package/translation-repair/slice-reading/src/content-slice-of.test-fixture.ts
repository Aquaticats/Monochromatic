import type { ChunkPair, } from '../dist/final/node/index.mjs';

//region Content slice of text
// ONE SLICE PAIR WHOSE ORIGINAL SIDE CARRIES GIVEN TEXT, target side empty.
// Offsets and nodes are named directly rather than parsed: what each of
// these cases is under test for is which text or picture comes back, and
// from where, not how a slice was carved.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The corpus-run/entry-pictures,
// document-readings, fidelity-window and slice-pictures tests kept their own
// copy of this pair builder; all now import it from here.

/**
 Builds one slice pair carrying given original text, target side empty.

 @param text - original-side text this slice covers

 @param sliceIndex - position of this slice in its document

 @returns Pair whose original side carries that text

 @example
 ```ts
 const pair = sliceOf({ text: 'Mittens naps.\n', sliceIndex: 0, },);
 ```
 */
export function sliceOf(
  {
    text,
    sliceIndex,
  }: {
    readonly text: string;
    readonly sliceIndex: number;
  },
): ChunkPair {
  return {
    source: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: text.length,
      text,
    },
    target: {
      sliceIndex,
      nodes: [],
      startOffset: 0,
      endOffset: 0,
      text: '',
    },
  };
}

//endregion Content slice of text
