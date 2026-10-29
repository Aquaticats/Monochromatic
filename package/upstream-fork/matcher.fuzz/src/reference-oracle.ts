/**
 Reference oracle for single-pattern matching: the documented matcher
 semantics as an independent implementation.
 
 The oracle compiles one pattern with only string scans (no shared code with
 the fork): split on unescaped `*`, fold case through the fork's own fold
 (the fold is verified separately against upstream's regex behavior), and
 apply the anchored greedy matcher. Multi-pattern combination logic
 (negation-first, all/any modes, requires-all-inputs) is verified by the
 differential oracle against upstream itself, so this oracle pins only the
 single-pattern core.
 
 @module
 */

import { foldCase, } from '@monochromatic-dev/upstream-fork-matcher/ts/case-fold.ts';
import { splitPattern, } from '@monochromatic-dev/upstream-fork-matcher/ts/pattern.ts';

//region Oracle

/**
 Reports whether one folded input matches one folded pattern's parts,
 using an independent backtracking matcher (not the fork's greedy scan).
 
 Backtracking tries every split of the input across the wildcard gaps, so
 agreement with the fork's greedy first-position scan proves the greedy
 choice correct on every generated case.
 
 @param input - Folded input.
 
 @param parts - Folded literal parts.
 
 @returns Whether the input matches.
 
 @example
 ```ts
 referenceMatchParts({ input: 'aab', parts: ['', 'a', 'ab'], }); // => true
 ```
 */
export function referenceMatchParts(
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

  return searchFrom({
    input,
    parts,
    partIndex: 0,
    offset: 0,
  },);
}

/**
 Recursive backtracking search over parts and input offsets.
 
 @param input - Folded input.
 
 @param parts - Folded literal parts.
 
 @param partIndex - Index of the part under test.
 
 @param offset - Input offset the part must match at or after.
 
 @returns Whether the remaining parts match the remaining input.
 
 @example
 ```ts
 searchFrom({ input: 'ab', parts: ['a', 'b'], partIndex: 0, offset: 0, });
 ```
 */
function searchFrom(
  {
    input,
    parts,
    partIndex,
    offset,
  }: {
    readonly input: string;
    readonly parts: readonly string[];
    readonly partIndex: number;
    readonly offset: number;
  },
): boolean {
  /**
   Part under test at this recursion level.
   */
  const part = parts[partIndex]
    ?? '';

  if (partIndex === (parts.length - 1))
    return input.endsWith(part,)
      && ((input.length - part.length) >= offset);

  if (partIndex === 0) {
    if (!input.startsWith(part,))
      return false;
    return searchFrom({
      input,
      parts,
      partIndex: partIndex + 1,
      offset: part.length,
    },);
  }

  /**
   Furthest offset this part may start at.
   */
  const last = parts.at(-1,)
    ?? '';
  /**
   Latest start keeping the suffix reachable.
   */
  const limit = input.length - last.length;

  /**
   Candidate start offset walked one unit at a time.
   */
  const cursor = { start: offset, };
  while (cursor.start <= limit) {
    /**
     Whether the part matches exactly at the candidate start.
     */
    const hits = input.startsWith(
      part,
      cursor.start,
    );
    if (hits && searchFrom({
      input,
      parts,
      partIndex: partIndex + 1,
      offset: cursor.start + part.length,
    },))
      return true;
    cursor.start += 1;
  }

  return false;
}

/**
 Reports whether one raw input matches one raw pattern, per the documented
 single-pattern semantics.
 
 @param input - Raw input string.
 
 @param pattern - Raw pattern string, possibly negated.
 
 @param caseSensitive - Whether matching folds case.
 
 @returns Whether the input matches (negated when the pattern is negated).
 
 @example
 ```ts
 referenceIsMatch({ input: 'foobar', pattern: 'foo*', caseSensitive: false, });
 ```
 */
export function referenceIsMatch(
  {
    input,
    pattern,
    caseSensitive,
  }: {
    readonly input: string;
    readonly pattern: string;
    readonly caseSensitive: boolean;
  },
): boolean {
  /**
   Whether the raw pattern carries the leading negation prefix.
   */
  const negated = pattern.startsWith('!',);
  /**
   Pattern remainder after negation stripping.
   */
  const bare = negated
    ? pattern.slice(1,)
    : pattern;
  /**
   Folded input shared by the match.
   */
  const foldedInput = foldCase({
    value: input,
    caseSensitive,
  },);
  /**
   Folded literal parts of the pattern.
   */
  const parts = splitPattern(foldCase({
    value: bare,
    caseSensitive,
  },),);
  /**
   Positive match verdict before negation.
   */
  const matched = referenceMatchParts({
    input: foldedInput,
    parts,
  },);
  return negated
    ? !matched
    : matched;
}

//endregion Oracle
