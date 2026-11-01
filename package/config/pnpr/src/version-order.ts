//region Semantic version precedence

/**
 Number of numeric components before any prerelease: major, minor, patch.
 */
const SEMVER_CORE_LENGTH = 3;

/**
 Reports whether a prerelease identifier consists only of ASCII digits.

 @param identifier - One dot-separated prerelease identifier.

 @returns Whether semver compares it numerically.

 @example
 ```ts
 isNumericIdentifier('10');
 // => true
 ```
 */
function isNumericIdentifier(identifier: string,): boolean {
  if (identifier === '')
    return false;
  // Index scan over UTF-16 units: any non-ASCII unit is not a digit, so no code-point splitting is needed.
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
 Parsed semantic version parts that decide precedence; build metadata is ignored.
 */
type ParsedVersion = {
  /**
   Numeric major, minor, and patch components.
   */
  readonly core: readonly [
    number,
    number,
    number,
  ];
  /**
   Dot-separated prerelease identifiers, empty for a release.
   */
  readonly prerelease: readonly string[];
};

/**
 Splits a semantic version into numeric core and prerelease identifiers.

 @param version - Version string from a manifest or packument.

 @returns Parsed parts used for precedence comparison.

 @throws Error when the core is not three non-negative integers.

 @example
 ```ts
 parseVersion('1.2.3-alpha.1+build');
 // => { core: [1, 2, 3], prerelease: ['alpha', '1'] }
 ```
 */
function parseVersion(version: string,): ParsedVersion {
  /**
   Version without build metadata, which never affects precedence.
   */
  const withoutBuild = version.split('+',)[0] ?? '';
  /**
   Position of the first hyphen, which starts the prerelease section.
   */
  const hyphenIndex = withoutBuild.indexOf('-',);
  /**
   Dot-separated numeric core text.
   */
  const coreText = hyphenIndex === (-1) ? withoutBuild : withoutBuild.slice(
    0,
    hyphenIndex,
  );
  /**
   Prerelease identifiers after the first hyphen.
   */
  const prerelease = hyphenIndex === (-1) ? [] : withoutBuild.slice(hyphenIndex + 1,)
    .split('.',);
  /**
   Core components parsed as integers.
   */
  const coreParts = coreText.split('.',)
    .map(function parseCoreComponent(part,) {
      // Number('') is 0, which would accept `1..3`; an empty component must fail the integer check.
      return part === '' ? Number.NaN : Number(part,);
    },);
  /**
   Major, minor, and patch, each undefined when the core is too short.
   */
  const [
    major,
    minor,
    patch,
  ] = coreParts;
  if (
    (coreParts.length !== SEMVER_CORE_LENGTH)
      || (major === undefined)
      || (minor === undefined)
      || (patch === undefined)
      || (!coreParts.every(function isNonNegativeInteger(part,) {
        return Number.isSafeInteger(part,) && (part >= 0);
      },))
  )
    throw new Error(`not a semantic version: ${JSON.stringify(version,)}`,);
  return {
    core: [
      major,
      minor,
      patch,
    ],
    prerelease,
  };
}

/**
 Orders two prerelease identifiers: numeric ones numerically and below alphanumeric ones, others by ASCII.

 @param left - Identifier from the first version.

 @param right - Identifier from the second version.

 @returns Negative, zero, or positive like `Array#sort` comparators.

 @example
 ```ts
 compareIdentifiers({ left: '2', right: '10' });
 // => negative
 ```
 */
function compareIdentifiers({
  left,
  right,
}: {
  readonly left: string;
  readonly right: string
},): number {
  /**
   Whether the left identifier is purely numeric.
   */
  const leftIsNumeric = isNumericIdentifier(left,);
  /**
   Whether the right identifier is purely numeric.
   */
  const rightIsNumeric = isNumericIdentifier(right,);
  if (leftIsNumeric && rightIsNumeric)
    return Number(left,) - Number(right,);
  if (leftIsNumeric)
    return -1;
  if (rightIsNumeric)
    return 1;
  if (left === right)
    return 0;
  return left < right ? -1 : 1;
}

/**
 Compares two semantic versions by precedence, so a registry's `latest` tag only moves forward.

 @param left - First version.

 @param right - Second version.

 @returns Negative when left precedes right, zero when equal in precedence, positive otherwise.

 @throws Error when either version is not semantic.

 @example
 ```ts
 compareVersions({ left: '1.0.0-alpha', right: '1.0.0' });
 // => negative
 ```
 */
export function compareVersions({
  left,
  right,
}: {
  readonly left: string;
  readonly right: string
},): number {
  /**
   Parsed first version.
   */
  const leftParts = parseVersion(left,);
  /**
   Parsed second version.
   */
  const rightParts = parseVersion(right,);
  for (let index = 0; index < SEMVER_CORE_LENGTH; index += 1) {
    /**
     Difference in this core component.
     */
    const difference = (leftParts.core[index] ?? 0) - (rightParts.core[index] ?? 0);
    if (difference !== 0)
      return difference;
  }
  if ((leftParts.prerelease
    .length
    === 0) || (rightParts.prerelease
      .length
      === 0))
    return rightParts.prerelease
      .length
      - leftParts.prerelease
        .length;
  /**
   Identifier pairs to compare before length decides.
   */
  const sharedLength = Math.min(
    leftParts.prerelease
      .length,
    rightParts.prerelease
      .length,
  );
  for (let index = 0; index < sharedLength; index += 1) {
    /**
     Order of this identifier pair.
     */
    const order = compareIdentifiers({
      left: leftParts.prerelease[index] ?? '',
      right: rightParts.prerelease[index] ?? '',
    },);
    if (order !== 0)
      return order;
  }
  return leftParts.prerelease
    .length
    - rightParts.prerelease
      .length;
}

/**
 Picks the dist-tag for a new version so `latest` never moves backwards.

 @param version - Version about to publish.

 @param currentLatest - Registry's current `latest`, absent for a new package.

 @returns `latest` when the version is newer than the current latest, otherwise `backfill`.

 @example
 ```ts
 chooseDistTag({ version: '0.1.0', currentLatest: '0.2.0' });
 // => 'backfill'
 ```
 */
export function chooseDistTag(
  {
    version,
    currentLatest,
  }: {
    readonly version: string;
    readonly currentLatest?: string;
  },
): 'latest' | 'backfill' {
  if (currentLatest === undefined)
    return 'latest';
  return compareVersions({
    left: version,
    right: currentLatest,
  },) > 0 ? 'latest' : 'backfill';
}

//endregion Semantic version precedence
