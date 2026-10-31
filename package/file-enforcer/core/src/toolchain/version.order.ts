/**
 `version.order.ts`: semantic version order for the toolchain sync, pre-releases included (semver 2.0.0 rule 11).

 The order is strict: text that is not `MAJOR.MINOR.PATCH` with an optional pre-release is incomparable, so callers fall
 back to the equality they had before.
 */

import { normalizeVersion, } from './version.ts';

//region Semver order

/**
 Parsed semantic version.
 */
type Semver = {
  readonly core: readonly [bigint, bigint, bigint,];
  readonly prerelease: readonly string[];
};

/**
 Whether text is non-empty and made of ASCII digits only.

 @param text - Text to test

 @returns `true` for a run of `0` to `9`
 */
function isDigits(text: string,): boolean {
  if (text.length === 0)
    return false;
  for (let index = 0; index < text.length; index += 1) {
    /**
     Code of one character; `0` is 48 and `9` is 57.
     */
    const code = text.charCodeAt(index,);
    if ((code < 48) || (code > 57))
      return false;
  }
  return true;
}

/**
 Parses strict semantic version text; build metadata is dropped because it never affects order.

 @param text - Normalized version text

 @returns The parsed version, or `undefined` when the text is not `MAJOR.MINOR.PATCH` with an optional pre-release
 */
function parseSemver(text: string,): Semver | undefined {
  /**
   Text before build metadata.
   */
  const withoutBuild = text.split('+',)[0] ?? '';
  /**
   Position of the pre-release separator.
   */
  const dash = withoutBuild.indexOf('-',);
  /**
   Numeric core parts.
   */
  const parts = ((dash === -1)
    ? withoutBuild
    : withoutBuild.slice(
      0,
      dash,
    )).split('.',);
  /**
   Pre-release identifiers; none when the separator is absent.
   */
  const prerelease = (dash === -1)
    ? []
    : withoutBuild.slice(dash + 1,)
      .split('.',);
  if ((parts.length !== 3) || !parts.every(isDigits,))
    return undefined;
  if (prerelease.some(function empty(identifier,): boolean {
    return identifier.length === 0;
  },) || ((dash !== -1) && (prerelease.length === 0)))
    return undefined;
  return {
    core: [
      BigInt(parts[0] ?? '0',),
      BigInt(parts[1] ?? '0',),
      BigInt(parts[2] ?? '0',),
    ],
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
 Orders two pre-release identifier lists: numeric identifiers compare by value and sit below alphanumeric ones,
 alphanumeric ones compare in ASCII order, and a list that prefixes a longer one sits below it.

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
  for (let index = 0; index < Math.min(
    a.length,
    b.length,
  ); index += 1) {
    /**
     Identifier of each side at this position.
     */
    const left = a[index] ?? '';
    const right = b[index] ?? '';
    /**
     Whether each identifier is numeric.
     */
    const leftNumeric = isDigits(left,);
    const rightNumeric = isDigits(right,);
    if (leftNumeric && rightNumeric) {
      /**
       Numeric order of this identifier pair.
       */
      const byValue = compareBigint({
        a: BigInt(left,),
        b: BigInt(right,),
      },);
      if (byValue !== 0)
        return byValue;
      continue;
    }
    if (leftNumeric !== rightNumeric) {
      return leftNumeric
        ? -1
        : 1;
    }
    if (left !== right) {
      return (left < right)
        ? -1
        : 1;
    }
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
   Parsed sides.
   */
  const a = parseSemver(normalizeVersion(candidate,),);
  const b = parseSemver(normalizeVersion(than,),);
  if ((a === undefined) || (b === undefined))
    return false;
  for (let index = 0; index < 3; index += 1) {
    /**
     Order of one core part.
     */
    const byPart = compareBigint({
      a: a.core[index] ?? 0n,
      b: b.core[index] ?? 0n,
    },);
    if (byPart !== 0)
      return byPart > 0;
  }
  // Equal cores: a release sits above any pre-release of the same core.
  if (a.prerelease.length === 0)
    return b.prerelease.length > 0;
  if (b.prerelease.length === 0)
    return false;
  return comparePrerelease({
    a: a.prerelease,
    b: b.prerelease,
  },) > 0;
}

//endregion Semver order
