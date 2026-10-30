import {
  decimalDigits,
  usesOnly,
} from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/character-set.ts';

import {
  maximumVersionLength,
  versionIdentifierCharacters,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 Number of dot-separated numbers in the core of a version: major, minor and patch.
 */
const coreNumberCount = 3;

/**
 Check a numeric identifier: digits only, and no leading zero unless the identifier is `0`.

 @param identifier - text under test

 @returns true for `0`, `4` and `10`; false for `04`, the empty text and anything with other characters

 @example
 ```ts
 isNumericIdentifier('10');
 ```
 */
function isNumericIdentifier(identifier: string): boolean {
  return usesOnly({
    text: identifier,
    allowed: decimalDigits
  })
    && ((identifier === '0') || (!identifier.startsWith('0')));
}

/**
 Check a list of dot-separated identifiers of the prerelease or build part: each non-empty, of letters, digits and
 hyphens, and, in a prerelease, a number-only identifier without a leading zero.

 @param part - text after the first hyphen or plus sign

 @param rejectLeadingZero - true for the prerelease part, where `01` is not a valid number

 @returns true when every identifier fits

 @example
 ```ts
 areIdentifiers({ part: 'rc.1', rejectLeadingZero: true });
 ```
 */
function areIdentifiers({
  part,
  rejectLeadingZero,
}: {
  readonly part: string;
  readonly rejectLeadingZero: boolean;
}): boolean {
  return part.split('.')
    .every(function fits(identifier) {
      return usesOnly({
        text: identifier,
        allowed: versionIdentifierCharacters
      })
        && ((!rejectLeadingZero) || (!usesOnly({
          text: identifier,
          allowed: decimalDigits
        }))
          || isNumericIdentifier(identifier));
    });
}

/**
 Check that a text is a Semantic Versioning version as Cargo reads it: three numbers, an optional prerelease after a
 hyphen and an optional build after a plus sign.

 The check is one linear pass of splits and character scans, so it is safe on unbounded input, and it accepts no
 character that would change meaning in a URL, a Git tag, a shell word or a manifest line.

 @param text - value under test

 @returns true for `0.4.10`, `1.0.0-rc.1` and `1.0.0+build.5`; false for everything else

 @example
 ```ts
 isReleaseVersion('0.4.10');
 ```
 */
export function isReleaseVersion(text: string): boolean {
  if (text.length > maximumVersionLength)
    return false;
  /**
   Text before and after the only plus sign allowed.
   */
  const [withoutBuild, build, ...extra] = text.split('+');
  if ((withoutBuild === undefined) || (extra.length > 0))
    return false;
  if ((build !== undefined) && (!areIdentifiers({
    part: build,
    rejectLeadingZero: false
  })))
    return false;
  /**
   Position of the hyphen that starts the prerelease.
   */
  const hyphen = withoutBuild.indexOf('-');
  /**
   The three numbers.
   */
  const core = (hyphen === (-1)) ? withoutBuild : withoutBuild.slice(
    0,
    hyphen
  );
  /**
   The numbers, each checked.
   */
  const numbers = core.split('.');
  if ((numbers.length !== coreNumberCount) || (!numbers.every(isNumericIdentifier)))
    return false;
  return (hyphen === (-1)) || areIdentifiers({
    part: withoutBuild.slice(hyphen + 1),
    rejectLeadingZero: true
  });
}

/**
 Accept a release version or refuse it with a message that names the option.

 @param option - option or argument name for the message

 @param value - text under test

 @returns the value unchanged

 @throws CrateReleaseError when the value is not a release version

 @example
 ```ts
 assertReleaseVersion({ option: '<to>', value: '0.4.10' });
 ```
 */
export function assertReleaseVersion({
  option,
  value,
}: {
  readonly option: string;
  readonly value: string;
}): string {
  if (!isReleaseVersion(value))
    throw new CrateReleaseError(
      `${option} must be a version such as 0.4.10 or 1.0.0-rc.1 (three numbers, then an optional -prerelease and +build), got ${JSON.stringify(value)}.`,
    );
  return value;
}
