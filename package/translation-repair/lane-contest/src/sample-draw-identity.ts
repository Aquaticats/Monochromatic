import { createHash, } from 'node:crypto';

//region Sample draw identity
// One draw's fingerprint, so a graded sheet can be proved to belong to the
// manifest it is scored against.
//
// The seed alone cannot do this. The draw is deterministic in its SEED but not
// in its POOL, and the pool grows with every entry that settles, so the same
// seed at the same corpus commit yields a DIFFERENT set of items once another
// entry lands. Two draws can therefore agree on seed, on corpus pin, and on
// item count while describing different issues, and the scorers join sheet to
// manifest BY POSITION. That join would then mislabel every verdict, silently,
// and produce a precision figure with no error anywhere in it.
//
// The digest covers identities only: which issue sat at which position, plus
// the seed and pin that name the draw. Never the rendered sheet body. The whole
// purpose of a sheet is that a human writes grades into it, so a digest over
// its text would break the moment it was used for what it is for.

/**
 Prefix distinguishing this digest from any other hash in the codebase.

 Carried inside the hashed value rather than beside it, so a digest computed
 for something else can never collide with one computed for a draw, and so a
 later change to what a draw identity contains announces itself as a mismatch
 rather than as agreement.
 */
export const DRAW_IDENTITY_DOMAIN = 'sample-draw/v1';

/**
 The part of a drawn item that its draw identity is taken over.

 @example
 ```ts
 const item: DrawIdentityItem = { position: 1, entryId: 'Kitten', issueId: 'adjudicated/nap', };
 ```
 */
export type DrawIdentityItem = {
  /**
   One-based sheet position.
   */
  readonly position: number;

  /**
   Corpus entry the issue came from.
   */
  readonly entryId: string;

  /**
   Adjudicated issue drawn at this position.
   */
  readonly issueId: string;
};

/**
 Fingerprints one draw from the items it produced.

 Canonicalized through `JSON.stringify` rather than by joining fields with a
 delimiter. An entry id or issue id is arbitrary text crossing into whatever
 grammar the canonical form uses, and a delimiter-joined encoding lets one
 item containing the delimiter impersonate two, so two different draws could
 hash alike.

 @param seed - draw seed

 @param corpusSha - pinned corpus commit

 @param items - drawn items, in the order both sheets render them

 @returns Hex digest naming this exact draw

 @example
 ```ts
 const digest = computeDrawDigest({ seed, corpusSha, items, },);
 ```
 */
export function computeDrawDigest(
  {
    seed,
    corpusSha,
    items,
  }: {
    readonly seed: string;
    readonly corpusSha: string;
    readonly items: readonly DrawIdentityItem[];
  },
): string {
  return createHash('sha256',)
    .update(
    JSON.stringify({
      domain: DRAW_IDENTITY_DOMAIN,
      seed,
      corpusSha,
      items: items.map(function toTuple(item,) {
        return [
          item.position,
          item.entryId,
          item.issueId,
        ];
      },),
    },),
    )
    .digest('hex',);
}

//endregion Sample draw identity
