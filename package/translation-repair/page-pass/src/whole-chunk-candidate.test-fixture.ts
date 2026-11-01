import type { Candidate, } from '@monochromatic-dev/translation-repair-house-rule';
import type { PatchOutcome, } from '../dist/final/node/index.mjs';

import { SEAT_HYPER_OPENROUTER_VISION_EDITOR, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Whole-chunk candidate
// ONE WHOLE-CHUNK CANDIDATE FROM AN EDITOR OUTSIDE THE BENCH, for cases that
// need a producer whose candidate is not drawn from a model roster slot.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The chunk-decline-consequence and
// repeated-content-rule tests kept their own copy of this builder; both now
// import it from here.

/**
 One whole-chunk candidate.

 @param patchedText - the candidate's chunk text

 @returns Candidate carrying that text

 @example
 ```ts
 const candidate = chunkCandidate({ patchedText: 'The cat loves the sun.', },);
 ```
 */
export function chunkCandidate({ patchedText, }: { readonly patchedText: string; },): Candidate<PatchOutcome> {
  return {
    producer: {
      kind: 'model',
      modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    },
    value: {
      patchedText,
      applied: [],
      rejected: [],
    },
    rendered: patchedText,
  };
}

//endregion Whole-chunk candidate
