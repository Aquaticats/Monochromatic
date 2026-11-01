/**
 TypeScript fork of [`matcher`](https://github.com/sindresorhus/matcher) by
 Sindre Sorhus (MIT): simple wildcard matching.
 
 Rewritten to this repository's TypeScript standards with the upstream
 `matcher` 6.1.0 matching semantics preserved: `*` matches zero or more
 characters (including newlines), a backslash escapes the next character, a
 leading `!` negates the pattern, and matching folds case unless
 `caseSensitive` is set.
 
 @module
 */

import {
  sanitizeInputs,
  sanitizePatterns,
} from './inputs.ts';
import {
  type CompiledMatchers,
  compileMatchers,
} from './matcher.ts';
import {
  type MatcherOptions,
  resolveMatcherOptions,
} from './matcher-options.ts';

//region Entry points

/**
 Filters inputs to those matching any of the patterns.
 
 Accepts a string or a readonly string list for both `inputs` and `patterns`.
 An input is kept when it hits a normal pattern (or when no normal pattern
 is present) and hits no negated pattern. Under `allPatterns`, an input is
 kept only when it hits every normal pattern and no negated pattern.
 
 @param inputs - Input string or readonly list of input strings.
 
 @param patterns - Pattern string or readonly list of pattern strings. `*`
 matches zero or more characters; a leading `!` negates the pattern.
 
 @param options - Case-sensitivity and all-patterns flags.
 
 @returns Kept inputs in input order, including duplicates.
 
 @throws InvalidInputsError when `inputs` is invalid.
 
 @throws InvalidPatternsError when `patterns` is invalid.
 
 @example
 ```ts
 import { matcher, } from '\@monochromatic-dev/upstream-fork-matcher';
 
 matcher(['foo', 'bar', 'moo'], ['*oo', '!foo']);
 // => ['moo']
 ```
 */
export function matcher(
  {
    inputs,
    patterns,
    options,
  }: {
    readonly inputs: string | readonly string[];
    readonly patterns: string | readonly string[];
    readonly options?: MatcherOptions;
  },
): string[] {
  /**
   Normalized inputs with holes and `undefined` entries removed.
   */
  const sanitizedInputs = sanitizeInputs(inputs,)
    .inputs;
  /**
   Normalized patterns with holes and `undefined` entries removed.
   */
  const sanitizedPatterns = sanitizePatterns(patterns,)
    .patterns;
  /**
   Validated matching flags.
   */
  const resolved = resolveMatcherOptions(options,);
  /**
   Compiled matcher over the normalized patterns.
   */
  const compiled = compileMatchers({
    patterns: sanitizedPatterns,
    options: resolved,
  },);
  return sanitizedInputs.filter(function keepMatching(input: string,): boolean {
    return compiled.matches(input,);
  },);
}

/**
 Reports whether any input matches the patterns.
 
 Accepts a string or a readonly string list for both `inputs` and `patterns`.
 Uses the same per-input verdict as `matcher`, except when `allPatterns`
 holds with more than one negated pattern and no normal pattern: then every
 input must match, matching upstream `matcher`'s `isMatch` handling.
 
 @param inputs - Input string or readonly list of input strings.
 
 @param patterns - Pattern string or readonly list of pattern strings. `*`
 matches zero or more characters; a leading `!` negates the pattern.
 
 @param options - Case-sensitivity and all-patterns flags.
 
 @returns Whether the inputs match under the rule above.
 
 @throws InvalidInputsError when `inputs` is invalid.
 
 @throws InvalidPatternsError when `patterns` is invalid.
 
 @example
 ```ts
 import { isMatch, } from '\@monochromatic-dev/upstream-fork-matcher';
 
 isMatch('unicorn', 'uni*'); // => true
 isMatch('rainbow', '!unicorn'); // => true
 ```
 */
export function isMatch(
  {
    inputs,
    patterns,
    options,
  }: {
    readonly inputs: string | readonly string[];
    readonly patterns: string | readonly string[];
    readonly options?: MatcherOptions;
  },
): boolean {
  /**
   Normalized inputs with holes and `undefined` entries removed.
   */
  const sanitizedInputs = sanitizeInputs(inputs,)
    .inputs;
  /**
   Normalized patterns with holes and `undefined` entries removed.
   */
  const sanitizedPatterns = sanitizePatterns(patterns,)
    .patterns;
  /**
   Validated matching flags.
   */
  const resolved = resolveMatcherOptions(options,);
  /**
   Compiled matcher over the normalized patterns.
   */
  const compiled = compileMatchers({
    patterns: sanitizedPatterns,
    options: resolved,
  },);

  if (compiled.requiresAllInputs)
    return (sanitizedInputs.length > 0)
      && sanitizedInputs.every(function everyMatching(input: string,): boolean {
        return compiled.matches(input,);
      },);

  return sanitizedInputs.some(function someMatching(input: string,): boolean {
    return compiled.matches(input,);
  },);
}

//endregion Entry points

export {
  InvalidInputsError,
  InvalidPatternsError,
} from './errors.ts';

export {
  cacheKey,
  cacheSize,
  clearPatternCache,
  getPattern,
} from './pattern-cache.ts';

export {
  compilePattern,
  type CompiledPattern,
  matchParts,
  splitPattern,
} from './pattern.ts';

export {
  foldCase,
  foldCharacter,
  isFastPathEligible,
} from './case-fold.ts';

export {
  sanitizeInputs,
  sanitizeList,
  sanitizePatterns,
} from './inputs.ts';

export {
  type CompiledMatchers,
  compileMatchers,
} from './matcher.ts';
export {
  type MatcherOptions,
  resolveMatcherOptions,
} from './matcher-options.ts';
