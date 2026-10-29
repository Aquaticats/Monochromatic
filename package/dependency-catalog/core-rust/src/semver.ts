/**
 Cargo semver comparison and single-comparator requirement matching.

 Covers the requirement forms the catalog and crate manifests use: bare and
 `^` caret, `~` tilde, `=` exact, and `>=` floors, each on a partial or full
 version. Follows Cargo's documented rules
 (https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html),
 including that a prerelease version only matches a comparator that names a
 prerelease of the same `major.minor.patch`.

 @module
 */

import { isVersion, } from './syntax.ts';

//region Parsing

/**
 A parsed version; absent minor or patch parts are `-1` so partial requirements keep their width.

 @example
 ```ts
 const version: ParsedVersion = { major: 1, minor: 2, patch: -1, prerelease: [], };
 ```
 */
export type ParsedVersion = {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;
  readonly prerelease: readonly string[];
};

/**
 Marker for a version part the text leaves out.
 */
const ABSENT_PART = -1;

/**
 Parses version text, partial or full, without an operator.

 @param text - version such as `1`, `0.4`, `1.0.104`, or `1.0.0-alpha.5+build`

 @returns parsed parts, with {@link ABSENT_PART} for missing minor or patch

 @throws TypeError when `text` is not a version

 @example
 ```ts
 parseVersion('1.0.0-rc.1'); // { major: 1, minor: 0, patch: 0, prerelease: ['rc', '1'] }
 ```
 */
export function parseVersion(text: string,): ParsedVersion {
  if (!isVersion(text,))
    throw new TypeError(`${JSON.stringify(text,)} is not a Cargo version.`,);
  /**
   Text before any build label.
   */
  const withoutBuild = text.split('+',)[0] ?? text;
  /**
   Position of the prerelease separator.
   */
  const dashAt = withoutBuild.indexOf('-',);
  /**
   Numeric core text.
   */
  const core = dashAt === (-1) ? withoutBuild : withoutBuild.slice(
    0,
    dashAt,
  );
  /**
   Numeric parts of the core.
   */
  const parts = core.split('.',)
    .map(Number,);
  return {
    major: parts[0] ?? 0,
    minor: parts[1] ?? ABSENT_PART,
    patch: parts[2] ?? ABSENT_PART,
    prerelease: dashAt === (-1) ? [] : withoutBuild.slice(dashAt + 1,)
      .split('.',),
  };
}

//endregion Parsing

//region Comparison

/**
 Whether a prerelease identifier consists only of ASCII digits.

 @param identifier - one dot-separated prerelease identifier

 @returns whether semver treats it as numeric

 @example
 ```ts
 isNumericIdentifier('11'); // true
 isNumericIdentifier('rc'); // false
 ```
 */
