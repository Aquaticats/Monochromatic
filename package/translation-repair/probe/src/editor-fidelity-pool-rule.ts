import { createHash, } from 'node:crypto';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Editor fidelity pool rule
// THE RULE THE EDITOR FIDELITY POOL IS DRAWN BY, in the parts that need no corpus: which entry ids take part, the
// order a seeded draw visits entries in, and the order donors are offered in. The draw (`corpus-run/editor-fidelity-
// pool-selection.ts`) applies them to the pinned clone; `editor-fidelity-pool-manifest.ts` states the whole rule.

/**
 Identity domain of this draw, so its digests can never collide with another draw's over the same seed and ids.
 */
export const POOL_DRAW_DOMAIN = 'editor-fidelity-pool/v2';

/**
 Seed of the pool the fourth round froze.
 */
export const DEFAULT_POOL_SEED = 'round-4';

/**
 Date the pool of this draw is frozen on, written into every drawn specification so that a fresh draw gives the committed
 bytes whatever day it is run. The sixth round redrew the pool wider on this day.
 */
export const POOL_DRAWN_ON = '2026-10-24';

/**
 Specifications to a drawn part file, so that no part passes the package's file-length cap.
 */
export const SPECS_PER_PART = 8;

/**
 Trials the draw asks for, reviewed ones counted. The draw stops at the trials wanted or when the eligible entries of the
 clone run out; the pinned clone offers fewer than this, so the pool holds every eligible entry's trials.
 */
export const POOL_TRIALS_WANTED = 150;

/**
 Shortest folded English a section may have to be drawn, in characters.
 */
export const MIN_REFERENCE_CHARS = 400;

/**
 Longest folded English a section may have to be drawn, in characters.
 */
export const MAX_REFERENCE_CHARS = 2_200;

/**
 Longest Chinese a section may have to be drawn, in characters; it must also be nonempty.
 */
export const MAX_SOURCE_CHARS = 2_200;

/**
 Separator between fields of a digest input: a NUL, which no seed or id holds, written as an escape so git keeps the file
 as text.
 */
const FIELD_SEPARATOR = '\u0000';

/**
 Characters an entry id may be made of.
 */
const ID_CHARACTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_.-';

/**
 Whether an entry id is plain: nonempty, made of letters, digits, underscores, dots and hyphens, and not beginning with a
 dot, since the id becomes a file name in the clone and a specification id in the manifest.

 @param entryId - id to test

 @returns Whether the draw may take part with it

 @example
 ```ts
 const plain = isPlainEntryId({ entryId: 'MTF_0615', },);
 ```
 */
export function isPlainEntryId({ entryId, }: { readonly entryId: string; },): boolean {
  if ((entryId === '') || entryId.startsWith('.',))
    return false;
  for (const character of entryId) {
    if (!ID_CHARACTERS.includes(character,))
      return false;
  }
  return true;
}

/**
 Orders entry ids by a digest of the domain, the seed and the id, so the order is reproducible from the seed alone and
 independent of the order the ids are listed in.

 @param entryIds - ids to order

 @param seed - seed of the draw

 @returns Ids in the order the seed visits them

 @example
 ```ts
 const order = seededEntryOrder({ entryIds, seed: DEFAULT_POOL_SEED, },);
 ```
 */
export function seededEntryOrder(
  {
    entryIds,
    seed,
  }: {
    readonly entryIds: readonly string[];
    readonly seed: string;
  },
): readonly string[] {
  return entryIds
    .map(function withKey(entryId,): {
      readonly entryId: string;
      readonly key: string
    } {
      return {
        entryId,
        key: createHash('sha256',)
          .update(
          [
            POOL_DRAW_DOMAIN,
            seed,
            entryId,
          ].join(FIELD_SEPARATOR,),
          )
          .digest('hex',),
      };
    },)
    .toSorted(function byKey(
      left,
      right,
    ): number {
      return compareCodePoints({
        left: left.key,
        right: right.key,
      },);
    },)
    .map(function toId(keyed,): string {
      return keyed.entryId;
    },);
}

/**
 Orders the sections a donor may come from, the furthest from the damaged section first and, between two equally far,
 the later one first.

 @param indices - positions of the sections on the page

 @param from - position of the damaged section, which donates nothing

 @returns Other positions, furthest first

 @example
 ```ts
 // order is [4, 0, 3, 1]
 const order = furthestFirst({ indices: [0, 1, 2, 3, 4], from: 2, },);
 ```
 */
export function furthestFirst(
  {
    indices,
    from,
  }: {
    readonly indices: readonly number[];
    readonly from: number;
  },
): readonly number[] {
  return indices
    .filter(function other(index,): boolean {
      return index !== from;
    },)
    .toSorted(function byDistanceThenLater(
      left,
      right,
    ): number {
      return (Math.abs(right - from,) - Math.abs(left - from,)) || (right - left);
    },);
}

//endregion Editor fidelity pool rule
