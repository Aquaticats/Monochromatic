/**
 * Timestamp parsing and recency checking for artifact directory names.
 *
 * Artifact directories encode timestamps in a filesystem-safe slug format
 * (colons replaced with hyphens). This module reverses that encoding and
 * checks whether timestamps fall within a 24-hour recency threshold.
 */

import { MS_PER_DAY, } from '@monochromatic-dev/const-core/ts';

/**
 * 24 hours in milliseconds
 */
export const TWENTY_FOUR_HOURS_MS: number = MS_PER_DAY;

//region Date-format constants

/**
 * Number of digits in the year portion (`YYYY`).
 */
const YEAR_DIGITS = 4;
/**
 * Position of the separator char immediately after the year.
 */
const YEAR_END = YEAR_DIGITS;
/**
 * Minimum slug length: year digits + 1-char separator.
 */
const YEAR_HYPHEN_PREFIX_LENGTH = YEAR_DIGITS + 1;
/**
 * Position of the first month digit.
 */
const MONTH_START = YEAR_HYPHEN_PREFIX_LENGTH;
/**
 * Position of the separator char immediately after the month (`YYYY:MM` is 7 chars).
 */
const MONTH_END = 7;
/**
 * Position of the first day digit.
 */
const DAY_START = 8;
/**
 * Position immediately after the day digits (`YYYY:MM:DD` is 10 chars).
 */
const DAY_END = 10;

//endregion Date-format constants

//region ASCII digit-run predicate

/**
 * Options for {@link isAsciiDigitRun}.
 */
type IsAsciiDigitRunOptions = {
  /**
   * String whose chars are inspected.
   */
  readonly s: string;
  /**
   * Inclusive start index of the run.
   */
  readonly start: number;
  /**
   * Number of chars to check from `start`.
   */
  readonly count: number;
};

/**
 * Tests whether the `count` chars of `s` starting at `start` are all ASCII
 * digits, in one linear left-to-right pass with no recursion.
 *
 * Out-of-range indices read as `''` via `charAt`, which fails the digit
 * check, so a run extending past the end returns `false`.
 *
 * @param s - string whose chars are inspected
 *
 * @param start - inclusive start index of the run
 *
 * @param count - number of chars to check from `start`
 *
 * @returns whether every char in the run is `0`-`9`
 *
 * @example
 * ```ts
 * isAsciiDigitRun({ s: '2026', start: 0, count: 4 }); // true
 * isAsciiDigitRun({ s: '20x6', start: 0, count: 4 }); // false
 * ```
 */
function isAsciiDigitRun({
  s,
  start,
  count,
}: IsAsciiDigitRunOptions,): boolean {
  return (function scanDigits(): boolean {
    /**
     * Offset into the run; advances one char per iteration.
     */
    let offset = 0;
    while (offset < count) {
      /**
       * Char at the absolute index; only ASCII digits keep the run alive.
       */
      const c = s.charAt(start + offset,);
      if ((c < '0') || (c > '9'))
        return false;
      offset += 1;
    }
    return true;
  })();
}

//endregion ASCII digit-run predicate

//region Year-hyphen prefix predicate

/**
 * Tests whether `s` starts with four ASCII digits followed by `-`,
 * matching the `\d{4}-` shape every timestamp slug shares.
 *
 * @param s - candidate slug
 *
 * @returns whether the slug satisfies the year-prefix shape
 *
 * @example
 * ```ts
 * hasYearHyphenPrefix('2026-03-06T12-00-00.000Z'); // true
 * hasYearHyphenPrefix('not-a-year');               // false
 * ```
 */
function hasYearHyphenPrefix(s: string,): boolean {
  if (s.length
    < YEAR_HYPHEN_PREFIX_LENGTH)
    return false;
  return isAsciiDigitRun({
    s,
    start: 0,
    count: YEAR_DIGITS,
  },)
    && (s.charAt(YEAR_END,)
      === '-');
}

//endregion Year-hyphen prefix predicate

/**
 * Result of scanning artifact directories for recent activity.
 * Contains both per-probe pairs and whole-model failures so the runner can
 * skip probes that were already attempted (successfully or not) within 24 hours.
 */
