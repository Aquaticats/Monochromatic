import { createHash, } from 'node:crypto';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';
import type { ShippedRegion, } from './damage-region-v2.ts';
import type { RelabelCase, } from './probe-relabel-case.ts';

//region Damage sample draw
// Draws shipped regions deterministically from a pool, and turns a drawn region
// into the case a sheet item is built from.

/**
 Regions drawn for one sheet.

 Twenty is a compromise between what a rate needs and what a person will read
 carefully. Each item carries a full source passage, both texts, and a
 judgement against the Chinese, so attention rather than count is the limit.
 */
const DAMAGE_SAMPLE_SIZE = 20;

/**
 Identity domain for this draw, so its digests can never collide with the
 detection draw's.

 BUMPED WITH THE POPULATION. Version 1 drew envelopes out of a top-level issue
 list; this draws slices out of both lanes' delivery ledgers. The two are
 different populations addressed by different identities, so sharing a domain
 would let a version 1 sheet and a version 2 sheet claim the same draw.
 */
const DAMAGE_DRAW_DOMAIN = 'damage-sample/v2';

/**
 Separator between fields of a digest input.

 NUL because it cannot occur in a domain, seed, entry id or envelope id, so no
 combination of those fields can reproduce another combination's digest input
 and collide with it.

 Spelled as an escape rather than written as a literal byte. A raw NUL makes
 git record the whole file as binary, so every diff of it reads
 `Binary files differ` and every `rg` search skips it unless asked for
 `--text`. Both were observed on this file before this constant existed.
 */
const FIELD_SEPARATOR = '\u0000';

/**
 One candidate region paired with the key its draw sorts on.

 Named rather than inferred, because an inferred object literal carries
 writable properties and the comparator and unwrapping map that read it then
 take mutable parameters they never mutate.
 */
type KeyedRegion = Readonly<{
  /**
   Region that may be drawn.
   */
  region: ShippedRegion;

  /**
   Seeded hash of the region's identity, the draw order.
   */
  key: string;
}>;

/**
 Draws regions deterministically from the pool.

 Ordered by a digest of the seed and the region's identity, so the draw is
 reproducible from the seed alone and independent of the order artifacts
 happen to sit in on disk.

 @param regions - whole pool

 @param seed - draw seed

 @returns At most twenty regions

 @example
 ```ts
 const drawn = drawRegions({ regions, seed: 'damage-round-one', },);
 ```
 */
export function drawRegions(
  {
    regions,
    seed,
  }: {
    readonly regions: readonly ShippedRegion[];
    readonly seed: string;
  },
): readonly ShippedRegion[] {
  return regions
    .map(function withKey(region,): KeyedRegion {
      return {
        region,
        key: createHash('sha256',)
          .update(
          [
            DAMAGE_DRAW_DOMAIN,
            seed,
            region.entryId,
            region.regionId,
          ].join(FIELD_SEPARATOR,),
          )
          .digest('hex',),
      };
    },)
    .toSorted(function byKey(
      left,
      right,
    ) {
      return compareCodePoints({
        left: left.key,
        right: right.key,
      },);
    },)
    .slice(
    0,
    DAMAGE_SAMPLE_SIZE,
    )
    .map(function toRegion(entry,) {
      return entry.region;
    },);
}

/**
 Turns one drawn region into the case a sheet item is built from.

 NO CORPUS READ AND NO RE-SLICING, unlike the version 1 path. The delivery row
 already carries the original passage and the wording that was there before, as
 the judges saw them, and the settled audit established that re-sliced text is a different
 input from the one that was judged. That also removes a failure mode rather
 than moving it: the old builder could draw a region and then fail to place it
 again, quietly shortening the sheet.

 THE WHOLE SLICE IS THE REGION here, because version 2 delivers by slice rather
 than by envelope, so the wording that was there before and the baseline the
 probe reads are the same text.

 @param ref - drawn region

 @returns One case

 @example
 ```ts
 const built = buildCase({ ref, },);
 ```
 */
export function buildCase(
  { ref, }: { readonly ref: ShippedRegion; },
): RelabelCase {
  return {
    entryId: ref.entryId,
    positions: [],
    region: {
      envelopeId: ref.regionId,
      issueIds: [],
      before: ref.incumbentText,
      editorAfter: ref.shippedText,
    },
    issues: [],
    sourceText: ref.sourceText,
    baselineText: ref.incumbentText,
    recorded: '',
  };
}

//endregion Damage sample draw
