/**
 Model version extraction and comparison helpers.
 
 The parsers scan model ids in a single pass over tokens and avoid regex so
 attacker-controlled or long ids cannot trigger backtracking or recursion.
 
 @module
 */

import type { ModelPricing, } from './types.ts';

//region Constants

/**
 Minimum digit count for a numeric token to be treated as a date stamp.
 */
const DATE_TOKEN_DIGIT_COUNT = 8;

/**
 Sentinel returned by {@link extractMajorVersion} when no version token exists.
 */
export const NO_MAJOR_VERSION: unique symbol = Symbol('major version number absent from model id',);

//endregion Constants

//region Public API

/**
 Extract the major version number from a model id.
 
 Finds the first digit sequence in any token, skipping date-like tokens.
 
 @param id - model id string
 
 @returns major version number, or {@link NO_MAJOR_VERSION} when not found
 
 @example
 ```typescript
 extractMajorVersion('gpt-4o-mini');
 ```
 */
export function extractMajorVersion(
  id: string,
): number | typeof NO_MAJOR_VERSION {
  for (const token of splitModelIdTokens(id,)) {
    if (isDateLikeToken(token,))
      continue;
    /**
     First digit run in current token.
     */
    const digitRun = firstDigitRun(token,);
    if (digitRun !== NO_MAJOR_VERSION)
      return digitRun;
  }
  return NO_MAJOR_VERSION;
}

/**
 Extract version numbers from a model id, skipping date-like tokens.
 
 @param id - model id string
 
 @returns version numbers found in model-id order
 
 @example
 ```typescript
 extractVersionNumbers('claude-3.5-sonnet');
 ```
 */
export function extractVersionNumbers(
  id: string,
): number[] {
  /**
   Accumulator for every per-token version number.
   */
  const numbers: number[] = [];
  for (const token of splitModelIdTokens(id,)) {
    if (isDateLikeToken(token,))
      continue;
    /**
     First digit run in current token.
     */
    const digitRun = firstDigitRun(token,);
    if (digitRun !== NO_MAJOR_VERSION)
      numbers.push(digitRun,);
  }
  return numbers;
}

/**
 Compare two models by extracted version numbers.
 
 Higher version numbers sort first, then shorter vectors sort first so aliases
 win over dated snapshots.
 
 @param a - first model
 
 @param b - second model
 
 @returns negative when `a` ranks before `b`
 
 @example
 ```typescript
 compareVersions({ a: modelA, b: modelB });
 ```
 */
export function compareVersions<TModel extends Pick<ModelPricing, 'id'>,>(
  {
    a,
    b,
  }: {
    readonly a: TModel;
    readonly b: TModel;
  },
): number {
  /**
   Version vector for `a`, e.g. `[3, 5]` for `claude-3.5-sonnet`.
   */
  const leftVersions = extractVersionNumbers(a.id,);
  /**
   Version vector for `b`, compared positionally against {@link leftVersions}.
   */
  const rightVersions = extractVersionNumbers(b.id,);
  /**
   Number of positions to walk.
   */
  const maxLength = Math.max(
    leftVersions.length,
    rightVersions.length,
  );
  for (let loopIndex = 0; loopIndex < maxLength; loopIndex++) {
    /**
     Per-position delta with `b` first so higher versions sort earlier.
     */
    const diff = (rightVersions[loopIndex]
      ?? 0) - (leftVersions[loopIndex]
        ?? 0);
    if (diff !== 0)
      return diff;
  }
  return leftVersions.length
    - rightVersions
      .length;
}

/**
 Find cheapest models across top major-version groups.
 
 `majorVersions`: one keeps latest only, two keeps latest plus previous, zero
 keeps all major versions.
 
 @param models - models to filter and sort
 
 @param majorVersions - number of major-version families to keep
 
 @returns cheapest models sorted by input cost then version
 
 @example
 ```typescript
 findCheapestInMajorVersions({ models, majorVersions: 1 });
 ```
 */
