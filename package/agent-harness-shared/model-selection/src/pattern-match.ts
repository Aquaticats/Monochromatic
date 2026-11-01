/**
 Pi-style model pattern parsing and fuzzy matching helpers.
 
 @module
 */

import { canonicalSlug, } from './model-id.ts';
import {
  findExactModelReferenceMatch,
  NO_EXACT_MATCH,
} from './exact-match.ts';
import type {
  ModelIdentity,
  ScopedModel,
  ScopedThinkingLevel,
} from './types.ts';

/**
 Sentinel returned by internal {@link tryMatchModel} when no exact or fuzzy model
 matches a pattern body. A `unique symbol`; narrowed with
 `=== NO_PATTERN_MATCH`.
 */
const NO_PATTERN_MATCH: unique symbol = Symbol('model-selection/no-pattern-match',);

/**
 Sentinel seeding the thinking-level accumulator in {@link parseModelPattern}
 before a valid thinking suffix is seen. A `unique symbol` rather than
 `undefined` because the loop-init binding would otherwise need a banned
 `ScopedThinkingLevel | undefined` annotation; narrowed with
 `=== NO_THINKING_LEVEL`.
 */
const NO_THINKING_LEVEL: unique symbol = Symbol('model-selection/no-thinking-level',);

//region Types and constants

/**
 Result of parsing a single model scope pattern.
 */
export type PatternResolution<TModel extends ModelIdentity = ModelIdentity,> = {
  /**
   Matched model, omitted when no model matched.
   */
  readonly model?: TModel;
  /**
   Thinking level suffix, when present.
   */
  readonly thinkingLevel?: ScopedThinkingLevel;
};

/**
 Valid pi thinking-level suffixes.
 */
const THINKING_LEVELS = [
  'off',
  'minimal',
  'low',
  'medium',
  'high',
  'xhigh',
] as const satisfies readonly ScopedThinkingLevel[];

/**
 Valid pi thinking-level suffixes as strings for runtime checks.
 */
const THINKING_LEVEL_SET: ReadonlySet<string> = new Set(THINKING_LEVELS,);

/**
 Digits in pi model date suffixes.
 */
const DATE_SUFFIX_DIGITS = 8;

/**
 Total characters in pi model date suffix including leading hyphen.
 */
const DATE_SUFFIX_TOTAL_LENGTH = DATE_SUFFIX_DIGITS + 1;

/**
 ASCII digit characters accepted in pi model date suffixes.
 */
const ASCII_DIGITS: ReadonlySet<string> = new Set([
  '0',
  '1',
  '2',
  '3',
  '4',
  '5',
  '6',
  '7',
  '8',
  '9',
],);

//endregion Types and constants

//region Public helpers

/**
 Parse exact or fuzzy pi model pattern.
 
 The parser strips colon suffixes iteratively to avoid recursion over flat
 strings. A valid thinking suffix is carried while earlier suffixes are
 stripped until a model body matches or no body remains.
 
 @param pattern - pi model pattern
 
 @param availableModels - models with configured auth
 
 @returns matched model and optional thinking level
 
 @example
 ```typescript
 parseModelPattern({ pattern: 'sonnet:high', availableModels });
 ```
 */
export function parseModelPattern<TModel extends ModelIdentity,>(
  {
    pattern,
    availableModels,
  }: {
    readonly pattern: string;
    readonly availableModels: readonly TModel[];
  },
): PatternResolution<TModel> {
  for (
    let currentPattern = pattern, thinkingLevel: ScopedThinkingLevel | typeof NO_THINKING_LEVEL = NO_THINKING_LEVEL;
    ;
  ) {
    /**
     Exact or fuzzy match for current pattern body.
     */
    const exactMatch = tryMatchModel({
      pattern: currentPattern,
      availableModels,
    },);
    if (exactMatch !== NO_PATTERN_MATCH) {
      return {
        model: exactMatch,
        ...(thinkingLevel === NO_THINKING_LEVEL ? {} : { thinkingLevel, }),
      };
    }

    /**
     Last colon, used for optional thinking suffix parsing.
     */
    const lastColonIndex = currentPattern.lastIndexOf(':',);
    if (lastColonIndex === (-1))
      return {};

    /**
     Candidate suffix after last colon.
     */
    const suffix = currentPattern.slice(lastColonIndex + 1,);
    if ((thinkingLevel === NO_THINKING_LEVEL) && isThinkingLevel(suffix,))
      thinkingLevel = suffix;
    currentPattern = currentPattern.slice(
      0,
      lastColonIndex,
    );
  }
}

/**
 Split a glob pattern from optional thinking suffix.
 
 @param pattern - glob pattern
 
 @returns pattern body and optional thinking level
 
 @example
 ```typescript
 splitThinkingSuffix('anthropic/*:high');
 ```
 */
export function splitThinkingSuffix(
  pattern: string,
): {
  readonly pattern: string;
  readonly thinkingLevel?: ScopedThinkingLevel;
} {
  /**
   Last colon, used for optional suffix parsing.
   */
  const colonIndex = pattern.lastIndexOf(':',);
  if (colonIndex === (-1))
    return { pattern, };

  /**
   Candidate suffix after last colon.
   */
  const suffix = pattern.slice(colonIndex + 1,);
  return isThinkingLevel(suffix,)
    ? {
      pattern: pattern.slice(
        0,
        colonIndex,
      ),
      thinkingLevel: suffix,
    }
    : { pattern, };
}

