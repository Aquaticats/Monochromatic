// Character classes that the inspection scripts' single-pass scans decide on, one owner for each class.
// What: predicates for an ASCII digit, a hexadecimal digit, a word character, the whitespace ECMAScript names,
// and the four line terminators.
// Why: each scan walks characters where a banned pattern walked them, so the set a pattern reached is spelled in
// one place and cannot drift; the ASCII predicates are narrower than a Unicode class and say so.
// Tested in scan-characters.test.ts.

/** Code unit of `0`, the lowest ASCII digit. */
const lowestDigitCodeUnit = '0'.charCodeAt(0);

/** Code unit of `9`, the highest ASCII digit. */
const highestDigitCodeUnit = '9'.charCodeAt(0);

/**
 Whether character is one of the ten ASCII digits.
 
 @param character - Single character of the text.
 
 @returns Whether character lies from `0` to `9`.
 
 @example
 ```ts
 isAsciiDigit('7'); // true
 isAsciiDigit('٧'); // false, an Arabic-Indic digit
 ```
 */
export function isAsciiDigit(character: string): boolean {
  if (character.length !== 1) return false;
  const code = character.charCodeAt(0);
  return code >= lowestDigitCodeUnit && code <= highestDigitCodeUnit;
}
