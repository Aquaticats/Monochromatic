/**
 Unicode-aware case folding for case-insensitive wildcard matching.
 
 Derived from [`matcher`](https://github.com/sindresorhus/matcher) by Sindre
 Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README for the full
 attribution. Folding semantics match upstream `matcher` 6.1.0: fold to
 uppercase exactly as a case-insensitive regular expression does, keeping
 characters whose uppercasing changes length or turns non-ASCII into ASCII so
 lookalike characters never match ASCII patterns.
 
 @module
 */

//region Constants

/**
 First code point of the surrogate range, excluded from the case-fold fast
 path so astral characters stay unchanged.
 */
const SURROGATE_FLOOR = 0xD8_00;

/**
 Last code point of the surrogate range, excluded from the case-fold fast
 path so astral characters stay unchanged.
 */
const SURROGATE_CEILING = 0xDF_FF;

/**
 Highest ASCII code point. Characters above it fold through the guarded
 per-character path.
 */
const ASCII_CEILING = 0x7F;

/**
 Characters whose uppercasing turns non-ASCII into ASCII. Upstream `matcher`
 names exactly these two: `ı` folds to `I` and `ſ` folds to `S`.
 */
const ASCII_FOLDING_EXCEPTIONS = new Set([
  'ı',
  'ſ',
],);

//endregion Constants

//region Folding

/**
 Uppercases one non-ASCII character the way a case-insensitive regular
 expression does.
 
 Keeps the character when uppercasing changes its length (such as `ﬁ` to
 `FI`) or turns it into ASCII (such as `ß` to `SS` or `ı` to `I`), so
 lookalike characters do not match ASCII patterns.
 
 @param character - Single non-ASCII character to fold.
 
 @returns Uppercased character, or the original when folding would merge it
 with an ASCII lookalike or expand its length.
 
 @example
 ```ts
 foldCharacter('é'); // => 'É'
 foldCharacter('ß'); // => 'ß'
 ```
 */
export function foldCharacter(character: string,): string {
  /**
   Uppercased form of the character.
   */
  const uppercased = character.toUpperCase();

  /**
   Code point of the original character.
   */
  const sourcePoint = character.codePointAt(0,)
    ?? 0;

  /**
   Code point of the uppercased form's first character.
   */
  const foldedPoint = uppercased.codePointAt(0,)
    ?? 0;

  if ((uppercased.length !== 1)
    || ((sourcePoint > ASCII_CEILING) && (foldedPoint <= ASCII_CEILING)))
    return character;

  return uppercased;
}

/**
 Reports whether every character of `value` folds through the whole-string
 fast path: no `ı`, no `ſ`, and no surrogate halves.
 
 Surrogates are excluded to keep astral characters unchanged, matching
 upstream `matcher`'s fast-path guard.
 
 @param value - String to inspect.
 
 @returns `true` when the whole-string fast path may run.
 
 @example
 ```ts
 isFastPathEligible('abc'); // => true
 ```
 */
export function isFastPathEligible(value: string,): boolean {
  /**
   UTF-16 unit index; units are inspected one at a time so a paired
   surrogate still reports its halves, exactly as upstream `matcher`'s
   character-class test inspects them.
   */
  const unitCursor = { index: 0, };
  while (unitCursor.index < value.length) {
    /**
     UTF-16 unit under inspection.
     */
    const unit = value[unitCursor.index]
      ?? '';
    if (ASCII_FOLDING_EXCEPTIONS.has(unit,))
      return false;
    /**
     Code point of the unit (a lone surrogate reports its own value).
     */
    const point = unit.codePointAt(0,)
      ?? 0;
    if ((point >= SURROGATE_FLOOR) && (point <= SURROGATE_CEILING))
      return false;
    unitCursor.index += 1;
  }

  return true;
}

/**
 Folds one string through the guarded per-character path: non-ASCII
 characters fold via `foldCharacter`, then ASCII lowercase runs uppercase.
 Surrogates iterate as lone UTF-16 units (never paired), so a wildcard can
 match half of a surrogate pair exactly as a non-`u`-flag regular expression
 does.
 
 @param value - String to fold.
 
 @returns Folded string.
 
 @example
 ```ts
 foldSlowPath('aé'); // => 'AÉ'
 ```
 */
function foldSlowPath(value: string,): string {
  /**
   Value with every non-ASCII character folded through the guarded path.
   Walked by UTF-16 unit (never pairing surrogates), so astral characters
   fold as lone halves exactly as upstream `matcher`'s
   `replaceAll(/\\P{ASCII}/gu, ...)` callback receives them.
   */
  const foldedUnits: string[] = [];
  /**
   UTF-16 unit index; advanced per unit below so surrogates never pair.
   */
  const unitCursor = { index: 0, };
  while (unitCursor.index < value.length) {
    /**
     UTF-16 unit under the fold.
     */
    const unit = value[unitCursor.index]
      ?? '';
    /**
     Code point of the unit (a lone surrogate reports its own value).
     */
    const point = unit.codePointAt(0,)
      ?? 0;
    if (point <= ASCII_CEILING)
      foldedUnits.push(unit,);
    else
      foldedUnits.push(foldCharacter(unit,),);
    unitCursor.index += 1;
  }
  /**
   Folded non-ASCII value rebuilt from its units.
   */
  const foldedNonAscii = foldedUnits.join('',);
  /**
   Characters of the non-ASCII-folded value, walked once with ASCII lowercase
   runs uppercased inline.
   */
  const foldedCharacters: string[] = [];
  /**
   Start of the pending ASCII lowercase run, or -1 when not inside one.
   */
  const runState = { start: -1, };
  for (let index = 0; index <= foldedNonAscii.length; index += 1) {
    /**
     Character at this position, or empty at the end to flush the run.
     */
    const character = foldedNonAscii[index]
      ?? '';
    /**
     Whether this character continues an ASCII lowercase run.
     */
    const continuesRun = (character >= 'a') && (character <= 'z');
    if (continuesRun) {
      if (runState.start === (-1))
        runState.start = index;
      continue;
    }
    if (runState.start !== (-1)) {
      foldedCharacters.push(foldedNonAscii.slice(
        runState.start,
        index,
      )
        .toUpperCase(),);
      runState.start = -1;
    }
    if (index < foldedNonAscii.length)
      foldedCharacters.push(character,);
  }
  return foldedCharacters.join('',);
}

/**
 Folds one string for case-insensitive comparison, matching upstream
 `matcher`'s `normalizeCase`.
 
 In case-sensitive mode the string passes through unchanged. Otherwise the
 whole-string fast path runs first when eligible and length-preserving;
 inputs needing guarded folding fall back to the per-character path for
 non-ASCII characters plus an ASCII-letter run pass.
 
 @param value - String to fold.
 
 @param caseSensitive - When `true`, returns the string unchanged.
 
 @returns Folded string used for both pattern compilation and input comparison.
 
 @example
 ```ts
 foldCase('Unicorn', false); // => 'UNICORN'
 foldCase('Unicorn', true); // => 'Unicorn'
 ```
 */
export function foldCase(
  {
    value,
    caseSensitive,
  }: {
    readonly value: string;
    readonly caseSensitive: boolean;
  },
): string {
  if (caseSensitive)
    return value;

  if (isFastPathEligible(value,)) {
    /**
     Whole-string uppercased candidate for the fast path.
     */
    const uppercased = value.toUpperCase();
    if (uppercased.length === value.length)
      return uppercased;
    return foldSlowPath(value,);
  }

  return foldSlowPath(value,);


}

//endregion Folding