export function findCheapestInMajorVersions<TModel extends ModelPricing,>(
  {
    models,
    majorVersions,
  }: {
    readonly models: readonly TModel[];
    readonly majorVersions: number;
  },
): TModel[] {
  /**
   Distinct major-version numbers seen across candidate set.
   */
  const allVersions = new Set<number>();
  for (const model of models) {
    /**
     Major version extracted from current model id.
     */
    const version = extractMajorVersion(model.id,);
    if (version !== NO_MAJOR_VERSION)
      allVersions.add(version,);
  }

  /**
   Major versions in descending order.
   */
  const sortedVersions = [...allVersions,]
    .toSorted(function sortDescending(
      left,
      right,
    ) {
      return right - left;
    },);
  if (sortedVersions.length
    === 0)
    return [];

  /**
   Subset of major versions to keep.
   */
  const included = majorVersions === 0
    ? sortedVersions
    : sortedVersions.slice(
      0,
      majorVersions,
    );
  /**
   Set form for O(1) membership checks.
   */
  const includedSet = new Set(included,);

  /**
   Models whose major version is in the kept set.
   */
  const eligible = models.filter(function hasIncludedVersion(model,) {
    /**
     Per-model major version used for membership.
     */
    const version = extractMajorVersion(model.id,);
    return (version !== NO_MAJOR_VERSION) && includedSet
      .has(version,);
  },);

  return eligible.toSorted(function byCostThenVersion(
    left,
    right,
  ) {
    /**
     Cost-only ordering before version tie-break.
     */
    const costDiff = left.cost
      .input
      - right
        .cost
        .input;
    if (costDiff !== 0)
      return costDiff;
    return compareVersions({
      a: left,
      b: right,
    },);
  },);
}

//endregion Public API

//region Token scanning

/**
 Split a model id into tokens separated by punctuation or whitespace.
 
 @param id - model id to split
 
 @returns non-empty tokens
 */
function splitModelIdTokens(
  id: string,
): string[] {
  /**
   Completed model-id tokens.
   */
  const tokens: string[] = [];
  /**
   Characters collected for current token.
   */
  let currentTokenCharacters: string[] = [];
  for (const character of id) {
    if (isModelIdSeparator(character,)) {
      if (currentTokenCharacters.length
        > 0) {
        tokens.push(currentTokenCharacters.join('',),);
        currentTokenCharacters = [];
      }
      continue;
    }
    currentTokenCharacters.push(character,);
  }
  if (currentTokenCharacters.length
    > 0)
    tokens.push(currentTokenCharacters.join('',),);
  return tokens;
}

/**
 Check whether a character separates version tokens.
 
 @param character - single character from a model id
 
 @returns whether character is a token separator
 */
function isModelIdSeparator(
  character: string,
): boolean {
  return (character === '.')
    || (character === '_')
    || (character === '-')
    || (character === ':')
    || isAsciiWhitespace(character,);
}

/**
 Check whether a token is an all-digit date-like token.
 
 @param token - model-id token
 
 @returns whether token should be skipped as a date
 */
function isDateLikeToken(
  token: string,
): boolean {
  return (token.length >= DATE_TOKEN_DIGIT_COUNT) && containsOnlyDigits(token,);
}

/**
 Return the first digit run inside a token.
 
 @param token - model-id token
 
 @returns parsed digit run, or {@link NO_MAJOR_VERSION} when absent
 */
function firstDigitRun(
  token: string,
): number | typeof NO_MAJOR_VERSION {
  /**
   Digit characters for current run.
   */
  const digits: string[] = [];
  for (const character of token) {
    if (isAsciiDigit(character,)) {
      digits.push(character,);
      continue;
    }
    if (digits.length
      > 0)
      return Math.trunc(Number(
        digits.join('',),
      ),);
  }
  return digits.length
    > 0 ? Math.trunc(Number(
      digits.join('',),
    ),) : NO_MAJOR_VERSION;
}

/**
 Check whether every character in a string is an ASCII digit.
 
 @param value - value to check
 
 @returns whether value contains only digits and is non-empty
 */
function containsOnlyDigits(
  value: string,
): boolean {
  if (value === '')
    return false;
  for (const character of value) {
    if (!isAsciiDigit(character,))
      return false;
  }
  return true;
}

/**
 Check whether a character is an ASCII digit.
 
 @param character - character to check
 
 @returns whether character is between zero and nine inclusive
 */
function isAsciiDigit(
  character: string,
): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Check whether a character is ASCII whitespace.
 
 @param character - character to check
 
 @returns whether character is whitespace used by model ids
 */
function isAsciiWhitespace(
  character: string,
): boolean {
  return (character === ' ')
    || (character === '\t')
    || (character === '\n')
    || (character === '\r')
    || (character === '\f')
    || (character === '\v');
}

//endregion Token scanning
