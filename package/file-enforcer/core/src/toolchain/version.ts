/**
 `version.ts`: normalizes version text and decides whether an installed version satisfies a locked one.
 */

//region Version comparison

/**
 Letters and digits a release-tag prefix (`bun-v1.3.14`) may contain besides its separators.
 */
const TAG_PREFIX_CHARACTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._-';

/**
 ASCII letters, which a release-tag prefix must start with.
 */
const ASCII_LETTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';

/**
 ASCII digits.
 */
const ASCII_DIGITS = '0123456789';

/**
 Whether text is exactly one ASCII digit.

 @param ch - Text to test

 @returns `true` for `0` to `9`
 */
function isAsciiDigit(ch: string,): boolean {
  return (ch.length === 1) && ASCII_DIGITS.includes(ch,);
}

/**
 Index of the character that ends the leading release-tag prefix run: a letter followed by letters, digits,
 `.`, `_` and `-`.

 @param version - Raw version text

 @returns Length of the prefix run; zero when the text does not start with a letter
 */
function tagPrefixRunLength(version: string,): number {
  if ((version.length === 0) || (!ASCII_LETTERS.includes(version.charAt(0,),)))
    return 0;
  for (let index = 1; index < version.length; index += 1) {
    if (!TAG_PREFIX_CHARACTERS.includes(version.charAt(index,),))
      return index;
  }
  return version.length;
}

/**
 Normalizes a version string for comparison: drops one leading `v`, and the
 release-tag prefix before it (`bun-v1.3.14` becomes `1.3.14`).

 Release tags carry prefixes (`v1.9.0`, `bun-v1.3.14`); the lock holds the bare version. Anything without a
 v-before-digit stays as written. When several `-v<digit>` boundaries could end the prefix, the last one wins.

 @param version - Raw version text

 @returns Normalized version text

 @example
 ```ts
 normalizeVersion('bun-v1.3.14',); // '1.3.14'
 ```
 */
export function normalizeVersion(version: string,): string {
  /**
   Length of the leading run that may form a tag prefix.
   */
  const runLength = tagPrefixRunLength(version,);
  // A prefix is a letter, more prefix characters, then a `-` right before `v<digit>`.
  for (let dash = runLength - 1; dash >= 1; dash -= 1) {
    if ((version.charAt(dash,) === '-')
      && (version.charAt(dash + 1,) === 'v')
      && isAsciiDigit(version.charAt(dash + 2,),))
      return version.slice(dash + 2,);
  }
  return (version.startsWith('v',) && isAsciiDigit(version.charAt(1,),))
    ? version.slice(1,)
    : version;
}

/**
 Whether an observed version satisfies a locked version:
 equal after normalization, or the observed value extends the lock with a
 platform suffix (`nightly-2026-10-21-x86_64-unknown-linux-gnu`).

 @param observed - Version text the machine reports

 @param locked - Version text the lock records

 @returns `true` when the observed version matches the lock

 @example
 ```ts
 versionMatches({ observed: 'v1.9.0', locked: '1.9.0', },); // true
 ```
 */
export function versionMatches(
  {
    observed,
    locked,
  }: {
    readonly observed: string;
    readonly locked: string;
  },
): boolean {
  /**
   Normalized observed version.
   */
  const o = normalizeVersion(observed,);
  /**
   Normalized locked version.
   */
  const l = normalizeVersion(locked,);
  return (o === l) || o.startsWith(`${l}-`,)
    || l.startsWith(`${o}-`,);
}

//endregion Version comparison
