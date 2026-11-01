//region Destination renderings
// WHICH SOURCE DESTINATIONS A PAGE MAY LACK, given how the archive rendered them.
//
// THE OWNER'S DECISION OF 2026-09-04 ("Either rendering"): where the archive
// rendered a reference another way than the original (`twitter.com` moved to
// `x.com`, a Chinese Wikipedia article replaced by the English one), a text
// owes one rendering, taken from either side, never both and never neither.
// `translate-atom-rendering.ts` applies that per slice; this is the same rule
// over the whole would-ship page, so the publisher does not refuse the page
// the slice rule accepted. The luxuanwen3 re-run of 2026-09-04 (13:18 UTC)
// ended exactly there: every slice passed, and `DroppedDestinationError` read
// the source alone and refused the `x.com` page 56 minutes in.
//
// THE POOL IS THE WHOLE DOCUMENT'S. Source destinations the archive never
// carried and archive destinations the source never carried form one pool; the
// page owes the larger side's count from it. A destination both sides carry
// as written is owed outright, as before. With no archive, or one carrying
// nothing the source does not, this is the old rule: every source destination
// owed. The document pool is coarser than the slice pools, never stricter: a
// page assembled from slices each meeting its own pool meets this one.
//
// WHAT STAYS A SOURCE GUARD. An archive destination the page lost is not a
// source destination and is not reported here; the slice rule refuses that
// text before it ships.

/**
 Finding recorded when the page lacks a source destination because it
 carries the archive's rendering of that reference instead.
 */
export const ARCHIVE_RENDERING_FINDING: string = 'destinations-archive-rendering';

/**
 Finding recorded when the page carries more renderings from the pool than
 it owes, that is, both the original's and the archive's for one reference.
 */
export const BOTH_RENDERINGS_FINDING: string = 'destinations-both-renderings';

/**
 What the rendering rule decided about a page.

 @example
 ```ts
 const verdict: DestinationRenderingVerdict = judgeDestinationRenderings({ source, page, archive, },);
 ```
 */
export type DestinationRenderingVerdict = {
  /**
   Source destinations the page owes and does not carry.
   */
  readonly dropped: readonly string[];

  /**
   Findings in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Address with a trailing slash shed, so two spellings of one address compare
 equal.

 THE SITE ROOT KEEPS ITS SLASH. `/` is the slash alone, and shed it read as
 the empty string, the key of a destination written with nothing in it,
 which names nowhere; the root and that compared equal.

 @param url - address as written

 @returns Address without a trailing slash, or the slash alone for the site
 root

 @example
 ```ts
 const same = sameAddress({ url: 'https://example.org/a/', },) === sameAddress({ url: 'https://example.org/a', },);
 ```
 */
export function sameAddress({ url, }: { readonly url: string; },): string {
  return (url.endsWith('/',) && (url !== '/'))
    ? url.slice(
      0,
      -1,
    )
    : url;
}

/**
 Addresses as a set of comparison keys.

 @param urls - addresses as written

 @returns Keys with the trailing slash shed

 @example
 ```ts
 const keys = addressKeys({ urls, },);
 ```
 */
function addressKeys({ urls, }: { readonly urls: readonly string[]; },): ReadonlySet<string> {
  return new Set(urls.map(function keyOf(url,): string {
    return sameAddress({ url, },);
  },),);
}

/**
 Addresses whose key is (or is not) in a set.

 @param urls - addresses filtered

 @param keys - keys compared against

 @param present - whether to keep the addresses found in the set or the rest

 @returns Addresses kept, in their given order

 @example
 ```ts
 const rewritten = addressesWhere({ urls: source, keys: archiveKeys, present: false, },);
 ```
 */
function addressesWhere(
  {
    urls,
    keys,
    present,
  }: {
    readonly urls: readonly string[];
    readonly keys: ReadonlySet<string>;
    readonly present: boolean;
  },
): readonly string[] {
  return urls.filter(function matches(url,): boolean {
    return keys.has(sameAddress({ url, },),) === present;
  },);
}

/**
 Source destinations the page owes and lacks, under the either-rendering
 rule.

 @param source - distinct destinations the source carries

 @param page - distinct destinations the would-ship page carries

 @param archive - distinct destinations the archive carried before the run;
 empty when there is no archive

 @returns Dropped destinations and the findings the pool raised

 @example
 ```ts
 const { dropped, findings, } = judgeDestinationRenderings({ source, page, archive, },);
 ```
 */
export function judgeDestinationRenderings(
  {
    source,
    page,
    archive,
  }: {
    readonly source: readonly string[];
    readonly page: readonly string[];
    readonly archive: readonly string[];
  },
): DestinationRenderingVerdict {
  /**
   Keys of what the archive carries.
   */
  const archiveKeys = addressKeys({ urls: archive, },);

  /**
   Keys of what the source carries.
   */
  const sourceKeys = addressKeys({ urls: source, },);

  /**
   Keys of what the page carries.
   */
  const pageKeys = addressKeys({ urls: page, },);

  /**
   Source destinations the archive carries as written: owed outright.
   */
  const shared = addressesWhere({
    urls: source,
    keys: archiveKeys,
    present: true,
  },);

  /**
   Source destinations the archive rendered another way, or dropped.
   */
  const rewritten = addressesWhere({
    urls: source,
    keys: archiveKeys,
    present: false,
  },);

  /**
   Archive destinations the source never carried: its renderings, or additions.
   */
  const replacements = addressesWhere({
    urls: archive,
    keys: sourceKeys,
    present: false,
  },);

  /**
   How many of the pool the page owes: the larger side's count.
   */
  const owed = Math.max(
    rewritten.length,
    replacements.length,
  );

  /**
   Pool entries the page carries.
   */
  const carriedFromPool = addressesWhere({
    urls: [
      ...rewritten,
      ...replacements,
    ],
    keys: pageKeys,
    present: true,
  },);

  /**
   How many of the pool the page carries.
   */
  const drawn = carriedFromPool.length;

  /**
   Shared destinations the page lacks: dropped whatever the pool says.
   */
  const sharedDropped = addressesWhere({
    urls: shared,
    keys: pageKeys,
    present: false,
  },);

  /**
   Rewritten destinations the page lacks: dropped only when the pool is short.
   */
  const rewrittenAbsent = addressesWhere({
    urls: rewritten,
    keys: pageKeys,
    present: false,
  },);

  /**
   Whether the page drew enough from the pool.
   */
  const poolMet = drawn >= owed;

  return {
    dropped: [
      ...sharedDropped,
      ...(poolMet ? [] : rewrittenAbsent),
    ],
    findings: [
      // Conditional spreads keep the findings list free of blanks.
      ...((drawn > owed) ? [BOTH_RENDERINGS_FINDING,] : []),
      ...((poolMet && (rewrittenAbsent.length > 0)) ? [ARCHIVE_RENDERING_FINDING,] : []),
    ],
  };
}

//endregion Destination renderings
