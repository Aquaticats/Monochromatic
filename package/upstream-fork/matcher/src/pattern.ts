/**
 Wildcard pattern compilation and matching.
 
 Derived from [`matcher`](https://github.com/sindresorhus/matcher) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Matching semantics match upstream `matcher` 6.1.0: `*` matches
 zero or more characters (including newlines), a backslash makes the next
 character literal, and a leading `!` negates the pattern.
 
 @module
 */

import { foldCase, } from './case-fold.ts';

//region Types

/**
 One compiled pattern: its negation flag and a test over folded inputs.
 
 The test expects an input already passed through `foldCase` with the same
 case-sensitivity, matching upstream `matcher`'s compiled-pattern contract.
 
 @example
 ```ts
 const compiled: CompiledPattern = compilePattern({ pattern: 'a*', caseSensitive: false, });
 compiled.test('ABC'); // => true only when 'ABC' is already folded
 ```
 */
export type CompiledPattern = {
  /**
   Whether the pattern was negated with a leading `!`.
   */
  readonly negated: boolean;
  /**
   Tests one folded input against the folded literal parts.
   
   @param input - Input already passed through `foldCase`.
   
   @returns Whether the folded input matches.
   */
  readonly test: (input: string,) => boolean;
};

//endregion Types

//region Splitting

/**
 Code unit for `*`, the only wildcard character.
 */
const STAR_UNIT = '*';

/**
 Code unit for the backslash escape character.
 */
const ESCAPE_UNIT = '\\';

/**
 Negation prefix: only a leading `!` negates, matching upstream `matcher`.
 */
const NEGATION_PREFIX = '!';

/**
 Splits a pattern into the literal parts between unescaped `*` wildcards.
 
 A backslash makes the next character literal: both the backslash and the
 escaped character leave the wildcard scan, and the escaped character joins
 the current literal part. A trailing backslash with nothing after it stays
 literal. Slices (not single characters) are appended, since each append
 adds a rope node.
 
 @param pattern - Raw pattern text after negation stripping and case folding.
 
 @returns Literal parts; length 1 means the pattern holds no wildcard.
 
 @example
 ```ts
 splitPattern('a*b'); // => ['a', 'b']
 splitPattern(String.raw`a\*b`); // => ['a*b']
 ```
 */
export function splitPattern(pattern: string,): readonly string[] {
  /**
   Literal parts collected so far, in pattern order.
   */
  const parts: string[] = [];
  /**
   Escaped characters accumulated for the part under construction. Stays
   separate from the slice window so `part + slice` never duplicates text.
   */
  const state = {
    part: '',
    sliceStart: 0,
  };

  for (let index = 0; index < pattern.length; index += 1) {
    /**
     Code unit under the scan.
     */
    const character = pattern[index];
    if (character === STAR_UNIT) {
      parts.push(state.part + pattern.slice(
        state.sliceStart,
        index,
      ),);
      state.part = '';
      state.sliceStart = index + 1;
    }
    else if ((character === ESCAPE_UNIT) && ((index + 1) < pattern.length)) {
      state.part += pattern.slice(
        state.sliceStart,
        index,
      );
      state.sliceStart = index + 1;
      index += 1;
    }
  }

  parts.push(state.part + pattern.slice(state.sliceStart,),);
  return parts;
}

/**
 Reports whether a folded input matches folded literal parts.
 
 Greedy matching of each part at its first possible position is correct for
 `*`-only wildcards and never backtracks, unlike a regular expression, so
 the time is at most proportional to input length times pattern length.
 
 @param input - Folded input to test.
 
 @param parts - Folded literal parts from `splitPattern`.
 
 @returns Whether the input matches.
 
 @example
 ```ts
 matchParts({ input: 'foobar', parts: ['foo', 'bar'], }); // => true
 ```
 */
export function matchParts(
  {
    input,
    parts,
  }: {
    readonly input: string;
    readonly parts: readonly string[];
  },
): boolean {
  if (parts.length === 1)
    return input === parts[0];

  /**
   Literal prefix the input must start with.
   */
  const first = parts[0]
    ?? '';
  /**
   Literal suffix the input must end with.
   */
  const last = parts.at(-1,)
    ?? '';
  /**
   Latest start offset the suffix may occupy.
   */
  const end = input.length - last.length;

  /**
   Whether the input opens with the literal prefix.
   */
  const prefixHits = input.startsWith(first,);
  /**
   Whether the input closes with the literal suffix.
   */
  const suffixHits = input.endsWith(last,);
  /**
   Whether the input leaves room for the prefix before the suffix.
   */
  const roomHolds = end >= first.length;
  if (!prefixHits)
    return false;
  if (!suffixHits)
    return false;
  if (!roomHolds)
    return false;

  /**
   Earliest offset the next middle part may occupy.
   */
  const cursor = { position: first.length, };

  for (let partIndex = 1; partIndex < (parts.length - 1); partIndex += 1) {
    /**
     Middle literal part to find after the cursor.
     */
    const part = parts[partIndex]
      ?? '';
    /**
     Found offset of the middle part at or after the cursor.
     */
    const found = input.indexOf(
      part,
      cursor.position,
    );
    if ((found === (-1)) || ((found + part.length) > end))
      return false;
    cursor.position = found + part.length;
  }

  return true;
}

//endregion Splitting

//region Compilation

/**
 Compiles one pattern into its negation flag and literal parts.
 
 A leading `!` negates the pattern; only the leading position negates, so
 `!!foo` compiles to a negated pattern matching `!foo`. The remainder is
 case-folded before splitting, matching upstream `matcher`'s compile order.
 
 @param pattern - Raw pattern text.
 
 @param caseSensitive - Whether folding passes the pattern through unchanged.
 
 @returns Compiled pattern whose test expects a folded input.
 
 @example
 ```ts
 const compiled = compilePattern({ pattern: '!a*', caseSensitive: false, });
 compiled.negated; // => true
 ```
 */
export function compilePattern(
  {
    pattern,
    caseSensitive,
  }: {
    readonly pattern: string;
    readonly caseSensitive: boolean;
  },
): CompiledPattern {
  /**
   Whether the raw pattern carries the leading negation prefix.
   */
  const negated = pattern.startsWith(NEGATION_PREFIX,);
  /**
   Pattern remainder after negation stripping.
   */
  const bare = negated
    ? pattern.slice(1,)
    : pattern;
  /**
   Folded literal parts, copied so the cache holds flat strings of their
   own. Otherwise V8 can keep rope nodes, or the larger string the pattern
   was sliced from, in memory.
   */
  const parts: readonly string[] = [
    ...splitPattern(foldCase({
      value: bare,
      caseSensitive,
    },),),
  ];

  return {
    negated,
    test: function testPattern(input: string,): boolean {
      return matchParts({
        input,
        parts,
      },);
    },
  };
}

//endregion Compilation
