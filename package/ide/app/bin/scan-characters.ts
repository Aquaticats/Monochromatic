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

/** Code unit of the tab, the ASCII whitespace a log line separates fields with as well. */
const tabCodeUnit = 0x09;

/** Code unit of the vertical tab, whitespace that no line start follows. */
const verticalTabCodeUnit = 0x0b;

/** Code unit of the form feed, whitespace that no line start follows either. */
const formFeedCodeUnit = 0x0c;

/** Code unit of the space, the widest ASCII whitespace. */
const spaceCodeUnit = 0x0020;

/** Code unit of the no-break space, whitespace that is not a space. */
const noBreakSpaceCodeUnit = 0x00a0;

/** Code unit of the ogham space mark. */
const oghamSpaceMarkCodeUnit = 0x1680;

/** Code units from the en quad to the hair space, the run of spaces Unicode prints purposes with. */
const firstPrintingSpaceCodeUnit = 0x2000;

/** Last code unit of the run that starts at {@link firstPrintingSpaceCodeUnit}. */
const lastPrintingSpaceCodeUnit = 0x200a;

/** Code unit of the narrow no-break space. */
const narrowNoBreakSpaceCodeUnit = 0x202f;

/** Code unit of the medium mathematical space. */
const mediumMathematicalSpaceCodeUnit = 0x205f;

/** Code unit of the ideographic space, the widest whitespace. */
const ideographicSpaceCodeUnit = 0x3000;

/** Code unit of the byte order mark, which ECMAScript counts as whitespace. */
const byteOrderMarkCodeUnit = 0xfeff;

/**
 Whether character is whitespace the way ECMAScript names it.
 
 @param character - Single character of the text.
 
 @returns Whether character separates words for `\s`.
 
 @example
 ```ts
 isWhitespace('\u00a0'); // true
 isWhitespace('\u0085'); // false, a next-line character `\s` never reached
 ```
 */
export function isWhitespace(character: string): boolean {
  if (character.length !== 1) return false;
  const code = character.charCodeAt(0);
  return code === tabCodeUnit
    || code === verticalTabCodeUnit
    || code === formFeedCodeUnit
    || code === spaceCodeUnit
    || code === noBreakSpaceCodeUnit
    || code === oghamSpaceMarkCodeUnit
    || (code >= firstPrintingSpaceCodeUnit && code <= lastPrintingSpaceCodeUnit)
    || code === narrowNoBreakSpaceCodeUnit
    || code === mediumMathematicalSpaceCodeUnit
    || code === ideographicSpaceCodeUnit
    || code === byteOrderMarkCodeUnit
    || isLineTerminator(character);
}