export type RecentArtifactScan = {
  /**
   * Map from model label to set of recently-tested probe names
   */
  readonly probePairs: ReadonlyMap<string, ReadonlySet<string>>;
  /**
   * Model labels that had a whole-model failure (e.g. 429, auth error) within 24 hours
   */
  readonly failedModels: ReadonlySet<string>;
};

/**
 * Parsed components of an artifact directory name; `null` when the
 * directory name does not follow the `probe-pass-timestamp` convention.
 */
export type ArtifactDirParts = {
  /**
   * Probe name (everything up to the pass marker).
   */
  probe: string;
  /**
   * Pass marker, one of `'initial'` or `'fix'`.
   */
  pass: 'initial' | 'fix';
  /**
   * Filesystem-safe timestamp slug starting with a four-digit year.
   */
  timestamp: string;
};

/**
 * Sentinel returned by the directory-name parsers when the name does not match
 * the expected convention. A unique symbol keeps the "no match" outcome out of
 * the parsed value's space without a banned nullish union.
 */
export const NO_MATCH: unique symbol = Symbol('no-match',);

/**
 * Parses an artifact directory name into its `probe`, `pass`, and
 * `timestamp` components.
 *
 * Mirrors `/^(?<probe>.+)-(?<pass>initial|fix)-(?<timestamp>\d{4}-.+)$/`
 * with a linear search: locate the rightmost `-initial-` or `-fix-`
 * boundary whose tail starts with four digits and a `-`, then carve
 * the three substrings. Returns `null` when no boundary qualifies.
 *
 * @param name - directory basename
 *
 * @returns parsed components or {@link NO_MATCH}
 *
 * @example
 * ```ts
 * parseArtifactDir('csv-rfc4180-initial-2026-03-06T12-00-00.000Z');
 * // { probe: 'csv-rfc4180', pass: 'initial', timestamp: '2026-03-06T12-00-00.000Z' }
 * ```
 */
export function parseArtifactDir(name: string,): ArtifactDirParts | typeof NO_MATCH {
  /**
   * Searches `name` for `-<pass>-` followed by a timestamp slug.
   *
   * @param marker - one of `-initial-` or `-fix-`
   *
   * @returns matching parts, or {@link NO_MATCH} when the marker does not yield a timestamp
   */
  function tryMarker(marker: '-initial-' | '-fix-',): ArtifactDirParts | typeof NO_MATCH {
    return (function scanMarkers(): ArtifactDirParts | typeof NO_MATCH {
      // Walk every occurrence of `marker` right-to-left (longest valid probe
      // prefix wins, matching the original regex's greedy `.+` capture) in a
      // single linear pass: each step moves the cursor strictly left of the
      // previous match, so no frame stacks up the way the recursion did.
      /**
       * Search end index; `lastIndexOf` stops at or before it, then moves left of each non-matching occurrence.
       */
      let from = name.length;
      while (from >= 0) {
        /**
         * Right-most occurrence of `marker` at or before `from`; `-1` (and index 0, an empty probe) ends the search.
         */
        const idx = name.lastIndexOf(
          marker,
          from,
        );
        if (idx <= 0)
          return NO_MATCH;
        /**
         * Slug after the marker; must satisfy the year-prefix shape to count.
         */
        const tail = name.slice(idx + marker
          .length,);
        if (hasYearHyphenPrefix(tail,)) {
          return {
            probe: name.slice(
              0,
              idx,
            ),
            pass: marker === '-initial-' ? 'initial' : 'fix',
            timestamp: tail,
          };
        }
        from = idx - 1;
      }
      return NO_MATCH;
    })();
  }
  /**
   * Initial-pass match attempt; falls through to the fix-pass marker on no match.
   */
  const initial = tryMarker('-initial-',);
  return initial !== NO_MATCH ? initial : tryMarker('-fix-',);
}

/**
 * Parses a failure-artifact directory name into its `timestamp`
 * component.
 *
 * Mirrors `/^failure-(?<timestamp>\d{4}-.+)$/` with a linear check:
 * requires the `failure-` prefix and a four-digit-year-anchored tail.
 *
 * @param name - directory basename
 *
 * @returns timestamp slug or {@link NO_MATCH}
 *
 * @example
 * ```ts
 * parseFailureDir('failure-2026-03-06T12-00-00.000Z');
 * // { timestamp: '2026-03-06T12-00-00.000Z' }
 * ```
 */
