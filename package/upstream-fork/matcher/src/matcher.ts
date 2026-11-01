/**
 Compiled pattern-set matching: partition one pattern list and test inputs.

 Derived from [`matcher`](https://github.com/sindresorhus/matcher) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution.

 @module
 */

import { foldCase, } from './case-fold.ts';
import type { CompiledPattern, } from './pattern.ts';
import { getPattern, } from './pattern-cache.ts';

//region Empty set

/**
 Empty-set matcher: an empty pattern list matches nothing.
 */
const emptySetMatchers: CompiledMatchers = {
  matches: function matchesNothing(): boolean {
    return false;
  },
  requiresAllInputs: false,
};

//endregion Empty set

//region Compilation

/**
 Compiled matcher over one pattern list: tests one input at a time and
 reports whether `isMatch` must require every input to match.
 
 @example
 ```ts
 const compiled = compileMatchers({
   patterns: ['a*'],
   options: { caseSensitive: false, allPatterns: false, },
 });
 compiled.matches('abc'); // => true
 ```
 */
export type CompiledMatchers = {
  /**
   Tests one raw input against the compiled patterns.
   
   @param input - Raw input string; folded inside before testing.
   
   @returns Whether the input matches under the resolved options.
   */
  readonly matches: (input: string,) => boolean;
  /**
   Whether `isMatch` must require every input to match. Set only when
   `allPatterns` holds with more than one negated pattern and no normal
   pattern, matching upstream `matcher`'s multi-negation handling.
   */
  readonly requiresAllInputs: boolean;
};

/**
 Compiles one pattern list into partitioned compiled patterns.
 
 An empty pattern list matches nothing. Otherwise patterns split into
 negated and normal sets: a negated hit excludes immediately, an empty
 normal set includes when no negation hit, `allPatterns` requires every
 normal pattern to hit, and default mode requires any normal pattern hit.
 
 @param patterns - Normalized pattern strings.
 
 @param options - Validated matching flags.
 
 @returns Compiled matcher plus the multi-negation mode flag.
 
 @example
 ```ts
 const compiled = compileMatchers({
   patterns: ['*oo', '!foo'],
   options: { caseSensitive: false, allPatterns: false, },
 });
 ```
 */
export function compileMatchers(
  {
    patterns,
    options,
  }: {
    readonly patterns: readonly string[];
    readonly options: {
      readonly caseSensitive: boolean;
      readonly allPatterns: boolean;
    };
  },
): CompiledMatchers {
  if (patterns.length === 0)
    return emptySetMatchers;

  /**
   Every pattern compiled through the shared cache.
   */
  const compiled: readonly CompiledPattern[] = patterns.map(function compileOne(pattern: string,): CompiledPattern {
    return getPattern({
      pattern,
      caseSensitive: options.caseSensitive,
    },);
  },);
  /**
   Negated patterns, checked first for immediate exclusion.
   */
  const negated = compiled.filter(function isNegated(entry: CompiledPattern,): boolean {
    return entry.negated;
  },);
  /**
   Normal patterns, checked after the negations.
   */
  const positive = compiled.filter(function isPositive(entry: CompiledPattern,): boolean {
    return !entry.negated;
  },);

  /**
   Tests one raw input: folds it, then applies the negation-first,
   positive-second decision order described above.
   
   @param input - Raw input string.
   
   @returns Whether the input matches.
   */
  function matches(input: string,): boolean {
    /**
     Folded input shared by every pattern test.
     */
    const folded = foldCase({
      value: input,
      caseSensitive: options.caseSensitive,
    },);

    for (const entry of negated)
      if (entry.test(folded,))
        return false;

    if (positive.length === 0)
      return true;

    if (options.allPatterns) {
      for (const entry of positive)
        if (!entry.test(folded,))
          return false;
      return true;
    }

    for (const entry of positive)
      if (entry.test(folded,))
        return true;
    return false;
  }

  return {
    matches,
    requiresAllInputs: options.allPatterns && (negated.length > 1)
      && (positive.length === 0),
  };
}

//endregion Compilation
