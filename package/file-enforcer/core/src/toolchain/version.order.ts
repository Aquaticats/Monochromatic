/**
 `version.order.ts`: semantic version order for the toolchain sync, pre-releases included (semver 2.0.0 rule 11).

 The order is strict: text that is not `MAJOR.MINOR.PATCH` with an optional pre-release is incomparable, so callers fall
 back to the equality they had before.
 */

import { normalizeVersion, } from './version.ts';

//region Semver order

/**
 Number of dot-separated numeric parts in the version core (`MAJOR.MINOR.PATCH`).
 */
const CORE_PART_COUNT = 3;

/**
 ASCII digits.
 */
const DIGITS = '0123456789';

/**
 Marks text that is not a semantic version.
 */
const NOT_SEMVER: unique symbol = Symbol('text outside the MAJOR.MINOR.PATCH grammar',);

/**
 Parsed semantic version.
 */
type Semver = {
  readonly core: readonly bigint[];
  readonly prerelease: readonly string[];
};

/**
 Whether text is non-empty and made of ASCII digits only.

 @param text - Text to test

 @returns `true` for a run of `0` to `9`
 */
function isDigits(text: string,): boolean {
  if (text === '')
    return false;
  for (const character of text) {
    if (!DIGITS.includes(character,))
      return false;
  }
  return true;
}

/**
 Parses strict semantic version text; build metadata is dropped because it never affects order.

 @param text - Normalized version text

 @returns The parsed version, or {@link NOT_SEMVER} when the text is not `MAJOR.MINOR.PATCH` with an optional pre-release
 */
function parseSemver(text: string,): Semver | typeof NOT_SEMVER {
  /**
   Text before build metadata.
   */
  const withoutBuild = text.split('+',)[0] ?? '';
  /**
   Position of the pre-release separator.
   */
  const dash = withoutBuild.indexOf('-',);
  /**
   Whether the text carries a pre-release.
   */
  const hasPrerelease = withoutBuild.includes('-',);
  /**
   Text of the numeric core.
   */
  const coreText = hasPrerelease
    ? withoutBuild.slice(
      0,
      dash,
    )
    : withoutBuild;
  /**
   Numeric core parts.
   */
  const parts = coreText.split('.',);
  /**
   Pre-release identifiers; none when the separator is absent.
   */
  const prerelease = hasPrerelease
    ? withoutBuild.slice(dash + 1,)
      .split('.',)
    : [];
  if ((parts.length !== CORE_PART_COUNT) || (!parts.every(isDigits,)))
    return NOT_SEMVER;
  if (prerelease.includes('',))
    return NOT_SEMVER;
  return {
    core: parts.map(BigInt,),
    prerelease,
  };
}

/**
 Orders two bigints.

 @param a - First number

 @param b - Second number

 @returns Negative, zero or positive like a sort comparator
 */
function compareBigint(
  {
    a,
    b,
  }: {
    readonly a: bigint;
    readonly b: bigint;
  },
): number {
  if (a === b)
    return 0;
  return (a < b)
    ? -1
    : 1;
}

/**
 Orders two pre-release identifiers: numeric ones compare by value and sit below alphanumeric ones, alphanumeric
 ones compare in ASCII order.

 @param left - First identifier

 @param right - Second identifier

 @returns Negative, zero or positive like a sort comparator
 */
function compareIdentifier(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string;
  },
): number {
  /**
   Whether each identifier is numeric.
   */
  const leftNumeric = isDigits(left,);
  /**
   Whether the second identifier is numeric.
   */
  const rightNumeric = isDigits(right,);
  if (leftNumeric && rightNumeric) {
    return compareBigint({
      a: BigInt(left,),
      b: BigInt(right,),
    },);
  }
  if (leftNumeric !== rightNumeric) {
    return leftNumeric
      ? -1
      : 1;
  }
  if (left === right)
    return 0;
  return (left < right)
    ? -1
    : 1;
}

/**
 Orders two pre-release identifier lists identifier by identifier; a list that prefixes a longer one sits below it.

 @param a - First identifier list

 @param b - Second identifier list

 @returns Negative, zero or positive like a sort comparator
 */
function comparePrerelease(
  {
    a,
    b,
  }: {
    readonly a: readonly string[];
    readonly b: readonly string[];
  },
): number {
  for (const [index, left,] of a.entries()) {
    /**
     Identifier of the other list at the same position.
     */
    const right = b[index];
    if (right === undefined)
      return 1;
    /**
     Order of this identifier pair.
     */
    const byIdentifier = compareIdentifier({
      left,
      right,
    },);
    if (byIdentifier !== 0)
      return byIdentifier;
  }
  return a.length - b.length;
}

/**
 Whether one version is strictly newer than another under semantic version order, pre-releases included.

 @param candidate - Version that may be newer; a release-tag prefix or leading `v` is dropped first

 @param than - Version it is compared against

 @returns `true` only when both sides parse as semantic versions and the candidate orders above the other;
 `false` for equal versions and for any text that is not a semantic version

 @example
 ```ts
 isNewerVersion({ candidate: '0.4.4', than: '0.4.0', },); // true
 isNewerVersion({ candidate: '1.0.0', than: '1.0.0-rc.1', },); // true
 isNewerVersion({ candidate: 'nightly-2026-10-21', than: '0.4.0', },); // false
 ```
 */
export function isNewerVersion(
  {
    candidate,
    than,
  }: {
    readonly candidate: string;
    readonly than: string;
  },
): boolean {
  /**
   Parsed candidate.
   */
  const a = parseSemver(normalizeVersion(candidate,),);
  /**
   Parsed comparison version.
   */
  const b = parseSemver(normalizeVersion(than,),);
  if ((a === NOT_SEMVER) || (b === NOT_SEMVER))
    return false;
  for (const [index, part,] of a.core
    .entries()) {
    /**
     Order of one core part.
     */
    const byPart = compareBigint({
      a: part,
      b: b.core[index] ?? 0n,
    },);
    if (byPart !== 0)
      return byPart > 0;
  }
  /**
   Number of pre-release identifiers on each side; none marks a release.
   */
  const candidateIdentifiers = a.prerelease
    .length;
  /**
   Number of pre-release identifiers of the comparison version.
   */
  const thanIdentifiers = b.prerelease
    .length;
  // Equal cores: a release sits above any pre-release of the same core.
  if (candidateIdentifiers === 0)
    return thanIdentifiers > 0;
  if (thanIdentifiers === 0)
    return false;
  return comparePrerelease({
    a: a.prerelease,
    b: b.prerelease,
  },) > 0;
}

//endregion Semver order
