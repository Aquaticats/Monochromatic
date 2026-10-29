/**
 String-scan validators for crate names and version requirements.

 Written as character scans rather than regular expressions, matching the
 repository's no-regex lint rule.

 @module
 */

//region Characters

/**
 Longest crate name crates.io accepts.
 */
const MAX_CRATE_NAME_LENGTH = 64;

/**
 Whether `character` is an ASCII letter.

 @param character - single character to test

 @returns whether it is `a` to `z` or `A` to `Z`

 @example
 ```ts
 isAsciiLetter('a'); // true
 ```
 */
function isAsciiLetter(character: string,): boolean {
  return ((character >= 'a') && (character <= 'z')) || ((character >= 'A') && (character <= 'Z'));
}

/**
 Whether `character` is an ASCII digit.

 @param character - single character to test

 @returns whether it is `0` to `9`

 @example
 ```ts
 isAsciiDigit('7'); // true
 ```
 */
function isAsciiDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

//endregion Characters

//region Crate names

/**
 Whether `name` is a valid Cargo dependency or package name.

 Rules: one to 64 characters, ASCII letters, digits, `-`, or `_`, starting with
 a letter (the crates.io publishing rule).

 @param name - candidate name

 @returns whether Cargo and crates.io accept it

 @example
 ```ts
 isCrateName('tracing-subscriber'); // true
 isCrateName('__proto__'); // false
 ```
 */
export function isCrateName(name: string,): boolean {
  if ((name.length === 0) || (name.length > MAX_CRATE_NAME_LENGTH))
    return false;
  if (!isAsciiLetter(name.charAt(0,),))
    return false;
  return [...name,].every(function isNameCharacter(character,): boolean {
    return isAsciiLetter(character,) || isAsciiDigit(character,)
      || (character === '-')
      || (character === '_');
  },);
}

//endregion Crate names

//region Versions

/**
 Whether `part` is a run of one or more digits without a leading zero (except `0` itself).

 @param part - one dot-separated version component

 @returns whether it is a semver numeric identifier

 @example
 ```ts
 isNumericComponent('104'); // true
 isNumericComponent('01'); // false
 ```
 */
function isNumericComponent(part: string,): boolean {
  if (part.length === 0)
    return false;
  if ((part.length > 1) && part.startsWith('0',))
    return false;
  return [...part,].every(isAsciiDigit,);
}

/**
 Whether `label` is a non-empty run of ASCII letters, digits, `.`, and `-`.

 @param label - prerelease or build label after `-` or `+`

 @returns whether semver accepts the label's characters

 @example
 ```ts
 isLabel('alpha.5'); // true
 ```
 */
function isLabel(label: string,): boolean {
  if (label.length === 0)
    return false;
  return [...label,].every(function isLabelCharacter(character,): boolean {
    return isAsciiLetter(character,) || isAsciiDigit(character,)
      || (character === '.')
      || (character === '-');
  },);
}

/**
 Whether `version` is a Cargo version with one to three numeric parts and optional labels.

 Partial versions (`1`, `0.4`) are what Cargo requirements usually carry, so
 they are accepted.

 @param version - version text without an operator

 @returns whether it parses as a partial or full semver version

 @example
 ```ts
 isVersion('1.0.104'); // true
 isVersion('0.4'); // true
 isVersion('1.0.0-alpha.5'); // true
 isVersion('1.x'); // false
 ```
 */
export function isVersion(version: string,): boolean {
  /**
   Position of the build label, which may follow a prerelease label.
   */
  const buildAt = version.indexOf('+',);
  /**
   Version without its build label.
   */
  const withoutBuild = buildAt === (-1) ? version : version.slice(
    0,
    buildAt,
  );
  if ((buildAt !== (-1)) && (!isLabel(version.slice(buildAt + 1,),)))
    return false;
  /**
   Position of the prerelease label.
   */
  const prereleaseAt = withoutBuild.indexOf('-',);
  /**
   Dot-separated numeric core.
   */
  const core = prereleaseAt === (-1) ? withoutBuild : withoutBuild.slice(
    0,
    prereleaseAt,
  );
  if ((prereleaseAt !== (-1)) && (!isLabel(withoutBuild.slice(prereleaseAt + 1,),)))
    return false;
  /**
   Numeric components of the core.
   */
  const parts = core.split('.',);
  if ((parts.length === 0) || (parts.length > 3))
    return false;
  return parts.every(isNumericComponent,);
}

/**
 Whether `requirement` is a `>=` floor such as `>=1.0.104`.

 @param requirement - Cargo version requirement

 @returns whether it is exactly one `>=` comparator on a valid version

 @example
 ```ts
 isFloorRequirement('>=1.0.104'); // true
 isFloorRequirement('1.0.104'); // false
 ```
 */
export function isFloorRequirement(requirement: string,): boolean {
  if (!requirement.startsWith('>=',))
    return false;
  return isVersion(requirement.slice(2,)
    .trim(),);
}

/**
 Operator prefixes a bounded requirement may start with, longest first.
 */
const BOUNDED_OPERATORS = [
  '=',
  '^',
  '~',
] as const;

/**
 Whether `requirement` is bounded above: caret (bare or `^`), tilde, or exact.

 @param requirement - Cargo version requirement

 @returns whether it is one bounded comparator on a valid version

 @example
 ```ts
 isBoundedRequirement('0.58'); // true
 isBoundedRequirement('=1.18.1'); // true
 isBoundedRequirement('>=0.58'); // false
 ```
 */
export function isBoundedRequirement(requirement: string,): boolean {
  /**
   Operator the requirement starts with, if any.
   */
  const operator = BOUNDED_OPERATORS.find(function startsWithOperator(candidate,): boolean {
    return requirement.startsWith(candidate,);
  },);
  /**
   Version text after the operator.
   */
  const version = operator === undefined ? requirement : requirement.slice(operator.length,);
  return isVersion(version.trim(),);
}

/**
 Version a `>=` floor names.

 @param requirement - `>=` floor requirement

 @returns version text after `>=`

 @throws {TypeError} when `requirement` is not a `>=` floor

 @example
 ```ts
 floorVersion('>=1.0.104'); // '1.0.104'
 ```
 */
export function floorVersion(requirement: string,): string {
  if (!isFloorRequirement(requirement,))
    throw new TypeError(`${JSON.stringify(requirement,)} is not a \`>=\` floor requirement.`,);
  return requirement.slice(2,)
    .trim();
}

//endregion Versions
