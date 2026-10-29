/**
 Zig resolver: the newest stable version from the download index, for pin
 reporting only.

 @module
 */

import {
  type Fetcher,
  isJsonObject,
  parseJson,
} from './update.resolvers.shared.ts';

/**
 Compares two dotted numeric version keys segment by segment (`0.15.1` against `0.16.0`).

 @param left - first key

 @param right - second key

 @returns negative when `left` is older, positive when newer, zero when equal
 */
function compareVersionKeys(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   Numeric segments of the first key.
   */
  const leftSegments = left.split('.',)
    .map(Number,);
  /**
   Numeric segments of the second key.
   */
  const rightSegments = right.split('.',)
    .map(Number,);
  for (let at = 0; at < Math.max(
    leftSegments.length,
    rightSegments.length,
  ); at += 1) {
    /**
     Segment difference at this position.
     */
    const delta = (leftSegments[at] ?? 0) - (rightSegments[at] ?? 0);
    if (delta !== 0)
      return delta;
  }
  return 0;
}

/**
 Reads the newest stable zig version from the download index, for pin
 reporting only (zig's pin moves only through a plan edit).

 @param fetcher - injected GET

 @returns newest stable index key, or empty when only dev builds are listed

 @example
 ```ts
 await resolveZigLatest({ fetcher, },); // '0.16.0'
 ```
 */
export async function resolveZigLatest(
  { fetcher, }: { readonly fetcher: Fetcher; },
): Promise<string> {
  /**
   Download index keyed by version.
   */
  const doc = parseJson({
    response: await fetcher('https://ziglang.org/download/index.json',),
    label: 'ziglang.org index',
  },);
  /**
   Stable keys, newest first.
   */
  const stable = (isJsonObject(doc,)
    ? Object.keys(doc,)
    : [])
    .filter(
    function isStable(key,): boolean {
      return (key !== 'master') && (!key.includes('-',));
    },
    )
    .toSorted(
    function byVersion(
      left,
      right,
    ): number {
      return compareVersionKeys({
        left,
        right,
      },);
    },
    )
    .toReversed();
  return stable.at(0,) ?? '';
}
