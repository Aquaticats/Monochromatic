/**
 Semver parsing and comparison utilities for catalog-tighten.

 Handles `>=` range extraction and semver precedence, which decides when a
 catalog floor can be raised. Precedence comes from the shared
 `@monochromatic-dev/dependency-catalog-core-rust` helpers rather than a local
 copy: an npm catalog and a Cargo catalog order the same versions, and one
 comparison serves both.
 */

import {
  compareVersions,
  isVersion,
  parseVersion,
} from '@monochromatic-dev/dependency-catalog-core-rust/ts';

//region Types

/**
 Parsed `>=` range from a catalog entry.
 
 @example `{ prefix: "npm:\@jsr/zod__zod\@", range: ">=4.1.8" }` for `"npm:\@jsr/zod__zod\@>=4.1.8"`
 */
export type ParsedRange = {
  /**
   Everything before the `>=` token, including any `npm:` alias prefix. Empty string for plain `>=x.y.z`.
   */
  prefix: string;
  /**
   Semver version string after `>=`, e.g. `"1.2.3"` or `"7.0.0-dev.20250311"`.
   */
  version: string;
};

//endregion Types

//region Semver parsing

/**
 Literal token that separates the optional alias prefix from the version.
 */
const RANGE_TOKEN = '>=';

/**
 Sentinel returned by {@link parseRange} when a catalog value is not a `>=`
 range. A `unique symbol`; callers narrow with `=== NOT_A_RANGE`.
 */
export const NOT_A_RANGE: unique symbol = Symbol('catalog-tighten/not-a-range',);

/**
 Extracts the `>=` version and any alias prefix from a catalog value.
 Returns {@link NOT_A_RANGE} for values that are not `>=` ranges.
 
 Linear: a single `indexOf` locates the leftmost `>=` (matching the lazy
 `^.*?` semantics of the prior regex), and the substrings on either side
 become the prefix and version. The version must be non-empty for the
 value to count as a range.
 
 @param value - raw catalog entry value, e.g. `">=1.2.3"` or `"npm:@jsr/foo@>=1.0.0"`
 
 @returns parsed prefix and version, or {@link NOT_A_RANGE}
 
 @example
 ```ts
 parseRange(">=1.2.3") // { prefix: "", version: "1.2.3" }
 parseRange("npm:\@jsr/zod__zod\@>=4.1.8") // { prefix: "npm:\@jsr/zod__zod\@", version: "4.1.8" }
 parseRange("*") // NOT_A_RANGE
 ```
 */
export function parseRange(value: string,): ParsedRange | typeof NOT_A_RANGE {
  /**
   Leftmost index of `>=`; `-1` means the value isn't a `>=` range so the caller treats it as opaque.
   */
  const idx = value.indexOf(RANGE_TOKEN,);
  if (idx === (-1))
    return NOT_A_RANGE;
  /**
   Substring before `>=`, preserved verbatim so the npm alias prefix round-trips into the rewritten range.
   */
  const prefix = value.slice(
    0,
    idx,
  );
  /**
   Substring after `>=`, used as the semver to compare against installed versions.
   */
  const version = value.slice(idx + RANGE_TOKEN
    .length,);
  if (version.length
    === 0)
    return NOT_A_RANGE;
  return {
    prefix,
    version,
  };
}

/**
 Result of comparing two versions; positive means the left version wins.
 */
const EQUAL_PRECEDENCE = 0;

/**
 Determines whether `installed` is strictly greater than `cataloged`.
 
 Follows semver precedence (clause 11): major, then minor, then patch, then
 dot-separated prerelease identifiers compared one by one, with numeric
 identifiers compared by value and ranked below alphanumeric ones. A release
 outranks any prerelease of the same core, build metadata never takes part,
 and an absent core segment counts as zero, so a `4.0` floor means `4.0.0`.
 
 Text that is not a version on either side answers `false`: precedence cannot
 be established, so the floor stays as written rather than being raised on a
 guess.
 
 @param cataloged - version from the catalog range
 
 @param installed - version from node_modules
 
 @returns `true` if `installed` is strictly newer
 
 @example
 ```ts
 isStrictlyGreater({ cataloged: "1.2.0", installed: "1.3.0" }) // true
 isStrictlyGreater({ cataloged: "1.2.0", installed: "1.2.0" }) // false
 isStrictlyGreater({ cataloged: "7.0.0-dev.1", installed: "7.0.0-dev.2" }) // true
 isStrictlyGreater({ cataloged: "2.0.1-rc.29", installed: "2.0.1-rc.9" }) // false
 ```
 */
export function isStrictlyGreater(
  {
    cataloged,
    installed,
  }: {
    readonly cataloged: string;
    readonly installed: string;
  },
): boolean {
  if ((!isVersion(cataloged,)) || (!isVersion(installed,)))
    return false;
  /**
   Precedence of the installed version against the catalog floor; only a
   positive result, meaning the install is strictly newer, raises the floor.
   */
  const installedPrecedence = compareVersions({
    left: parseVersion(installed,),
    right: parseVersion(cataloged,),
  },);
  return installedPrecedence > EQUAL_PRECEDENCE;
}

//endregion Semver parsing
