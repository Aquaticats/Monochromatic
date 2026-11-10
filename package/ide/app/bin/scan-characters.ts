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

/** Code unit of the line feed, which ends a line and starts the next. */
const lineFeedCodeUnit = 0x0a;

/** Code unit of the carriage return, which ends a line on its own as well as before a line feed. */
const carriageReturnCodeUnit = 0x0d;

/** Code unit of the line separator, a Unicode line terminator without a byte of its own. */
const lineSeparatorCodeUnit = 0x2028;

/** Code unit of the paragraph separator, the fourth line terminator. */
const paragraphSeparatorCodeUnit = 0x2029;

/**
 Whether character ends a line for an anchored line-start search.
 
 @param character - Single character of the text.
 
 @returns Whether character is one of the four line terminators ECMAScript names.
 
 @example
 ```ts
 isLineTerminator('\n'); // true
 isLineTerminator('\u0085'); // false, a next-line character no pattern line start reaches
 ```
 */
export function isLineTerminator(character: string): boolean {
  if (character.length !== 1) return false;
  const code = character.charCodeAt(0);
  return code === lineFeedCodeUnit
    || code === carriageReturnCodeUnit
    || code === lineSeparatorCodeUnit
    || code === paragraphSeparatorCodeUnit;
}

/** The decimal point that stands inside a number a report prints as a plain decimal. */
const decimalPoint = '.';

/**
 Whether character may stand in a plain decimal number a report prints.
 
 @param character - Single character of the text.
 
 @returns Whether character is an ASCII digit or the decimal point.
 
 @example
 ```ts
 isDecimalNumberCharacter('.'); // true
 isDecimalNumberCharacter(','); // false
 ```
 */
export function isDecimalNumberCharacter(character: string): boolean {
  return isAsciiDigit(character) || character === decimalPoint;
}
