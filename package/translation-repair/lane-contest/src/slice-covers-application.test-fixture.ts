import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type { SeededErrorApplication, } from '../dist/final/node/index.mjs';

//region Slice covers application
// WHETHER A SLICE'S TARGET REGION COVERS A PLANTED SEED'S APPLICATION, for
// cases that find which prepared slice a seeded error landed in.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The repair-benchmark and
// seed-detection-slicing tests kept their own copy of this check; both now
// import it from here.

/**
 Whether a slice's target region covers the planted seed's application.

 @param slice - prepared slice to check

 @param application - planted seed's applied region

 @returns Whether the application's start offset falls inside the slice

 @example
 ```ts
 const found = sliceCoversApplication({ slice, application, },);
 ```
 */
export function sliceCoversApplication(
  {
    slice,
    application,
  }: {
    readonly slice: ChunkPair;
    readonly application: SeededErrorApplication;
  },
): boolean {
  /**
   Where the slice's target region starts and ends.
   */
  const {
    startOffset,
    endOffset,
  } = slice.target;

  return (startOffset <= application.startOffset) && (application.startOffset < endOffset);
}

//endregion Slice covers application
