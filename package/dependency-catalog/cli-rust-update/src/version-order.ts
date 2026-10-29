/**
 Semantic-version ordering and Cargo's caret compatibility for crate versions.

 Crate versions are SemVer 2.0.0: `MAJOR.MINOR.PATCH`, an optional `-prerelease`,
 and optional `+build` metadata, which never affects ordering
 (for example `0.11.1+wasi-snapshot-preview1`).

 @module
 */

//region Parse

/**
 One parsed version.

 @example
 ```ts
 const parsed: ParsedVersion = { major: 1, minor: 2, patch: 3, prerelease: ['rc', '1',], };
 ```
 */
export type ParsedVersion = {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;

  /**
   Dot-separated prerelease identifiers, empty for a release.
   */
  readonly prerelease: readonly string[];
};

/**
 Thrown for text that is not a SemVer version.

 @example
 ```ts
 throw new VersionSyntaxError('1.x');
 ```
 */
export class VersionSyntaxError extends Error {
  /**
   Builds the error.

   @param version - text that failed to parse
   */
  constructor(version: string,) {
    super(`${JSON.stringify(version,)} is not a SemVer version (MAJOR.MINOR.PATCH, optional -prerelease and +build)`,);
    this.name = 'VersionSyntaxError';
  }
}

/**
 Parts of a SemVer core: `MAJOR.MINOR.PATCH`.
 */
const CORE_PART_COUNT = 3;

/**
 Code point of `0`.
 */
const DIGIT_ZERO = 48;

/**
 Code point of `9`.
 */
const DIGIT_NINE = 57;

/**
 Whether text is one or more ASCII digits.

 @param text - candidate

 @returns true for a non-empty all-digit string

 @example
 ```ts
 isDigits('42'); // true
 ```
 */
function isDigits(text: string,): boolean {
  if (text === '')
    return false;
  for (const char of text) {
    /**
     Code point of the character.
     */
    const code = char.codePointAt(0,) ?? 0;
    if ((code < DIGIT_ZERO) || (code > DIGIT_NINE))
      return false;
  }
  return true;
}

/**
 One numeric core part, rejecting leading zeros as SemVer does.

 @param part - text between dots

 @param version - whole version, for the error

 @returns the number

 @throws VersionSyntaxError when the part is not a canonical non-negative integer

 @example
 ```ts
 corePart({ part: '12', version: '1.12.0', }); // 12
 ```
 */
function corePart(
  {
    part,
    version,
  }: {
    readonly part: string;
    readonly version: string
  },
): number {
  if ((!isDigits(part,))
    || ((part.length > 1) && part.startsWith('0',)))
    throw new VersionSyntaxError(version,);
  /**
   Parsed value.
   */
  const value = Number(part,);
  if (!Number.isSafeInteger(value,))
    throw new VersionSyntaxError(version,);
  return value;
}

/**
 Parses a SemVer version, dropping build metadata.

 @param version - version text

 @returns its parts

 @throws VersionSyntaxError when the text is not a SemVer version

 @example
 ```ts
 parseVersion('0.11.1+wasi-snapshot-preview1'); // { major: 0, minor: 11, patch: 1, prerelease: [] }
 ```
 */
export function parseVersion(version: string,): ParsedVersion {
  /**
   Text before any build metadata.
   */
  const [withoutBuild = '',] = version.split(
    '+',
    1,
  );
  /**
   Position of the prerelease separator.
   */
  const dash = withoutBuild.indexOf('-',);
  /**
   Core `MAJOR.MINOR.PATCH`.
   */
  const core = dash === (-1) ? withoutBuild : withoutBuild.slice(
    0,
    dash,
  );
  /**
   Prerelease text, empty for a release.
   */
  const pre = dash === (-1) ? '' : withoutBuild.slice(dash + 1,);
  /**
   Core parts.
   */
  const parts = core.split('.',);
  if ((parts.length !== CORE_PART_COUNT) || ((dash !== (-1)) && (pre === '')))
    throw new VersionSyntaxError(version,);
  /**
   Prerelease identifiers.
   */
  const prerelease = pre === '' ? [] : pre.split('.',);
  if (prerelease.some(function isEmpty(identifier: string,): boolean {
    return identifier === '';
  },))
    throw new VersionSyntaxError(version,);
  return {
    major: corePart({
      part: parts[0] ?? "",
      version,
    },),
    minor: corePart({
      part: parts[1] ?? "",
      version,
    },),
    patch: corePart({
      part: parts[2] ?? "",
      version,
    },),
    prerelease,
  };
}