/**
 Check whether a string contains glob syntax.
 
 @param pattern - model pattern
 
 @returns whether pattern contains glob syntax
 
 @example
 ```typescript
 patternHasGlob('anthropic/*');
 ```
 */
export function patternHasGlob(
  pattern: string,
): boolean {
  return [
    '*',
    '?',
    '[',
  ]
    .some(function patternIncludes(token,) {
      return pattern.includes(token,);
    },);
}

/**
 Convert model to scoped model entry.
 
 @param model - pi model object
 
 @param thinkingLevel - optional thinking level from scope pattern
 
 @returns scoped model
 
 @example
 ```typescript
 scopedModelFromModel({ model });
 ```
 */
export function scopedModelFromModel<TModel extends ModelIdentity,>(
  {
    model,
    thinkingLevel,
  }: {
    readonly model: TModel;
    readonly thinkingLevel?: ScopedThinkingLevel;
  },
): ScopedModel<TModel> {
  return {
    model,
    canonicalSlug: canonicalSlug(model,),
    ...(thinkingLevel === undefined ? {} : { thinkingLevel, }),
  };
}

/**
 Check whether a suffix is a valid thinking level.
 
 @param value - candidate suffix
 
 @returns whether suffix is a valid thinking level
 
 @example
 ```typescript
 isThinkingLevel('high');
 ```
 */
export function isThinkingLevel(
  value: string,
): value is ScopedThinkingLevel {
  return THINKING_LEVEL_SET.has(value,);
}

/**
 Check whether a model id is treated as an alias by pi.
 
 @param id - model id
 
 @returns whether model id is an alias
 
 @example
 ```typescript
 isAlias('claude-sonnet-latest');
 ```
 */
export function isAlias(
  id: string,
): boolean {
  return (id.endsWith('-latest',))
    || (!hasDateSuffix(id,));
}

/**
 Check whether a model id ends with pi's date suffix shape.
 
 @param id - model id
 
 @returns whether id ends with hyphen followed by eight ASCII digits
 
 @example
 ```typescript
 hasDateSuffix('claude-sonnet-20251001');
 ```
 */
export function hasDateSuffix(
  id: string,
): boolean {
  if (id.length
    < DATE_SUFFIX_TOTAL_LENGTH)
    return false;

  /**
   Index of leading hyphen for suffix candidate.
   */
  const suffixHyphenIndex = id.length
    - DATE_SUFFIX_TOTAL_LENGTH;
  if (id.at(suffixHyphenIndex,)
    !== '-')
    return false;

  /**
   Candidate date suffix without leading hyphen.
   */
  const dateSuffix = id.slice(suffixHyphenIndex + 1,);
  return isAsciiDigitString(dateSuffix,);
}

//endregion Public helpers

//region Internal helpers

/**
 Try exact and fuzzy model matching.
 
 @param pattern - pi model pattern
 
 @param availableModels - models with configured auth
 
 @returns matched model, if any
 */
function tryMatchModel<TModel extends ModelIdentity,>(
  {
    pattern,
    availableModels,
  }: {
    readonly pattern: string;
    readonly availableModels: readonly TModel[];
  },
): TModel | typeof NO_PATTERN_MATCH {
  /**
   Exact match by canonical slug or bare id.
   */
  const exact = findExactModelReferenceMatch({
    modelReference: pattern,
    availableModels,
  },);
  if (exact !== NO_EXACT_MATCH)
    return exact;

  /**
   Lowercase pattern for fuzzy id and name matching.
   */
  const normalizedPattern = pattern.toLowerCase();
  /**
   Fuzzy matches by model id or display name.
   */
  const matches = availableModels.filter(function fuzzyMatches(model,) {
    return model
      .id
      .toLowerCase()
      .includes(normalizedPattern,)
      || model
        .name
        .toLowerCase()
        .includes(normalizedPattern,);
  },);
  if (matches.length
    === 0)
    return NO_PATTERN_MATCH;

  /**
   Alias matches, preferred over dated versions.
   */
  const aliases = matches.filter(function keepAlias(model,) {
    return isAlias(model.id,);
  },);
  /**
   Candidate list before sorting.
   */
  const candidates = aliases.length
    > 0 ? aliases : matches;
  /**
   Candidate list sorted by pi's descending id tie-break.
   */
  const sortedCandidates = candidates.toSorted(function compareByIdDesc(
    left,
    right,
  ) {
    return right.id
      .localeCompare(left.id,);
  },);
  /**
   First candidate after sorting.
   */
  const [firstCandidate,] = sortedCandidates;
  return firstCandidate ?? NO_PATTERN_MATCH;
}

/**
 Check whether bounded string contains only ASCII digits.
 
 @param value - candidate digit string
 
 @returns whether every character is an ASCII digit
 */
function isAsciiDigitString(
  value: string,
): boolean {
  for (const character of value) {
    if (!isAsciiDigit(character,))
      return false;
  }
  return true;
}

/**
 Check whether one character is an ASCII digit.
 
 @param character - single-character string
 
 @returns whether character is between zero and nine inclusive
 */
function isAsciiDigit(
  character: string,
): boolean {
  return ASCII_DIGITS.has(character,);
}

//endregion Internal helpers
