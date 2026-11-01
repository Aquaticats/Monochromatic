/**
 Semver parsing and comparison utilities for catalog-tighten.
 
 Handles `>=` range extraction and semver comparison
 for determining when catalog entries can be tightened.
 */

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
 Splits a semver string into `[major, minor, patch, prerelease]`.
 Prerelease is everything after the first `-`, or empty string.
 
 @param version - semver string, e.g. `"1.2.3"` or `"7.0.0-dev.20250311"`
 
 @returns tuple of `[major, minor, patch, prerelease]`
 
 @example
 ```ts
 splitSemver("1.2.3") // [1, 2, 3, ""]
 splitSemver("7.0.0-dev.20250311") // [7, 0, 0, "dev.20250311"]
 ```
 */
function splitSemver(version: string,): [
  number,
  number,
  number,
  string,
] {
  /**
   Position of the prerelease separator `-`, or `-1` when the version is a plain release.
   */
  const dashIndex = version.indexOf('-',);
  /**
   Sentinel value returned by `String.indexOf` when the search target is absent.
   */
  const NOT_FOUND = -1;
  /**
   Whether the version carries a prerelease suffix; precomputed so the slice expressions read as a plain ternary.
   */
  const hasPrerelease = dashIndex !== NOT_FOUND;
  /**
   Prerelease tag (substring after `-`) or empty string for a plain release.
   */
  const prerelease = hasPrerelease ? version.slice(dashIndex + 1,) : '';
  /**
   Dotted-numeric core of the version (`major.minor.patch`), stripped of any prerelease suffix.
   */
  const coreStr = hasPrerelease
    ? version.slice(
      0,
      dashIndex,
    )
    : version;
  /**
   Three string segments parsed out of `coreStr`; cast to numbers below.
   */
  const parts = coreStr.split('.',);
  return [
    Number(parts[0],),
    Number(parts[1],),
    Number(parts[2],),
    prerelease,
  ];
}

/**
 Determines whether `installed` is strictly greater than `cataloged`.
 Follows semver ordering: major \> minor \> patch \> prerelease (lexicographic).
 A release version (no prerelease) is greater than any prerelease of the same triple.
 
 @param cataloged - version from the catalog range
 
 @param installed - version from node_modules
 
 @returns `true` if `installed` is strictly newer
 
 @example
 ```ts
 isStrictlyGreater({ cataloged: "1.2.0", installed: "1.3.0" }) // true
 isStrictlyGreater({ cataloged: "1.2.0", installed: "1.2.0" }) // false
 isStrictlyGreater({ cataloged: "7.0.0-dev.1", installed: "7.0.0-dev.2" }) // true
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
  /**
   Catalog version split into `[major, minor, patch, prerelease]` for component-wise comparison.
   */
  const [cMaj, cMin, cPat, cPre,] = splitSemver(cataloged,);
  /**
   Installed version split into `[major, minor, patch, prerelease]` to test against the catalog tuple.
   */
  const [iMaj, iMin, iPat, iPre,] = splitSemver(installed,);

  if (iMaj !== cMaj)
    return iMaj > cMaj;
  if (iMin !== cMin)
    return iMin > cMin;
  if (iPat !== cPat)
    return iPat > cPat;

  // Same major.minor.patch: compare prerelease
  // No prerelease > any prerelease (release is "greater" than prerelease of same triple)
  if ((cPre !== '') && (iPre === ''))
    return true;
  if ((cPre === '') && (iPre !== '')) {
    // Installed is a prerelease of the same triple; not greater
    return false;
  }
  // Both have prerelease or both have none
  return iPre > cPre;
}

//endregion Semver parsing
