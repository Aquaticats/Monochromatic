/**
 Tests for the finding a driver files when it refuses a cached slice.

 WHY THIS SITS APART from `translate-unheard.unit.test.ts`: `await describe`
 throws, so a failure in that file`s suite would abort this one before it ran,
 and a GFP round pointed at either could then prove nothing about the other.

 WHAT IT PINS is the slice this finding names. A cached record that heard no
 translator was written by another build, and the driver asks the slice again
 rather than settling on a wording nobody produced. Measured on 2026-08-25,
 shifting the index by one failed no case, so a corpus reader chasing the
 refusal would have opened the neighbouring slice and found nothing wrong with
 it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { unheardCacheDiscardFinding, } from '../dist/final/node/index.mjs';

await describe({
  name: unheardCacheDiscardFinding.name,
  children: [
    it({
      name: 'NAMES the slice it refused, and carries the tag its siblings share, so a run that started '
        + 'refusing every cached slice can be counted across a corpus rather than read one at a time',
      fn: async () => {
        expect(unheardCacheDiscardFinding({ sliceIndex: 4, },),).toBe(
          'translate-discarded-unheard-slice chunk 4; cached record heard no translator, which this '
            + 'driver never caches, so it was written by another build and the slice was asked again '
            + 'rather than settled on a wording nobody produced',
        );
      },
    },),
  ],
},);