export function parseFailureDir(name: string,): { timestamp: string; } | typeof NO_MATCH {
  /**
   * Literal prefix consumed before the timestamp slug.
   */
  const PREFIX = 'failure-';
  if (!name.startsWith(PREFIX,))
    return NO_MATCH;
  /**
   * Slug after the prefix; must start with four digits and a hyphen.
   */
  const tail = name.slice(PREFIX.length,);
  if (!hasYearHyphenPrefix(tail,))
    return NO_MATCH;
  return { timestamp: tail, };
}

/**
 * Restores an ISO timestamp from its filesystem-safe slug form.
 * Reverses the transformation done by `timestampSlug`: hyphens back to colons,
 * then fixes the date portion (year-MM-DD) which was incorrectly colonized.
 *
 * @param rawTimestamp - filesystem-safe timestamp slug (e.g. "2026-03-06T12-00-00.000Z")
 *
 * @returns ISO 8601 timestamp string, or undefined if parsing fails
 */
function restoreTimestamp(rawTimestamp: string,): string {
  /**
   * Slug with hyphens swapped back to colons; the date portion is fixed up to hyphens by the return expression.
   */
  const withColons = rawTimestamp
    .replaceAll(
      '-',
      ':',
    )
    .replace(
      'T:',
      'T',
    );
  // Fix the date part: year:MM:DD -> year-MM-DD (first two colons after year are date separators)
  return rewriteDateColons(withColons,);
}

/**
 * Replaces the two colons in a `YYYY:MM:DD` prefix with hyphens to restore
 * the ISO date shape. Mirrors `s.replace(/^(\d{4}):(\d{2}):(\d{2})/, '$1-$2-$3')`
 * with explicit index checks; returns `s` unchanged when the prefix does
 * not match.
 *
 * @param s - candidate timestamp string
 *
 * @returns `s` with the leading date colons replaced, or `s` verbatim
 */
function rewriteDateColons(s: string,): string {
  if (s.length
    < 'YYYY:MM:DD'
    .length)
    return s;
  if (
    (!isAsciiDigitRun({
      s,
      start: 0,
      count: YEAR_DIGITS,
    },))
    || (s.charAt(YEAR_END,)
      !== ':')
      || (!isAsciiDigitRun({
      s,
      start: MONTH_START,
      count: 2,
    },))
      || (s.charAt(MONTH_END,)
        !== ':')
      || (!isAsciiDigitRun({
      s,
      start: DAY_START,
      count: 2,
    },))
  ) {
    return s;
  }
  return `${
    s.slice(
      0,
      YEAR_END,
    )
  }-${
    s.slice(
      MONTH_START,
      MONTH_END,
    )
  }-${
    s.slice(
      DAY_START,
      DAY_END,
    )
  }${s.slice(DAY_END,)}`;
}

/**
 * Options for {@link isRecentTimestamp}.
 *
 * @example
 * ```ts
 * const opts: IsRecentTimestampOptions = {
 *   rawTimestamp: '2026-03-06T12-00-00.000Z',
 *   cutoff: Date.now() - 86_400_000,
 * };
 * ```
 */
type IsRecentTimestampOptions = {
  /**
   * Filesystem-safe timestamp slug
   */
  readonly rawTimestamp: string;
  /**
   * Cutoff time in milliseconds since epoch
   */
  readonly cutoff: number;
};

/**
 * Checks whether a filesystem-safe timestamp slug falls within the recent cutoff.
 *
 * @param rawTimestamp - filesystem-safe timestamp slug
 *
 * @param cutoff - cutoff time in milliseconds since epoch
 *
 * @returns true if the timestamp is recent (after the cutoff)
 *
 * @example
 * ```ts
 * const cutoff = Date.now() - 86_400_000;
 * isRecentTimestamp({ rawTimestamp: '2026-03-06T12-00-00.000Z', cutoff });
 * ```
 */
export function isRecentTimestamp({
  rawTimestamp,
  cutoff,
}: IsRecentTimestampOptions,): boolean {
  /**
   * Restored ISO 8601 timestamp; parsed by `Date` below for the cutoff comparison.
   */
  const fixed = restoreTimestamp(rawTimestamp,);
  /**
   * Epoch milliseconds for the artifact's timestamp; NaN when the slug failed to parse.
   */
  const entryTime = new Date(fixed,).getTime();
  return (!Number.isNaN(entryTime,)) && (entryTime >= cutoff);
}