function isNumericIdentifier(identifier: string,): boolean {
  if (identifier.length === 0)
    return false;
  for (let index = 0; index < identifier.length; index += 1) {
    /**
     Character at this position.
     */
    const character = identifier.charAt(index,);
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 Compares two prerelease identifiers by semver precedence.

 @param left - identifier

 @param right - identifier

 @returns negative, zero, or positive

 @example
 ```ts
 compareIdentifier({ left: '2', right: '10', }); // negative
 ```
 */
function compareIdentifier({
  left,
  right,
}: {
  readonly left: string;
  readonly right: string
},): number {
  /**
   Whether the left identifier is numeric.
   */
  const leftNumeric = isNumericIdentifier(left,);
  /**
   Whether the right identifier is numeric.
   */
  const rightNumeric = isNumericIdentifier(right,);
  if (leftNumeric && rightNumeric)
    return Number(left,) - Number(right,);
  if (leftNumeric)
    return -1;
  if (rightNumeric)
    return 1;
  return left < right ? -1 : (left > right ? 1 : 0);
}

/**
 Compares two versions by semver precedence; absent parts count as 0, build labels are ignored.

 @param left - version

 @param right - version

 @returns negative when `left` is lower, zero when equal, positive when higher

 @example
 ```ts
 compareVersions({ left: parseVersion('1.0.104'), right: parseVersion('1.0.99'), }); // positive
 ```
 */
export function compareVersions({
  left,
  right,
}: {
  readonly left: ParsedVersion;
  readonly right: ParsedVersion
},): number {
  /**
   Numeric parts paired for comparison.
   */
  const pairs = [
    [
      left.major,
      right.major,
    ],
    [
      Math.max(
        left.minor,
        0,
      ),
      Math.max(
        right.minor,
        0,
      ),
    ],
    [
      Math.max(
        left.patch,
        0,
      ),
      Math.max(
        right.patch,
        0,
      ),
    ],
  ] as const;
  for (const [leftPart, rightPart,] of pairs) {
    if (leftPart !== rightPart)
      return leftPart - rightPart;
  }
  if ((left.prerelease
    .length
    === 0) && (right.prerelease
      .length
      === 0))
    return 0;
  if (left.prerelease
    .length
    === 0)
    return 1;
  if (right.prerelease
    .length
    === 0)
    return -1;
  /**
   Identifier count both prerelease lists share.
   */
  const shared = Math.min(
    left.prerelease
      .length,
    right.prerelease
      .length,
  );
  for (let index = 0; index < shared; index += 1) {
    /**
     Precedence of the identifiers at this position.
     */
    const order = compareIdentifier({
      left: left.prerelease[index] ?? '',
      right: right.prerelease[index] ?? '',
    },);
    if (order !== 0)
      return order;
  }
  return left.prerelease
    .length
    - right.prerelease
      .length;
}

//endregion Comparison

//region Requirements

/**
 Splits a single-comparator requirement into its operator and version.

 @param requirement - requirement such as `>=1.0`, `^0.4`, `~1.2.3`, `=1.18.1`, or bare `1`

 @returns operator (`^` for bare) and parsed version

 @throws TypeError when the requirement is not one of those forms

 @example
 ```ts
 splitRequirement('>=1.0.104'); // { operator: '>=', version: { major: 1, minor: 0, patch: 104, prerelease: [] } }
 ```
 */
export function splitRequirement(requirement: string,): {
  readonly operator: '>=' | '^' | '~' | '=';
  readonly version: ParsedVersion
} {
  /**
   Requirement without surrounding spaces.
   */
  const trimmed = requirement.trim();
  for (const operator of [
    '>=',
    '^',
    '~',
    '=',
  ] as const) {
    if (trimmed.startsWith(operator,)) {
      return {
        operator,
        version: parseVersion(trimmed.slice(operator.length,)
          .trim(),),
      };
    }
  }
  return {
    operator: '^',
    version: parseVersion(trimmed,),
  };
}

/**
 Full release version with the given parts and no prerelease.

 @param major - major part

 @param minor - minor part

 @param patch - patch part

 @returns parsed version for a comparator bound

 @example
 ```ts
 releaseVersion({ major: 0, minor: 5, patch: 0, });
 ```
 */
function releaseVersion(
  {
    major,
    minor,
    patch,
  }: {
    readonly major: number;
    readonly minor: number;
    readonly patch: number;
  },
): ParsedVersion {
  return {
    major,
    minor,
    patch,
    prerelease: [],
  };
}

/**
 Exclusive upper bound of a caret or tilde comparator.

 @param operator - `^` or `~`

 @param floor - comparator version

 @returns smallest version the comparator excludes

 @example
 ```ts
 upperBound({ operator: '^', floor: parseVersion('0.4'), }); // 0.5.0
 ```
 */
function upperBound({
  operator,
  floor,
}: {
  readonly operator: '^' | '~';
  readonly floor: ParsedVersion
},): ParsedVersion {
  if (operator === '~') {
    return floor.minor === ABSENT_PART
      ? releaseVersion({
        major: floor.major + 1,
        minor: 0,
        patch: 0,
      },)
      : releaseVersion({
        major: floor.major,
        minor: floor.minor + 1,
        patch: 0,
      },);
  }
  if ((floor.major > 0) || (floor.minor === ABSENT_PART))
    return releaseVersion({
      major: floor.major + 1,
      minor: 0,
      patch: 0,
    },);
  if ((floor.minor > 0) || (floor.patch === ABSENT_PART))
    return releaseVersion({
      major: 0,
      minor: floor.minor + 1,
      patch: 0,
    },);
  return releaseVersion({
    major: 0,
    minor: 0,
    patch: floor.patch + 1,
  },);
}

/**
 Whether a full version satisfies a single-comparator Cargo requirement.

 @param version - candidate version, usually a locked or published release

 @param requirement - requirement such as `>=1.0`, `^0.4`, `~1.2.3`, `=1.18.1`, or bare `1`

 @returns whether Cargo would accept `version` for `requirement`

 @throws TypeError when either input is malformed

 @example
 ```ts
 satisfies({ version: '1.0.104', requirement: '1', }); // true
 satisfies({ version: '0.62.0', requirement: '0.58', }); // false
 ```
 */
export function satisfies({
  version,
  requirement,
}: {
  readonly version: string;
  readonly requirement: string
},): boolean {
  /**
   Candidate version.
   */
  const candidate = parseVersion(version,);
  /**
   Requirement operator and comparator version.
   */
  const {
    operator,
    version: floor,
  } = splitRequirement(requirement,);
  if (candidate.prerelease
    .length
    > 0) {
    // Cargo: a prerelease only matches a comparator naming a prerelease of the same release.
    /**
     Whether the comparator names a prerelease of the same major.minor.patch.
     */
    const sameRelease = (floor.prerelease
      .length
      > 0) && (floor.major === candidate.major)
      && (floor.minor === candidate.minor)
      && (floor.patch === candidate.patch);
    if (!sameRelease)
      return false;
  }
  /**
   Order of the candidate against the comparator version.
   */
  const fromFloor = compareVersions({
    left: candidate,
    right: floor,
  },);
  if (operator === '>=')
    return fromFloor >= 0;
  if (operator === '=') {
    if (floor.minor === ABSENT_PART)
      return candidate.major === floor.major;
    if (floor.patch === ABSENT_PART)
      return (candidate.major === floor.major) && (candidate.minor === floor.minor);
    return fromFloor === 0;
  }
  return (fromFloor >= 0) && (compareVersions({
    left: candidate,
    right: upperBound({
      operator,
      floor,
    },),
  },) < 0);
}

/**
 Formats a parsed version, padding absent parts with 0.

 @param version - parsed version

 @returns full version text

 @example
 ```ts
 formatVersion(parseVersion('0.4')); // '0.4.0'
 ```
 */
export function formatVersion(version: ParsedVersion,): string {
  /**
   Full numeric core.
   */
  const core = `${String(version.major,)}.${String(Math.max(
    version.minor,
    0,
  ),)}.${String(Math.max(
    version.patch,
    0,
  ),)}`;
  return version.prerelease
    .length
    === 0 ? core : `${core}-${version.prerelease
      .join('.',)}`;
}

//endregion Requirements