//endregion Parse

//region Order

/**
 Orders two prerelease identifiers by SemVer's rule:
 numeric ones numerically and below alphanumeric ones, alphanumeric ones by ASCII.

 @param left - first identifier

 @param right - second identifier

 @returns negative, zero, or positive

 @example
 ```ts
 compareIdentifiers({ left: '2', right: '10', }); // negative
 ```
 */
function compareIdentifiers(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string
  },
): number {
  /**
   Whether the left identifier is numeric.
   */
  const leftNumeric = isDigits(left,);
  /**
   Whether the right identifier is numeric.
   */
  const rightNumeric = isDigits(right,);
  if (leftNumeric && rightNumeric)
    return Number(left,) - Number(right,);
  if (leftNumeric)
    return -1;
  if (rightNumeric)
    return 1;
  if (left === right)
    return 0;
  return left < right ? -1 : 1;
}

/**
 Orders two prerelease lists; a release (empty list) ranks above any prerelease.

 @param left - first list

 @param right - second list

 @returns negative, zero, or positive

 @example
 ```ts
 comparePrerelease({ left: ['rc',], right: [], }); // negative
 ```
 */
function comparePrerelease(
  {
    left,
    right,
  }: {
    readonly left: readonly string[];
    readonly right: readonly string[]
  },
): number {
  if ((left.length === 0) || (right.length === 0))
    return right.length - left.length;
  /**
   Identifiers both lists have.
   */
  const shared = Math.min(
    left.length,
    right.length,
  );
  for (let index = 0; index < shared; index += 1) {
    /**
     Order at this position.
     */
    const order = compareIdentifiers({
      left: left[index] ?? '',
      right: right[index] ?? '',
    },);
    if (order !== 0)
      return order;
  }
  return left.length - right.length;
}

/**
 Orders two versions by SemVer precedence, ignoring build metadata.

 @param left - first version

 @param right - second version

 @returns negative when `left` is lower, zero when equal, positive when higher

 @throws VersionSyntaxError when either is not a SemVer version

 @example
 ```ts
 ['1.0.0', '1.0.0-rc.1',].toSorted(function byVersion(a, b) { return compareVersions({ left: a, right: b, }); });
 ```
 */
export function compareVersions(
  {
    left,
    right,
  }: {
    readonly left: string;
    readonly right: string
  },
): number {
  /**
   Parsed left side.
   */
  const a = parseVersion(left,);
  /**
   Parsed right side.
   */
  const b = parseVersion(right,);
  return (a.major - b.major) || (a.minor - b.minor)
    || (a.patch - b.patch)
    || comparePrerelease({
      left: a.prerelease,
      right: b.prerelease,
    },);
}

/**
 Whether `candidate` satisfies Cargo's default caret requirement written as `version`,
 so `cargo update --precise` can move a lockfile entry between them without a manifest change.

 `^1.4.2` allows `>=1.4.2, <2.0.0`; `^0.4.2` allows `>=0.4.2, <0.5.0`; `^0.0.3` allows only `0.0.3`.
 The check here keeps the compatibility class and ignores the lower bound,
 since a set-aside candidate is older than the refused version by construction.

 @param version - the refused version

 @param candidate - an older release to try

 @returns true when both sit in the same caret class

 @throws VersionSyntaxError when either is not a SemVer version

 @example
 ```ts
 caretCompatible({ version: '0.4.2', candidate: '0.4.0', }); // true
 ```
 */
export function caretCompatible(
  {
    version,
    candidate,
  }: {
    readonly version: string;
    readonly candidate: string
  },
): boolean {
  /**
   Parsed refused version.
   */
  const a = parseVersion(version,);
  /**
   Parsed candidate.
   */
  const b = parseVersion(candidate,);
  if (a.major !== b.major)
    return false;
  if (a.major > 0)
    return true;
  if (a.minor !== b.minor)
    return false;
  if (a.minor > 0)
    return true;
  return a.patch === b.patch;
}

//endregion Order
