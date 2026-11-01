/**
 Test-only picture sources for an entry's pass (ledger M70, M113): neither
 the pinned corpus nor the programs installed on the machine, so a test that
 leaves the evidence seam out and still names a picture stops with a message
 naming the seam, rather than reading the corpus and starting `dwebp` and
 `tesseract`. A case that needs pictures hands its own sources, or its own
 evidence reader.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { PassPictureSources, } from '../../dist/final/node/index.mjs';

/**
 Refuses, since a test that reaches it named pictures and no way to read them.
 It always rejects, and serves as the gatherer and the OCR reader alike.

 @throws Error naming the picture sources a test has to hand in

 @example
 ```ts
 await refuseForWantOfPictureSources();
 ```
 */
function refuseForWantOfPictureSources(): Promise<never> {
  return Promise.reject(
    new Error('this test named pictures and handed in no picture sources: pass its own `pictureSources` or a `visualEvidenceReader`',),
  );
}

/**
 Picture sources that read nothing: no corpus bytes and no OCR text.
 */
export const NO_PICTURE_SOURCES: PassPictureSources = {
  gather: refuseForWantOfPictureSources,
  readOcr: refuseForWantOfPictureSources,
};
